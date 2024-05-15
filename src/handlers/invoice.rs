use std::net::SocketAddr;

use crate::models::db_models::BillineInvoiceStatus;
use crate::models::json_responses::{BillineCreateInvoiceResponse, Prices};
use crate::models::{db_models::Invoice, json_responses::OneTimeToken};
use crate::tools::blake_hash;
use crate::{config, WsManagerEvent, WsManagerEventSender};

use billine::CallbackIframe;
use p2way::P2Way;
use qrcode_generator::QrCodeEcc;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use thedex::models::CreateQuickInvoice;
use thedex::TheDex;
use tracing::{debug, error, info, warn};

use self::json_requests::{CreateBillineInvoice, PayoutRequest};

use super::*;

/// Callback
///
/// Callback
#[utoipa::path(
        tag="invoice",
        post,
        path = "/api/invoice/callback",
        request_body = CreateInvoice,
        responses(
            (status = 200, description = "Answer", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn invoice_callback(
    _: bool,
    invoice: thedex::models::Invoice,
    db: DB,
    manager_writer: WsManagerEventSender,
) -> Result<WarpResponse, warp::Rejection> {
    match invoice.status {
        thedex::models::InvoiceStatus::Successful => {
            if let Some(order_id) = &invoice.order_id {
                if let Err(e) = db
                    .invoice_update_status(order_id, invoice.status.clone() as i32)
                    .await
                {
                    error!("Error updating invoice: {:?}", e);
                    return Err(reject::custom(ApiError::UpdateAmountsError));
                }
                if let Some(client_id) = &invoice.client_id {
                    let client_id = if let Ok(client_id) = i64::from_str_radix(&client_id, 10) {
                        client_id
                    } else {
                        error!("Error converting client_id: {:?}", invoice);
                        return Err(reject::custom(ApiError::UpdateAmountsError));
                    };

                    db.increase_amounts_by_usdt_amount(client_id, &invoice.amount.ceil())
                        .await
                        .map_err(|e| error!("Error updating invoice: {:?}", e))
                        .map_err(|_| ApiError::UpdateAmountsError)?;
                } else {
                    error!("Client id not found in invoice: {:?}", invoice);
                    return Err(reject::custom(ApiError::UpdateAmountsError));
                }
            } else {
                error!("Order id not found in invoice: {:?}", invoice);
                return Err(reject::custom(ApiError::UpdateAmountsError));
            }
        }
        thedex::models::InvoiceStatus::Rejected | thedex::models::InvoiceStatus::Unpaid => {
            debug!("Rejected invoice: {:?}", &invoice);
            if let Some(order_id) = &invoice.order_id {
                if let Err(e) = db
                    .invoice_update_status(order_id, invoice.status.clone() as i32)
                    .await
                {
                    error!("Error updating invoice: {:?}", e);
                    return Err(reject::custom(ApiError::UpdateAmountsError));
                } else {
                    error!("Client id not found in invoice: {:?}", invoice);
                    return Err(reject::custom(ApiError::UpdateAmountsError));
                }
            } else {
                error!("Order id not found in invoice: {:?}", invoice);
                return Err(reject::custom(ApiError::UpdateAmountsError));
            }
        }
        _ => {
            warn!("Not handling callback for invoice {:?}", invoice);
        }
    }

    manager_writer.send(WsManagerEvent::PropagateInvoice(invoice.into()));
    Ok(gen_info_response("Ok"))
}

/// Callback
///
/// Callback
#[utoipa::path(
        tag="invoice",
        post,
        path = "/api/invoice/billine/callback",
        request_body = CreateInvoice,
        responses(
            (status = 200, description = "Answer", body = CallbackIframe),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn invoice_billine_callback(
    invoice: CallbackIframe,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    info!("Billine Callback: {:?}", invoice);
    let calc_signature = billine::md5_signature(&invoice, &config::BILLINE_SECRET);
    if !calc_signature.eq(&invoice.co_sign) {
        error!(
            "Billine signatire is wrong: {:?} != {:?}",
            calc_signature, invoice.co_sign
        );
        return Err(reject::custom(ApiError::UpdateAmountsError));
    }
    match invoice.co_inv_st {
        billine::Status::Success => {
            db.billine_invoice_update_status(&invoice.co_order_no, BillineInvoiceStatus::Success)
                .await
                .map_err(|e| {
                    error!("Billine callback update status: {:?}", e);
                    e
                })
                .map_err(ApiError::DbError)?;
            let stored_invoice = db
                .fetch_billine_invoice(&invoice.co_order_no)
                .await
                .map_err(|e| {
                    error!("Billine fetching invoice: {:?}", e);
                    e
                })
                .map_err(ApiError::DbError)?;

            // TODO: calculate amount in USD from currency
            let amount_in_usd = invoice
                .co_amount
                .ok_or(ApiError::UpdateAmountsError)
                .map_err(|e| {
                    error!(
                        "Error increasing amount for user {:?}: {:?}",
                        stored_invoice, invoice
                    );
                    e
                })?
                .ceil();

            db.increase_amounts_by_usdt_amount(stored_invoice.user_id, &amount_in_usd)
                .await
                .map_err(|e| error!("Error updating invoice: {:?}", e))
                .map_err(|_| ApiError::UpdateAmountsError)?;
        }
        billine::Status::Fail => {
            db.billine_invoice_update_status(&invoice.co_order_no, BillineInvoiceStatus::Failed)
                .await
                .map_err(|e| {
                    error!("Billine callback update status: {:?}", e);
                    e
                })
                .map_err(ApiError::DbError)?;
        }
    }
    Ok(gen_raw_text_response("OK"))
}

/// Get prices
///
/// Gets prices, requires authentication
#[utoipa::path(
        tag="invoice",
        get,
        path = "/api/invoice/prices",
        responses(
            (status = 200, description = "Prices", body = Prices),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn crypto_prices(_: i64, mut dex: TheDex) -> Result<WarpResponse, warp::Rejection> {
    let response = dex
        .prices(chrono::Utc::now().timestamp_millis() as u64)
        .await
        .map_err(ApiError::TheDexError)?;

    Ok(gen_arbitrary_response(ResponseBody::Prices(Prices {
        prices: response,
    })))
}

/// Gets an existing invoice
///
/// Gets an existing invoice
#[utoipa::path(
        tag="invoice",
        get,
        path = "/api/invoice/{invoice_id}",
        responses(
            (status = 200, description = "User account was created", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("invoice_id" = String, Path, description = "Id of the invoice, returned by the invoice creation endpoint")
        )
    )]
pub async fn get_invoice(
    invoice_id: String,
    id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let invoice = db
        .fetch_invoice(&invoice_id)
        .await
        .map_err(ApiError::DbError)?;

    Ok(gen_arbitrary_response(ResponseBody::Invoice(Invoice {
        id: invoice_id,
        merchant_id: "EVYWM38X".into(),
        order_id: invoice.order_id,
        create_date: invoice.create_date,
        status: invoice.status,
        pay_url: invoice.pay_url,
        user_id: id,
        amount: invoice.amount,
        currency: invoice.currency,
    })))
}

/// Create a new invoice
///
/// Creates a new invoice
#[utoipa::path(
        tag="invoice",
        post,
        path = "/api/invoice/create",
        request_body = CreateInvoice,
        responses(
            (status = 200, description = "User account was created", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn create_invoice(
    data: CreateInvoice,
    id: i64,
    db: DB,
    mut dex: TheDex,
) -> Result<WarpResponse, warp::Rejection> {
    let order_id = blake_hash(&format!(
        "{}{}{}{}",
        id,
        data.amount.clone() as u32,
        data.currency,
        chrono::offset::Utc::now().timestamp_millis()
    ));
    let amount = Decimal::from_u32(data.amount.clone() as u32).unwrap();
    let prices = dex
        .prices(chrono::offset::Utc::now().timestamp_millis() as u64)
        .await
        .map_err(ApiError::TheDexError)?;

    let short_curr = &data
        .currency
        .split('_')
        .next()
        .ok_or(ApiError::UnknownCurrency(data.currency.clone()))?;
    let price = prices
        .iter()
        .find(|el| el.monetary.short.eq(short_curr))
        .ok_or(ApiError::UnknownCurrency(data.currency.clone()))?
        .rates
        .iter()
        .find(|el| el.fiat_currency.eq("USD"))
        .ok_or(ApiError::UnknownCurrency("USD".into()))?;

    let currency_amount = amount / price.rate;

    let result = dex
        .create_quick_invoice(
            CreateQuickInvoice {
                amount: currency_amount,
                pay_currency: data.currency.clone(),
                merchant_id: "EVYWM38X".into(),
                order_id: Some(order_id.clone()),
                email: None,
                client_id: Some(id.to_string()),
                title: Some(format!("Bying {}", currency_amount)),
                description: None,
                recalculation: Some(true),
                needs_email_confirmation: Some(false),
                success_url: Some(String::from(
                    "https://game.greekkeepers.io/api/invoice/success",
                )),
                failure_url: Some(String::from(
                    "https://game.greekkeepers.io/api/invoice/failure",
                )),
                callback_url: Some(String::from(
                    "https://game.greekkeepers.io/api/invoice/callback",
                )),
                unfix_amount: Some(false),
            },
            chrono::offset::Utc::now().timestamp_millis() as u64,
        )
        .await
        .map_err(ApiError::TheDexError)?;

    db.add_invoice(
        &order_id,
        "EVYWM38X",
        &order_id,
        result.status.clone() as i32,
        &result.purse,
        id,
        amount,
        &data.currency,
    )
    .await
    .map_err(ApiError::DbError)?;

    Ok(gen_arbitrary_response(ResponseBody::Invoice(Invoice {
        id: order_id.clone(),
        merchant_id: "EVYWM38X".into(),
        order_id,
        create_date: Default::default(),
        status: result.status as i32,
        pay_url: result.purse,
        user_id: id,
        amount,
        currency: data.currency,
    })))
}

/// Create a new billine invoice
///
/// Creates a new billine invoice `https://billline.net/ru/api-ru/#iframe-primary-request`
#[utoipa::path(
        tag="invoice",
        post,
        path = "/api/invoice/billine/create",
        request_body = CreateBillineInvoice,
        responses(
            (status = 200, description = "Invoice was created", body = BillineCreateInvoiceResponse),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn billine_create_invoice(
    data: CreateBillineInvoice,
    id: i64,
    db: DB,
    address: SocketAddr,
) -> Result<WarpResponse, warp::Rejection> {
    let order_id = blake_hash(&format!(
        "{}{}{}{}",
        id,
        data.amount.clone(),
        &data.currency,
        chrono::offset::Utc::now().timestamp_millis()
    ));
    let item_name = format!(
        "Purchasing for {} {} from GreekKeepers",
        data.amount, &data.currency
    );

    db.add_billine_invoice(
        &order_id,
        "EVYWM38X",
        &order_id,
        id,
        data.amount,
        &data.currency,
    )
    .await
    .map_err(ApiError::DbError)?;

    let data = billine::RequestIframe {
        merchant: config::BILLINE_MERCHANT.to_string(),
        order: order_id.clone(),
        amount: data.amount,
        currency: data.currency,
        item_name,
        first_name: data.first_name,
        last_name: data.last_name,
        user_id: id.to_string(),
        payment_url: "https://game.greekkeepers.io".to_string(),
        country: data.country,
        ip: address.ip().to_string(),
        custom: "".to_string(),
        email: data.email,
        phone: data.phone,
        address: data.address,
        city: data.city,
        post_code: data.post_code,
        region: data.region,
        lang: billine::Language::En,
        cpf: None,
    };
    let sign = billine::sha256_signature(&data, &config::BILLINE_SECRET);

    Ok(gen_arbitrary_response(ResponseBody::BillineCreateInvoice(
        BillineCreateInvoiceResponse { data, sign },
    )))
}
/// Generate qr code
///
/// Generates qr code from the specified data
#[utoipa::path(
        tag="invoice",
        get,
        path = "/api/invoice/qr/{invoice_id}",
        responses(
            (status = 200, description = "Get QR code for the invoice", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("invoice_id" = String, Path, description = "Id of the invoice, returned by the invoice creation endpoint")
        )
    )]
pub async fn generate_qr(invoice_id: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let invoice = db
        .fetch_invoice(&invoice_id)
        .await
        .map_err(ApiError::DbError)?;
    Ok(get_pgn_response(
        qrcode_generator::to_png_to_vec(&invoice.pay_url, QrCodeEcc::Low, 1024)
            .map_err(|_| ApiError::QrGenerationError(invoice_id))?,
    ))
}

/// Create a new p2p session token
///
/// Creates a new p2p token
#[utoipa::path(
        tag="invoice",
        get,
        path = "/api/p2way/ott",
        responses(
            (status = 200, description = "one time token", body = OneTimeToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn create_p2way_token(_: i64, p2way: P2Way) -> Result<WarpResponse, warp::Rejection> {
    let token = p2way
        .one_time_token_generation()
        .await
        .map_err(ApiError::P2WayError)?;

    Ok(gen_arbitrary_response(ResponseBody::OneTimeToken(
        OneTimeToken { token: token.token },
    )))
}

/// Callback
///
/// Callback
#[utoipa::path(
        tag="invoice",
        get,
        path = "/api/p2way/callback",
        responses(
            (status = 200, description = "OK", body = OneTimeToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn p2way_callback(
    data: p2way::models::CallbackResponse,
    db: DB,
    address: SocketAddr,
) -> Result<WarpResponse, warp::Rejection> {
    info!("P2Way callback {:?}, address: {:?}", data, address);
    if !config::P2WAY_SECRETKEY_HASH.eq(&data.data.merchant_secret_key) {
        info!("P2Way callback rejected, bad api_key");
        return Err(reject::custom(ApiError::UpdateAmountsError));
    }

    match data.data.order_state {
        p2way::OrderState::Success => db
            .increase_amounts_by_usdt_amount(
                i64::from_str_radix(&data.data.user_id, 10).map_err(|e| {
                    info!("Error on p2way callback: {:?}", e);
                    ApiError::UpdateAmountsError
                })?,
                &data.data.amount_from_user_in_usdt,
            )
            .await
            .map_err(|e| {
                info!("Error on p2way callback: {:?}", e);
                ApiError::UpdateAmountsError
            })?,
        p2way::OrderState::Canceled => {}
        p2way::OrderState::CanceledByUser => {}
    }

    info!("P2Way callback rejected, bad api_key");
    Ok(gen_raw_text_response("Ok"))
}

/// Create a new payout request
///
/// Creates a new payout request
#[utoipa::path(
        tag="invoice",
        post,
        path = "/api/payout/create",
        request_body = PayoutRequest,
        responses(
            (status = 200, description = "Request submitted", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn create_payout_request(
    data: PayoutRequest,
    user_id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.new_payout_request(user_id, data.amount, data.additional_data)
        .await
        .map_err(ApiError::DbError)?;

    Ok(gen_info_response("Request submitted"))
}

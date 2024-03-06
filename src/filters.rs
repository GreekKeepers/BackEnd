// use crate::communication::WsDataFeedReceiver;
// use crate::communication::WsDataFeedSender;
use crate::config::PASSWORD_SALT;
use crate::db::DB;
use crate::errors::ApiError;
use crate::handlers;
use crate::jwt;
use crate::jwt::Payload;
use crate::models::json_requests;
use crate::tools;
use crate::EngineBetSender;

use crate::WsManagerEventSender;
use base64::{engine::general_purpose, Engine as _};
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::net::SocketAddr;
use std::str;
use thedex::TheDex;
use tracing::debug;
use warp::filters::header::headers_cloned;
use warp::reject;

use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_thedex(
    dex: TheDex,
) -> impl Filter<Extract = (TheDex,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || dex.clone())
}

fn with_manager_channel(
    ch: WsManagerEventSender,
) -> impl Filter<Extract = (WsManagerEventSender,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ch.clone())
}

fn with_engine_channel(
    ch: EngineBetSender,
) -> impl Filter<Extract = (EngineBetSender,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ch.clone())
}

// async fn with_signature_nickname<'a>(
//     credentials: json_requests::SetNickname,
// ) -> Result<json_requests::SetNickname, warp::Rejection> {
//     if tools::verify_signature(
//         &credentials.address,
//         &credentials.nickname,
//         &credentials.signature,
//     ) {
//         Ok(credentials)
//     } else {
//         Err(reject::custom(ApiError::BadSignature(
//             credentials.address.to_string(),
//             credentials.nickname.to_string(),
//             credentials.signature,
//         )))
//     }
// }

// async fn with_signature_referal<'a>(
//     credentials: json_requests::CreateReferal,
// ) -> Result<json_requests::CreateReferal, warp::Rejection> {
//     let msg = format!("{} {}", &credentials.refer_to, &credentials.referal);
//     if tools::verify_signature(&credentials.referal, &msg, &credentials.signature) {
//         Ok(credentials)
//     } else {
//         Err(reject::custom(ApiError::BadSignature(
//             credentials.referal.to_string(),
//             msg.to_string(),
//             credentials.signature,
//         )))
//     }
// }

// async fn with_signature_connect_wallet<'a>(
//     credentials: json_requests::ConnectWallet,
// ) -> Result<json_requests::ConnectWallet, warp::Rejection> {
//     let msg = format!(
//         "CONNECT WALLET {} {} {} {}",
//         &credentials.partner_wallet,
//         &credentials.user_wallet,
//         &credentials.site_id,
//         &credentials.sub_id,
//     );
//     if tools::verify_signature(&credentials.user_wallet, &msg, &credentials.signature) {
//         Ok(credentials)
//     } else {
//         Err(reject::custom(ApiError::BadSignature(
//             credentials.user_wallet.to_string(),
//             msg.to_string(),
//             credentials.signature,
//         )))
//     }
// }

fn extract_token(headers: &HeaderMap<HeaderValue>) -> Result<(String, Payload), ApiError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(h) => h,
        None => return Err(ApiError::NoAuthError),
    };
    let auth_header = match str::from_utf8(header.as_bytes()) {
        Ok(h) => h,
        Err(_) => return Err(ApiError::NoAuthError),
    };
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::InvalidAuthHeaderError);
    }
    let token = auth_header.trim_start_matches("Bearer ").to_owned();
    let parts = token.split('.').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(ApiError::MalformedToken);
    }
    let decoded = serde_json::from_str::<jwt::Payload>(
        str::from_utf8(
            &general_purpose::STANDARD_NO_PAD
                .decode(parts[1])
                .map_err(|_| ApiError::MalformedToken)?,
        )
        .map_err(|_| ApiError::MalformedToken)?,
    )
    .map_err(|_| ApiError::MalformedToken)?;
    Ok((
        auth_header.trim_start_matches("Bearer ").to_owned(),
        decoded,
    ))
}

async fn auth_verified(headers: HeaderMap<HeaderValue>, db: DB) -> Result<i64, warp::Rejection> {
    match extract_token(&headers) {
        Ok((token, decoded)) => {
            debug!("Token {:?}", decoded);
            let user = db
                .fetch_user(decoded.sub)
                .await
                .map_err(|e| reject::custom(ApiError::DbError(e)))?
                .ok_or(ApiError::ArbitraryError(
                    "Wrong username or password".into(),
                ))?;
            let _token_serialized = tools::serialize_token(
                &token,
                &format!("{}{}{}", *PASSWORD_SALT, user.password, decoded.iat),
            )
            .map_err(|_| reject::custom(ApiError::MalformedToken))?;

            Ok(user.id)
        }
        Err(e) => Err(reject::custom(e)),
    }
}

fn with_auth(db: DB) -> impl Filter<Extract = (i64,), Error = warp::Rejection> + Clone {
    headers_cloned()
        .map(|header| header)
        .and(with_db(db))
        .and_then(auth_verified)
}

async fn dex(headers: HeaderMap<HeaderValue>, _: DB) -> Result<bool, warp::Rejection> {
    headers
        .get("X-EX-APIKEY")
        .ok_or(ApiError::TheDexBadApiKey)?;
    Ok(true)
}

fn with_dex_response(db: DB) -> impl Filter<Extract = (bool,), Error = warp::Rejection> + Clone {
    headers_cloned()
        .map(|header| header)
        .and(with_db(db))
        .and_then(dex)
}

// fn json_body_set_nickname(
// ) -> impl Filter<Extract = (json_requests::SetNickname,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_subscribe_referal(
// ) -> impl Filter<Extract = (json_requests::CreateReferal,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

fn json_body_invoice_callback(
) -> impl Filter<Extract = (thedex::models::Invoice,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_register_user(
) -> impl Filter<Extract = (json_requests::RegisterUser,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_login_user(
) -> impl Filter<Extract = (json_requests::Login,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_change_username(
) -> impl Filter<Extract = (json_requests::ChangeNickname,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_create_invoice(
) -> impl Filter<Extract = (json_requests::CreateInvoice,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn get_all_coins(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(with_db(db))
        .and_then(handlers::get_all_coins)
}

pub fn coin(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("coin").and(get_all_coins(db.clone()))
}

pub fn get_all_last_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(with_db(db))
        .and_then(handlers::get_all_last_bets)
}

pub fn get_bets_for_game(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("game" / String)
        .and(with_db(db))
        .and_then(handlers::get_bets_for_game)
}

pub fn bets(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("bets").and(
        //get_player_bets(db.clone())
        //.or(get_game_bets(db.clone()))
        //.or(get_network_bets(db.clone()))
        get_all_last_bets(db.clone()).or(get_bets_for_game(db.clone())),
    )
}

// USER

pub fn register_user(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(json_body_register_user())
        //.and_then(with_signature_partner)
        .and(with_db(db))
        .and_then(handlers::register_user)
}

pub fn login_user(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body_login_user())
        .and(with_db(db))
        .and_then(handlers::login_user)
}

pub fn get_amounts(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("amounts" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_amounts)
}

pub fn change_username(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("username")
        .and(warp::patch())
        .and(json_body_change_username())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::change_username)
}

pub fn get_user(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_user)
}

pub fn get_logined_user(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!()
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_db(db))
        .and_then(handlers::get_user)
}

pub fn get_client_seed(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("client")
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::get_client_seed)
}

pub fn get_server_seed(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("server")
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::get_server_seed)
}

pub fn seed(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("seed").and(get_client_seed(db.clone()).or(get_server_seed(db)))
}

pub fn user(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("user").and(
        get_user(db.clone())
            .or(register_user(db.clone()))
            .or(login_user(db.clone()))
            .or(get_amounts(db.clone()).or(change_username(db.clone())))
            .or(seed(db.clone()))
            .or(get_logined_user(db)),
    )
}

pub fn create_invoice(
    db: DB,
    dex: TheDex,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("create")
        .and(warp::post())
        .and(json_body_create_invoice())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and(with_thedex(dex))
        .and_then(handlers::create_invoice)
}

pub fn generate_qr(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("qr" / String)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(handlers::generate_qr)
}

pub fn crypto_prices(
    db: DB,
    dex: TheDex,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("prices")
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_thedex(dex))
        .and_then(handlers::crypto_prices)
}

pub fn invoice_callback(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("callback")
        .and(warp::post())
        .and(with_dex_response(db.clone()))
        .and(json_body_invoice_callback())
        .and(with_db(db))
        .and_then(handlers::invoice_callback)
}

pub fn invoice(
    db: DB,
    dex: TheDex,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("invoice").and(
        create_invoice(db.clone(), dex.clone()).or(generate_qr(db.clone())
            .or(crypto_prices(db.clone(), dex))
            .or(invoice_callback(db))),
    )
}

pub fn list_games(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(with_db(db.clone()))
        .and_then(handlers::get_all_games)
}

pub fn game(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("game").and(list_games(db.clone()))
}

pub fn init_filters(
    db: DB,
    dex: TheDex, //bet_sender: WsDataFeedSender,
    manager_channel: WsManagerEventSender,
    engine_sender: EngineBetSender,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // network(db.clone())
    //     .or(rpc(db.clone()))
    //     .or(block_explorer(db.clone()))
    //     .or(token(db.clone()))
    //     .or(game(db.clone()))
    //     .or(player(db.clone()))
    //     .or(abi(db.clone()))
    //     .or(bets(db.clone()))
    //     .or(general(db.clone()))
    //     .or(partners(db.clone()))
    //     .or(warp::path!("updates")
    //         .and(warp::ws())
    //         .and(with_db(db))
    //         .and(with_channel(bet_sender))
    //         .map(|ws: warp::ws::Ws, db, ch| {
    //             ws.on_upgrade(move |socket| handlers::websockets_handler(socket, db, ch))
    //         }))
    user(db.clone())
        .or(invoice(db.clone(), dex))
        .or(bets(db.clone()))
        .or(game(db.clone()))
        .or(coin(db.clone()))
        .or(warp::path!("updates")
            .and(warp::ws())
            .and(with_db(db))
            .and(with_manager_channel(manager_channel.clone()))
            .and(with_engine_channel(engine_sender.clone()))
            .and(warp::header::header::<SocketAddr>("X-Forwarded-For"))
            .map(
                |ws: warp::ws::Ws,
                 db,
                 channel: WsManagerEventSender,
                 engine_channel: EngineBetSender,
                 addr| {
                    ws.on_upgrade(move |socket| {
                        handlers::websockets_handler(
                            socket,
                            addr,
                            db,
                            channel.clone(),
                            engine_channel.clone(),
                        )
                    })
                },
            ))
}

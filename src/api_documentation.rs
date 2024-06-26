#[allow(unused_imports)]
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
#[allow(unused_imports)]
use utoipa_swagger_ui::Config;

use crate::handlers;
use crate::models::{db_models, json_requests, json_responses, LeaderboardType};

use crate::oauth_providers;
use std::sync::Arc;
use warp::{
    http::Uri,
    hyper::{Response, StatusCode},
    path::FullPath,
    path::Tail,
    Rejection, Reply,
};

#[derive(OpenApi)]
#[openapi(
        paths(
            handlers::get_user,
            handlers::change_username,
            handlers::get_amounts,
            handlers::login_user,
            handlers::register_user,
            handlers::create_invoice,
            handlers::generate_qr,
            handlers::get_client_seed,
            handlers::get_server_seed,
            handlers::get_bets_for_game,
            handlers::get_all_last_bets,
            handlers::get_all_games,
            handlers::get_all_coins,
            handlers::crypto_prices,
            handlers::get_user_bets,
            handlers::get_user_bets_inc,
            handlers::get_latest_games,
            handlers::get_users_totals,
            handlers::create_p2way_token,
            handlers::p2way_callback,
            handlers::get_invoice,
            handlers::get_leaderboard,
            handlers::refresh_token,
            handlers::get_totals,
            handlers::register_referal_link,
            handlers::register_referal,
            handlers::change_password,
            handlers::login_google,
            handlers::billine_create_invoice,
            handlers::get_prom_tokens,
            handlers::create_payout_request
        ),
        components(schemas(
            //json_requests::User,
            json_requests::ByNetworkId,
            json_requests::RegisterPartner,
            json_requests::PartnerContactBasic,
            json_requests::AddPartnerContacts,
            json_requests::AddPartnerSite,
            json_requests::AddPartnerSubid,
            json_requests::ConnectWallet,
            json_requests::Login,
            json_requests::RegisterUser,
            json_requests::WithdrawRequest,
            json_requests::ChangePasswordRequest,
            json_requests::ChangeNickname,
            json_requests::SubmitQuestion,
            json_requests::QrRequest,
            json_requests::InvoiceAmount,
            json_requests::CreateInvoice,
            json_requests::CreateBillineInvoice,
            json_requests::PayoutRequest,

            json_responses::JsonResponse,
            json_responses::ResponseBody,
            json_responses::ErrorText,
            json_responses::InfoText,
            json_responses::AccessToken,
            json_responses::Seed,
            json_responses::UserStripped,
            json_responses::Games,
            json_responses::Coins,
            json_responses::LatestGames,
            json_responses::OneTimeToken,
            json_responses::BillineCreateInvoiceResponse,
            json_responses::PromTokens,

            db_models::User,
            db_models::Coin,
            db_models::Amount,
            db_models::Game,
            db_models::UserSeed,
            db_models::ServerSeed,
            db_models::Bet,
            db_models::Invoice,
            db_models::Totals,
            db_models::UserTotals,
            db_models::Leaderboard,
            db_models::TimeBoundaries,
            db_models::BillineInvoice,
            db_models::BillineInvoiceStatus,

            oauth_providers::google::CodeResponse,
            dexscreener::models::Token,
            dexscreener::models::OrderBook,
            dexscreener::models::Transactions,
            dexscreener::models::TimeChange,
            dexscreener::models::Liquidity,
            dexscreener::models::Pair,
            dexscreener::models::Pairs,

            LeaderboardType

        )),
        tags(
            (name = "Core REST API", description = "Core REST API")
        )
    )]
pub struct ApiDoc;

pub async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/swagger-ui/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}

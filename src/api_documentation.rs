#[allow(unused_imports)]
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
#[allow(unused_imports)]
use utoipa_swagger_ui::Config;

use crate::handlers;
use crate::models::{db_models, json_requests, json_responses, LeaderboardType};

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
            handlers::get_server_seed
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
            json_requests::WithdrawRequest,
            json_requests::ChangePasswordRequest,
            json_requests::SubmitQuestion,
            json_requests::QrRequest,
            json_requests::InvoiceAmount,
            json_requests::CreateInvoice,

            json_responses::JsonResponse,
            json_responses::ResponseBody,
            json_responses::ErrorText,
            json_responses::InfoText,
            json_responses::AccessToken,
            json_responses::Seed,

            db_models::User,
            db_models::Coin,
            db_models::Amount,
            db_models::Game,
            db_models::UserSeed,
            db_models::ServerSeed,
            db_models::Bet,
            db_models::Invoice,

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

use crate::db::DB;
use crate::errors::ApiError;
use crate::models::json_requests::{self, CreateInvoice, Login};

use crate::communication::EnginePropagatedBet;
use crate::models::db_models::UserTotals;
use crate::models::json_responses::{ErrorText, InfoText, JsonResponse, ResponseBody, Status};
mod bets;
pub use bets::*;
mod coin;
pub use coin::*;
mod game;
pub use game::*;
mod general;
pub use general::*;
mod invoice;
pub use invoice::*;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
mod user;
pub use user::*;
mod partners;
pub use partners::*;
use warp::http::StatusCode;
use warp::Reply;
use warp::{http::Response as HttpResponse, reject, reply::Response as WarpResponse};

fn get_response_status_json<T: Serialize>(status_code: StatusCode, message: T) -> impl warp::Reply {
    warp::reply::with_status(warp::reply::json(&message), status_code)
}

fn get_pgn_response(image: Vec<u8>) -> WarpResponse {
    HttpResponse::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .body(image)
        .unwrap()
        .into_response()
}

pub fn gen_info_response(info: &str) -> WarpResponse {
    get_response_status_json(
        StatusCode::OK,
        JsonResponse {
            status: Status::OK,
            body: ResponseBody::InfoText(InfoText {
                message: info.into(),
            }),
        },
    )
    .into_response()
}

pub fn gen_raw_text_response(info: &'static str) -> WarpResponse {
    HttpResponse::builder()
        .status(200)
        .body(info)
        .unwrap()
        .into_response()
}

pub fn gen_arbitrary_response(info: ResponseBody) -> WarpResponse {
    get_response_status_json(
        StatusCode::OK,
        JsonResponse {
            status: Status::OK,
            body: info,
        },
    )
    .into_response()
}

pub fn gen_redirect_response(info: ResponseBody, location: &str) -> WarpResponse {
    HttpResponse::builder()
        .status(302)
        .header("Content-Type", "application/json")
        .header("location", location)
        .body(serde_json::to_string(&info).unwrap())
        .unwrap()
        .into_response()
}

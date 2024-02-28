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
use futures::FutureExt;
use futures::StreamExt;
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

// fn json_body_set_nickname(
// ) -> impl Filter<Extract = (json_requests::SetNickname,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_subscribe_referal(
// ) -> impl Filter<Extract = (json_requests::CreateReferal,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

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

fn json_body_generate_qr_code(
) -> impl Filter<Extract = (json_requests::QrRequest,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

// fn json_body_add_partner_contacts(
// ) -> impl Filter<Extract = (json_requests::AddPartnerContacts,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_add_partner_site(
// ) -> impl Filter<Extract = (json_requests::AddPartnerSite,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_add_partner_subid(
// ) -> impl Filter<Extract = (json_requests::AddPartnerSubid,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_delete_partner_contact(
// ) -> impl Filter<Extract = (json_requests::DeletePartnerContacts,), Error = warp::Rejection> + Clone
// {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_connect_wallet(
// ) -> impl Filter<Extract = (json_requests::ConnectWallet,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_submit_error(
// ) -> impl Filter<Extract = (json_requests::SubmitError,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_submit_withdrawal(
// ) -> impl Filter<Extract = (json_requests::WithdrawRequest,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_change_password(
// ) -> impl Filter<Extract = (json_requests::ChangePasswordRequest,), Error = warp::Rejection> + Clone
// {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// fn json_body_submit_question(
// ) -> impl Filter<Extract = (json_requests::SubmitQuestion,), Error = warp::Rejection> + Clone {
//     warp::body::content_length_limit(1024 * 16).and(warp::body::json())
// }

// // NETWORKS
// pub fn get_networks(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("list")
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_networks)
// }

// pub fn network(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("network").and(get_networks(db))
// }

// // RPCS
// pub fn get_rpcs(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / i64)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_rpcs)
// }

// pub fn rpc(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("rpc").and(get_rpcs(db))
// }

// // EXPLORERS
// pub fn get_all_explorers(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("list")
//         .and(with_db(db))
//         .and_then(handlers::get_all_explorers)
// }

// pub fn get_block_explorers(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / i64)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_block_explorers)
// }

// pub fn block_explorer(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("block_epxlorer").and(get_block_explorers(db.clone()).or(get_all_explorers(db)))
// }

// // TOKENS
// pub fn get_token_price(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("price" / String)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_token_price)
// }
// pub fn get_tokens(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / i64)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_tokens)
// }

// pub fn token(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("token").and(get_tokens(db.clone()).or(get_token_price(db)))
// }

// // GAMES
// pub fn get_game(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / i64 / String)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_game)
// }

// pub fn get_game_by_id(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / i64)
//         .and(with_db(db))
//         .and_then(handlers::get_game_by_id)
// }

// pub fn game(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("game").and(get_game(db.clone()).or(get_game_by_id(db)))
// }

// // PLAYER
// pub fn get_nickname(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / String)
//         .and(warp::get())
//         .and(with_db(db))
//         .and_then(handlers::get_nickname)
// }

// pub fn set_nickname(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("set")
//         .and(warp::post())
//         .and(json_body_set_nickname())
//         .and_then(with_signature_nickname)
//         .and(with_db(db))
//         .and_then(handlers::set_nickname)
// }

// pub fn get_player(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_player)
// }

// pub fn get_latest_games(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("latest_games" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_latest_games)
// }

// pub fn get_player_totals(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("totals" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_player_totals)
// }

// pub fn create_referal(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("subscribe")
//         .and(warp::post())
//         .and(json_body_subscribe_referal())
//         .and_then(with_signature_referal)
//         .and(with_db(db))
//         .and_then(handlers::player::create_referal)
// }

// pub fn player(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("player").and(
//         get_player(db.clone())
//             .or(warp::path("nickname").and(get_nickname(db.clone()).or(set_nickname(db.clone()))))
//             .or(get_latest_games(db.clone()))
//             .or(get_player_totals(db.clone()))
//             .or(warp::path("referal").and(create_referal(db))),
//     )
// }

// // ABI
// pub fn get_abi(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_abi)
// }

// pub fn abi(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("abi").and(get_abi(db))
// }

// // BETS
// pub fn get_player_bets(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("player" / String / ..)
//         .and(
//             warp::path::param::<i64>()
//                 .map(Some)
//                 .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
//         )
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_player_bets)
// }

// pub fn get_player_bets_inc(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("player" / "inc" / String / ..)
//         .and(
//             warp::path::param::<i64>()
//                 .map(Some)
//                 .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
//         )
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_player_bets_inc)
// }

// pub fn get_game_bets(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("game" / i64)
//         .and(with_db(db))
//         .and_then(handlers::get_game_bets)
// }

// pub fn get_network_bets(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("network" / i64)
//         .and(with_db(db))
//         .and_then(handlers::get_network_bets)
// }

// pub fn get_all_last_bets(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("list")
//         .and(with_db(db))
//         .and_then(handlers::get_all_last_bets)
// }

// pub fn get_bets_for_game(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("game" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_bets_for_game)
// }

// pub fn bets(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("bets").and(
//         get_player_bets(db.clone())
//             .or(get_game_bets(db.clone()))
//             .or(get_network_bets(db.clone()))
//             .or(get_all_last_bets(db.clone()))
//             .or(get_bets_for_game(db.clone()).or(get_player_bets_inc(db))),
//     )
// }

// // PARTNERS REFERALS
// pub fn submit_question(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("question")
//         .and(warp::post())
//         .and(json_body_submit_question())
//         .and(with_db(db))
//         .and_then(handlers::submit_question)
// }
// pub fn register_partner(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("register")
//         .and(warp::post())
//         .and(json_body_register_partner())
//         //.and_then(with_signature_partner)
//         .and(with_db(db))
//         .and_then(handlers::register_partner)
// }

// pub fn login_partner(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("login")
//         .and(warp::post())
//         .and(json_body_login_partner())
//         .and(with_db(db))
//         .and_then(handlers::login_partner)
// }

// pub fn get_partner(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         // .and(warp::header::<String>("auth"))
//         // .and(warp::header::<u64>("timestamp"))
//         // .and(warp::header::<String>("wallet"))
//         //.and(with_db(db.clone()))
//         //.and_then(with_auth_partner)
//         .and(with_auth(db.clone()))
//         .and(with_db(db))
//         .and_then(handlers::get_partner)
// }

// pub fn add_partner_contacts(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("add")
//         .and(warp::post())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(json_body_add_partner_contacts())
//         .and(with_db(db))
//         .and_then(handlers::add_contacts)
// }

// pub fn add_partner_site(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("add")
//         .and(warp::post())
//         .and(with_auth(db.clone()))
//         .and(json_body_add_partner_site())
//         .and(with_db(db))
//         .and_then(handlers::add_partner_site)
// }

// pub fn get_partner_sites(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(with_db(db))
//         .and_then(handlers::get_partner_sites)
// }

// pub fn add_partner_subid(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("add")
//         .and(warp::post())
//         .and(with_auth(db.clone()))
//         .and(json_body_add_partner_subid())
//         .and(with_db(db))
//         .and_then(handlers::add_partner_subid)
// }

// pub fn click_partner_subid(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("click" / String / i64 / i64)
//         .and(warp::post())
//         .and(with_db(db))
//         .and_then(handlers::click_partner_subid)
// }

// pub fn subid_get_clicks(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("clicks")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<i64>())
//         .and(warp::path::param::<i64>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_clicks)
// }

// pub fn site_get_clicks(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("clicks")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<i64>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_site_clicks)
// }

// pub fn partner_get_clicks(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("clicks")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(with_db(db))
//         .and_then(handlers::get_partner_clicks)
// }

// pub fn get_partner_clicks_exact_date(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("clicks")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_clicks_exact_date)
// }

// pub fn connect_wallet_subid(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("connect")
//         .and(warp::post())
//         .and(json_body_connect_wallet())
//         .and_then(with_signature_connect_wallet)
//         .and(with_db(db))
//         .and_then(handlers::connect_wallet)
// }

// pub fn get_partner_contacts(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("get")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(with_db(db))
//         .and_then(handlers::get_partner_contacts)
// }

// pub fn delete_partner_contacts(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("delete")
//         .and(warp::post())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(json_body_delete_partner_contact())
//         .and(with_db(db))
//         .and_then(handlers::delete_partner_contacts)
// }

// pub fn get_partner_connected_wallets_with_deposits_amount(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("connected_betted")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<TimeBoundaries>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_connected_wallets_with_deposits_amount)
// }

// pub fn get_partner_connected_wallets(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("connected")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<TimeBoundaries>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_connected_wallets)
// }

// pub fn get_partner_connected_wallets_exact_date(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("connected")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_connected_wallets_exact_date)
// }

// pub fn get_partner_connected_wallets_betted_exact_date(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("connected_betted")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::param::<u64>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_connected_wallets_betted_exact_date)
// }

// pub fn get_conected_totals(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("connected" / "totals")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(with_db(db))
//         .and_then(handlers::get_connected_totals)
// }

// pub fn get_partner_connected_wallets_info(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("wallets")
//         .and(warp::get())
//         //.and(json_body_register_partner())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<TimeBoundaries>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_partner_connected_wallets_info)
// }

// pub fn submit_partner_withdraw_request(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("withdraw")
//         .and(warp::post())
//         .and(with_auth(db.clone()))
//         .and(json_body_submit_withdrawal())
//         .and(with_db(db))
//         .and_then(handlers::submit_withdrawal)
// }

// pub fn partner_change_password(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("password")
//         .and(warp::put())
//         .and(with_auth(db.clone()))
//         .and(json_body_change_password())
//         .and(with_db(db))
//         .and_then(handlers::partner_change_password)
// }

// pub fn partner_get_withdrawals(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("withdrawals")
//         .and(warp::get())
//         .and(with_auth(db.clone()))
//         .and(warp::path::param::<TimeBoundaries>())
//         .and(warp::path::end())
//         .and(with_db(db))
//         .and_then(handlers::get_withdrawal_requests)
// }

// pub fn partner_change(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("change").and(partner_change_password(db.clone()))
// }

// pub fn partners(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("partner").and(
//         register_partner(db.clone())
//             .or(partner_get_withdrawals(db.clone()))
//             .or(submit_question(db.clone()))
//             .or(partner_change(db.clone()))
//             .or(submit_partner_withdraw_request(db.clone()))
//             .or(get_conected_totals(db.clone()))
//             .or(login_partner(db.clone()))
//             .or(get_partner_connected_wallets_info(db.clone()))
//             .or(get_partner_connected_wallets(db.clone()))
//             .or(get_partner_connected_wallets_exact_date(db.clone()))
//             .or(get_partner_connected_wallets_betted_exact_date(db.clone()))
//             .or(get_partner_connected_wallets_with_deposits_amount(
//                 db.clone(),
//             ))
//             .or(get_partner_clicks_exact_date(db.clone()))
//             .or(partner_get_clicks(db.clone()))
//             .or(get_partner(db.clone()))
//             .or(warp::path("contacts").and(
//                 get_partner_contacts(db.clone())
//                     .or(add_partner_contacts(db.clone()))
//                     .or(delete_partner_contacts(db.clone())),
//             ))
//             .or(warp::path("site").and(
//                 add_partner_site(db.clone())
//                     .or(site_get_clicks(db.clone()))
//                     .or(get_partner_sites(db.clone()))
//                     .or(warp::path("subid").and(
//                         add_partner_subid(db.clone())
//                             .or(click_partner_subid(db.clone()))
//                             .or(connect_wallet_subid(db.clone()))
//                             .or(subid_get_clicks(db.clone())),
//                     )),
//             )),
//     )
// }

// // GENERAL
// pub fn get_totals(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("totals")
//         .and(with_db(db))
//         .and_then(handlers::get_totals)
// }

// pub fn get_leaderboard(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("leaderboard" / LeaderboardType / TimeBoundaries)
//         .and(with_db(db))
//         .and_then(handlers::get_leaderboard)
// }

// pub fn submit_error(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path!("error")
//         .and(warp::post())
//         .and(json_body_submit_error())
//         .and(with_db(db))
//         .and_then(handlers::submit_error)
// }

// pub fn general(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     warp::path("general").and(
//         get_totals(db.clone())
//             .or(submit_error(db.clone()))
//             .or(get_leaderboard(db)),
//     )
// }

// // pub fn get_full_game(
// //     db: DB,
// // ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone{
// //     warp::path!("get_full_game" / String)
// //         .and(with_db(db))
// //         .and_then(handlers::get_full_game)
// // }

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
            .or(seed(db)),
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
    warp::path!("qr")
        .and(warp::get())
        .and(json_body_generate_qr_code())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::generate_qr)
}

pub fn invoice(
    db: DB,
    dex: TheDex,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("invoice").and(create_invoice(db.clone(), dex).or(generate_qr(db)))
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

use crate::config;
use crate::config::PASSWORD_SALT;
use crate::db::DB;
use crate::errors::ApiError;
use crate::handlers;
use crate::jwt;
use crate::jwt::Payload;
use crate::models::db_models::TimeBoundaries;
use crate::models::json_requests;
use crate::models::LeaderboardType;
use crate::oauth_providers;
use crate::tools;
use crate::EngineBetSender;

use crate::WsManagerEventSender;
use base64::{engine::general_purpose, Engine as _};
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use p2way::P2Way;
use std::net::SocketAddr;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use thedex::TheDex;
use tracing::debug;
use warp::filters::header::headers_cloned;
use warp::reject;

use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_google(
    google: oauth_providers::google::GoogleOauth,
) -> impl Filter<Extract = (oauth_providers::google::GoogleOauth,), Error = std::convert::Infallible>
       + Clone {
    warp::any().map(move || google.clone())
}

fn with_thedex(
    dex: TheDex,
) -> impl Filter<Extract = (TheDex,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || dex.clone())
}

fn with_dexscreener(
    dex: dexscreener::DexScreener,
) -> impl Filter<Extract = (dexscreener::DexScreener,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || dex.clone())
}

fn with_p2way(
    p2way: P2Way,
) -> impl Filter<Extract = (P2Way,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || p2way.clone())
}

fn with_hcap(
    hcap: hcaptcha::HCaptcha,
) -> impl Filter<Extract = (hcaptcha::HCaptcha,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || hcap.clone())
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

    let start = SystemTime::now();
    let current_time = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // TODO: add this back, frontend team asked to remove it for some time
    //if !decoded.aud.eq("Auth") || decoded.exp < current_time {
    if !decoded.aud.eq("Auth") {
        return Err(ApiError::MalformedToken);
    }

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
            let key = if decoded.iss.eq("Local") {
                format!("{}{}{}", *PASSWORD_SALT, user.password, decoded.iat)
            } else {
                format!("{}{}", *PASSWORD_SALT, decoded.iat)
            };
            let _token_serialized = tools::serialize_token(&token, &key)
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
    debug!("headers {:?}", headers);
    let rec_api_key = headers
        .get("X-EX-APIKEY")
        .ok_or(ApiError::TheDexBadApiKey)?
        .to_str()
        .map_err(|_| ApiError::TheDexBadApiKey)?;
    if !rec_api_key.eq(&(*config::X_EX_APIKEY)) {
        return Err(ApiError::TheDexBadApiKey.into());
    }
    Ok(true)
}

fn with_dex_response(db: DB) -> impl Filter<Extract = (bool,), Error = warp::Rejection> + Clone {
    headers_cloned()
        .map(|header| header)
        .and(with_db(db))
        .and_then(dex)
}

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

fn json_body_change_password(
) -> impl Filter<Extract = (json_requests::ChangePasswordRequest,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_create_invoice(
) -> impl Filter<Extract = (json_requests::CreateInvoice,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_create_billine_invoice(
) -> impl Filter<Extract = (json_requests::CreateBillineInvoice,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_billine_callback(
) -> impl Filter<Extract = (billine::CallbackIframe,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_p2way_callback(
) -> impl Filter<Extract = (p2way::models::CallbackResponse,), Error = warp::Rejection> + Clone {
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

// BETS

pub fn get_user_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / i64 / ..)
        .and(
            warp::path::param::<i64>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(handlers::get_user_bets)
}

pub fn get_user_bets_inc(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "inc" / i64 / ..)
        .and(
            warp::path::param::<i64>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(handlers::get_user_bets_inc)
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
        get_all_last_bets(db.clone())
            .or(get_bets_for_game(db.clone()))
            .or(get_user_bets(db.clone()))
            .or(get_user_bets_inc(db)),
    )
}

// USER
pub fn register_user(
    db: DB,
    hcap: hcaptcha::HCaptcha,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(json_body_register_user())
        //.and_then(with_signature_partner)
        .and(with_db(db))
        .and(with_hcap(hcap))
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

pub fn login_user_google(
    db: DB,
    google: oauth_providers::google::GoogleOauth,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login" / "google")
        .and(warp::query::<oauth_providers::google::CodeResponse>())
        .and(warp::get())
        .and(with_db(db))
        .and(with_google(google))
        .and_then(handlers::login_google)
}

pub fn refresh_token(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("refresh" / String)
        .and(warp::post())
        .and(with_db(db))
        .and_then(handlers::refresh_token)
}

pub fn get_amounts(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("amounts" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_amounts)
}

pub fn get_latest_games(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("latest_games" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_latest_games)
}

pub fn get_user_totals(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("totals" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_users_totals)
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

pub fn change_password(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("password")
        .and(warp::patch())
        .and(json_body_change_password())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::change_password)
}

pub fn register_referal_link(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register_ref" / String)
        .and(warp::post())
        .and(with_auth(db.clone()))
        .and(with_db(db))
        .and_then(handlers::register_referal_link)
}

pub fn register_referal(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("referal" / String)
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_db(db))
        .and_then(handlers::register_referal)
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

pub fn user(
    db: DB,
    hcap: hcaptcha::HCaptcha,
    google: oauth_providers::google::GoogleOauth,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("user").and(
        get_user(db.clone())
            .or(register_user(db.clone(), hcap.clone()))
            .or(login_user(db.clone()))
            .or(get_amounts(db.clone())
                .or(change_username(db.clone()))
                .or(change_password(db.clone())))
            .or(seed(db.clone()))
            .or(get_logined_user(db.clone()))
            .or(get_user_totals(db.clone()))
            .or(refresh_token(db.clone()))
            .or(login_user_google(db.clone(), google))
            .or(register_referal_link(db.clone()))
            .or(register_referal(db.clone()))
            .or(get_latest_games(db)),
    )
}

pub fn create_one_time_token(
    db: DB,
    p2way: P2Way,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("ott")
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_p2way(p2way))
        .and_then(handlers::create_p2way_token)
}

pub fn p2way_callback(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("callback")
        .and(warp::post())
        .and(json_body_p2way_callback())
        .and(with_db(db))
        .and(warp::header::header::<SocketAddr>("X-Forwarded-For"))
        .and_then(handlers::p2way_callback)
}
pub fn p2way_filter(
    db: DB,
    p2way: P2Way,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("p2way").and(create_one_time_token(db.clone(), p2way).or(p2way_callback(db)))
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

// TODO: Unite functions under same route "billine"
pub fn create_billine_invoice(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("billine" / "create")
        .and(warp::post())
        .and(json_body_create_billine_invoice())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and(warp::header::header::<SocketAddr>("X-Forwarded-For"))
        .and_then(handlers::billine_create_invoice)
}
//invoice_billine_callback
pub fn billine_invoice_callback(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("billine" / "callback")
        .and(warp::post())
        .and(json_body_billine_callback())
        .and(with_db(db.clone()))
        .and_then(handlers::invoice_billine_callback)
}

pub fn generate_qr(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("qr" / String)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(handlers::generate_qr)
}

pub fn get_invoice(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(String)
        .and(warp::get())
        .and(with_auth(db.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::get_invoice)
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
    ch: WsManagerEventSender,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("callback")
        .and(warp::post())
        .and(with_dex_response(db.clone()))
        .and(json_body_invoice_callback())
        .and(with_db(db))
        .and(with_manager_channel(ch))
        .and_then(handlers::invoice_callback)
}

pub fn invoice(
    db: DB,
    dex: TheDex,
    ch: WsManagerEventSender,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("invoice").and(
        create_invoice(db.clone(), dex.clone())
            .or(generate_qr(db.clone()))
            .or(crypto_prices(db.clone(), dex))
            .or(invoice_callback(db.clone(), ch))
            .or(get_invoice(db.clone()))
            .or(create_billine_invoice(db.clone()))
            .or(billine_invoice_callback(db)),
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

pub fn get_totals(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("totals")
        .and(with_db(db.clone()))
        .and_then(handlers::get_totals)
}

pub fn get_prom_tokens(
    dexs: dexscreener::DexScreener,
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("promtokens")
        .and(with_dexscreener(dexs.clone()))
        .and(with_db(db.clone()))
        .and_then(handlers::get_prom_tokens)
}

pub fn get_leaderboard(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("leaderboard" / LeaderboardType / TimeBoundaries)
        .and(with_db(db))
        .and_then(handlers::get_leaderboard)
}

pub fn general(
    dexs: dexscreener::DexScreener,
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("general").and(
        get_totals(db.clone())
            .or(get_leaderboard(db.clone()))
            .or(get_prom_tokens(dexs, db)),
    )
}

pub fn init_filters(
    db: DB,
    dex: TheDex,
    p2way: P2Way,
    manager_channel: WsManagerEventSender,
    engine_sender: EngineBetSender,
    hcap: hcaptcha::HCaptcha,
    google: oauth_providers::google::GoogleOauth,
    dexs: dexscreener::DexScreener,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    user(db.clone(), hcap, google)
        .or(invoice(db.clone(), dex, manager_channel.clone()))
        .or(bets(db.clone()))
        .or(game(db.clone()))
        .or(coin(db.clone()))
        .or(general(dexs, db.clone()))
        .or(p2way_filter(db.clone(), p2way))
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

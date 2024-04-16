use crate::jwt;
use crate::models::db_models::OauthProvider;
use crate::models::json_responses::{Amounts, LatestGames, Seed, UserStripped};
use crate::tools::blake_hash;
use crate::{config::PASSWORD_SALT, models::json_responses::AccessToken};
use base64::{engine::general_purpose, Engine as _};
use blake2::{Blake2b512, Digest};

use hex::ToHex;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tracing::{debug, error};

use self::json_requests::{ChangeNickname, ChangePasswordRequest};
use crate::tools;
use std::str;

use super::*;
use crate::oauth_providers;

/// Register new user account
///
/// Registers new user account
#[utoipa::path(
        tag="user",
        post,
        path = "/api/user/register",
        request_body = RegisterUser,
        responses(
            (status = 200, description = "User account was created", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn register_user(
    data: json_requests::RegisterUser,
    db: DB,
    hcap: hcaptcha::HCaptcha,
) -> Result<WarpResponse, warp::Rejection> {
    let captcha_response = hcap
        .verify(data.h_captcha_response)
        .await
        .map_err(|e| reject::custom(ApiError::HCaptchaError(e)))?;

    if !captcha_response.success {
        return Err(reject::custom(ApiError::BadCaptcha));
    }

    let mut hasher = Blake2b512::new();

    hasher.update(data.password.as_bytes());

    let res: String = hasher.finalize().encode_hex();

    debug!("res {:?}", res);
    let user = db
        .register_user(&data.username, &data.username, OauthProvider::Local, &res)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    let coins = db
        .fetch_coins()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    debug!("Coins {:?}", coins);

    for coin in coins {
        if coin.id == 1 {
            db.init_amount(user.id, coin.id, Decimal::from_u64(1000).unwrap())
                .await
                .map_err(|e| reject::custom(ApiError::DbError(e)))?;
        } else {
            db.init_amount(user.id, coin.id, Decimal::from_u64(0).unwrap())
                .await
                .map_err(|e| reject::custom(ApiError::DbError(e)))?;
        }
    }

    Ok(gen_info_response("User account has been created"))
}

/// Register referal link
///
/// Registers new referal link for the loged in user
#[utoipa::path(
        tag="user",
        post,
        path = "/api/user/register_ref/{link_name}",
        responses(
            (status = 200, description = "Link has been registered", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("link_name" = String, Path, description = "Name for the referal link")
        )

    )]
pub async fn register_referal_link(
    link_name: String,
    referal: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.create_referal_link(referal, &link_name)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Link has been registered"))
}

/// Register referal
///
/// Registers new referal for the referal link
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/referal/{link_name}",
        responses(
            (status = 200, description = "Link has been registered", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("link_name" = String, Path, description = "Name for the referal link")
        )

    )]
pub async fn register_referal(
    link_name: String,
    referal: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let referal_link = db
        .fetch_referal_link(&link_name)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    db.new_referal(referal_link.refer_to, referal)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Referal as been registered"))
}

/// Login via google/register
///
/// Logins a user via google, if the account didn't exist, creates a new one
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/login/google",
        responses(
            (status = 200, description = "An access token", body = AccessToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),

    )]
pub async fn login_google(
    query: oauth_providers::google::CodeResponse,
    db: DB,
    google: oauth_providers::google::GoogleOauth,
) -> Result<WarpResponse, warp::Rejection> {
    let code = if let Some(code) = query.code {
        code
    } else {
        error!("Error on logging in with google: {:?}", query);
        return Ok(gen_info_response("Error logging in with google"));
    };

    let token = google.request_token(&code).await?;
    let google_user = google
        .get_google_user(&token.access_token, &token.id_token)
        .await?;

    let user = db
        .fetch_user_by_login(&google_user.email)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    let user_id = match user {
        Some(user) => {
            debug!("Google user found {:?}", user);
            if user.provider != OauthProvider::Google {
                error!(
                    "User with email: `{}` has provider: `{:?}` instead of google",
                    &google_user.email, user.provider
                );

                return Ok(gen_info_response(
                    "Error user was registered under different provider",
                ));
            }
            user.id
        }
        None => {
            let user = db
                .register_user(
                    &google_user.email,
                    &google_user.name,
                    OauthProvider::Google,
                    "",
                )
                .await
                .map_err(|e| reject::custom(ApiError::DbError(e)))?;

            let coins = db
                .fetch_coins()
                .await
                .map_err(|e| reject::custom(ApiError::DbError(e)))?;

            debug!("Coins {:?}", coins);

            for coin in coins {
                if coin.id == 1 {
                    db.init_amount(user.id, coin.id, Decimal::from_u64(1000).unwrap())
                        .await
                        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
                } else {
                    db.init_amount(user.id, coin.id, Decimal::from_u64(0).unwrap())
                        .await
                        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
                }
            }

            user.id
        }
    };

    let start = SystemTime::now();
    let iat = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let key = format!("{}{}", *PASSWORD_SALT, iat);

    let token = jwt::generate_token(
        &jwt::Payload {
            iss: "Google".into(),
            sub: user_id,
            exp: iat + 600,
            iat,
            aud: "Auth".into(),
        },
        &key,
    );

    let refresh_token = jwt::generate_token(
        &jwt::Payload {
            iss: "Google".into(),
            sub: user_id,
            exp: iat + 3000,
            iat,
            aud: "Refresh".into(),
        },
        &key,
    );

    db.new_refresh_token(user_id, &refresh_token)
        .await
        .map_err(ApiError::DbError)?;

    Ok(gen_redirect_response(
        ResponseBody::AccessToken(AccessToken {
            access_token: token.clone(),
            token_type: "Bearer".into(),
            expires_in: 600,
            refresh_token,
        }),
        "https://rew.greekkeepers.io/",
    ))
}

/// Refresh Token
///
/// Refreshes the auth token, removed used refresh token
#[utoipa::path(
        tag="user",
        post,
        path = "/api/user/refresh/{refresh_token}",
        responses(
            (status = 200, description = "Access token", body = AccessToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("refresh_token" = String, Path, description = "Refresh token")
        )

    )]
pub async fn refresh_token(token: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let parts = token.split('.').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(reject::custom(ApiError::MalformedToken));
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

    let user = db
        .fetch_user(decoded.sub)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(ApiError::ArbitraryError("Malformed token".into()))?;

    let key = if decoded.iss.eq("Local") {
        format!("{}{}{}", *PASSWORD_SALT, user.password, decoded.iat)
    } else {
        format!("{}{}", *PASSWORD_SALT, decoded.iat)
    };

    let _token_serialized = tools::serialize_token(&token, &key)
        .map_err(|_| reject::custom(ApiError::MalformedToken))?;

    let start = SystemTime::now();
    let current_time = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if !decoded.aud.eq("Refresh") || current_time > decoded.exp {
        return Err(reject::custom(ApiError::MalformedToken));
    }

    if !db
        .remove_refresh_token(&token, user.id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
    {
        return Err(reject::custom(ApiError::MalformedToken));
    }

    let token = jwt::generate_token(
        &jwt::Payload {
            iss: "Local".into(),
            sub: user.id,
            exp: current_time + 600,
            iat: current_time,
            aud: "Auth".into(),
        },
        &key,
    );

    let refresh_token = jwt::generate_token(
        &jwt::Payload {
            iss: "Local".into(),
            sub: user.id,
            exp: current_time + 3000,
            iat: current_time,
            aud: "Refresh".into(),
        },
        &key,
    );

    db.new_refresh_token(user.id, &refresh_token)
        .await
        .map_err(ApiError::DbError)?;

    Ok(gen_arbitrary_response(ResponseBody::AccessToken(
        AccessToken {
            access_token: token.clone(),
            token_type: "Bearer".into(),
            expires_in: 600,
            refresh_token,
        },
    )))
}

/// Login user
///
/// Logins user with provided login/password
#[utoipa::path(
        tag="user",
        post,
        path = "/api/user/login",
        request_body = Login,
        responses(
            (status = 200, description = "Access token", body = AccessToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn login_user(login: Login, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let hashed_password = blake_hash(&login.password);
    let user = db
        .login_user(&login.login, &hashed_password)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(reject::custom(ApiError::WrongLoginPassword))?;

    let start = SystemTime::now();
    let iat = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let key = format!("{}{}{}", *PASSWORD_SALT, hashed_password, iat);

    let token = jwt::generate_token(
        &jwt::Payload {
            iss: "Local".into(),
            sub: user.id,
            exp: iat + 600,
            iat,
            aud: "Auth".into(),
        },
        &key,
    );

    let refresh_token = jwt::generate_token(
        &jwt::Payload {
            iss: "Local".into(),
            sub: user.id,
            exp: iat + 3000,
            iat,
            aud: "Refresh".into(),
        },
        &key,
    );

    db.new_refresh_token(user.id, &refresh_token)
        .await
        .map_err(ApiError::DbError)?;

    Ok(gen_arbitrary_response(ResponseBody::AccessToken(
        AccessToken {
            access_token: token.clone(),
            token_type: "Bearer".into(),
            expires_in: 600,
            refresh_token,
        },
    )))
}

/// Get user coins amounts
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/amounts/{user_id}",
        responses(
            (status = 200, description = "Coins amounts", body = Amounts),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("user_id" = i64, Path, description = "User id")
        )
    )]
pub async fn get_amounts(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let amounts = db
        .fetch_amounts(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Amounts(Amounts {
        amounts,
    })))
}

/// Get user's latest games
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/latest_games/{user_id}",
        responses(
            (status = 200, description = "Latest Games", body = LatestGames),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("user_id" = i64, Path, description = "User id")
        )
    )]
pub async fn get_latest_games(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let games = db
        .latest_games(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::LatestGames(
        LatestGames { games },
    )))
}

/// Get user's totals
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/totals/{user_id}",
        responses(
            (status = 200, description = "User totals", body = UserTotals),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("user_id" = i64, Path, description = "User id")
        )
    )]
pub async fn get_users_totals(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let totals = db
        .fetch_user_totals(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::UserTotals(totals)))
}

/// Change user's username
///
/// requires user being logined
#[utoipa::path(
        tag="user",
        patch,
        path = "/api/user/username",
        request_body = ChangeNickname,
        responses(
            (status = 200, description = "Username was changed", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn change_username(
    data: ChangeNickname,
    id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.change_username(id, &data.nickname)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Username was changed"))
}

/// Change user's password
///
/// requires user being logined
#[utoipa::path(
        tag="user",
        patch,
        path = "/api/user/password",
        request_body = ChangePasswordRequest,
        responses(
            (status = 200, description = "Password was changed", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn change_password(
    data: ChangePasswordRequest,
    id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let mut hasher = Blake2b512::new();

    hasher.update(data.old_password.as_bytes());
    let old_password_hash: String = hasher.finalize().encode_hex();

    let user = db
        .fetch_user(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(reject::custom(ApiError::UserDoesntExist))?;

    if !user.password.eq(&old_password_hash) {
        return Err(reject::custom(ApiError::BadPassword));
    }

    let mut hasher = Blake2b512::new();

    hasher.update(data.new_password.as_bytes());

    let res: String = hasher.finalize().encode_hex();
    db.change_password(id, &res)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Password was changed"))
}

/// Get user info
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/{user_id}",
        responses(
            (status = 200, description = "User Info", body = UserStripped),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("user_id" = i64, Path, description = "User id")
        )
    )]
pub async fn get_user(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let user = db
        .fetch_user(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .map(|u| UserStripped {
            id: u.id,
            registration_time: u.registration_time,
            username: u.username,
        })
        .ok_or(ApiError::UserDoesntExist)?;

    Ok(gen_arbitrary_response(ResponseBody::User(user)))
}

/// Get user client seed
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/seed/client",
        responses(
            (status = 200, description = "Client seed", body = Seed),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_client_seed(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let seed = db
        .fetch_current_user_seed(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::ClientSeed(Seed {
        seed: seed.user_seed,
    })))
}

/// Get user server seed
///
///
#[utoipa::path(
        tag="user",
        get,
        path = "/api/user/seed/server",
        responses(
            (status = 200, description = "Server seed", body = Seed),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_server_seed(id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let seed = db
        .fetch_current_server_seed(id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::ServerSeedHidden(
        Seed {
            seed: seed.server_seed,
        },
    )))
}

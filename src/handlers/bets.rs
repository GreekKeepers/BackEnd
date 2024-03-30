use crate::{config, models::json_responses::Bets};

use super::*;

/// Get all last bets for a game
///
/// Gets 10 of the latest bets from the game
#[utoipa::path(
        tag="bets",
        get,
        path = "/api/bets/game/{game_name}",
        responses(
            (status = 200, description = "Bets", body = Bets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("game_name" = String, Path, description = "Name of the game")
        ),
    )]
pub async fn get_bets_for_game(game_name: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .fetch_bets_for_gamename(&game_name, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

/// Get player bets
///
/// Gets bets of the player by player id, max amount of returned bets per call is 10
#[utoipa::path(
        tag="bets",
        get,
        path = "/api/bets/user/{user_id}/{last_id}",
        responses(
            (status = 200, description = "User's bets", body = Bets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("address" = String, Path, description = "User address"),
            ("last_id" = Option<i64>, Path, description = "last bet id")
        ),
    )]
pub async fn get_user_bets(
    user_id: i64,
    last_id: Option<i64>,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .fetch_bets_for_user(user_id, last_id, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

/// Get player bets in increasing order
///
/// Gets bets of the player by player id, max amount of returned bets per call is 10
#[utoipa::path(
        tag="bets",
        get,
        path = "/api/bets/user/inc/{user_id}/{last_id}",
        responses(
            (status = 200, description = "User's bets", body = Bets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("address" = String, Path, description = "User address"),
            ("last_id" = Option<i64>, Path, description = "last bet id")
        ),
    )]
pub async fn get_user_bets_inc(
    user_id: i64,
    last_id: Option<i64>,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .fetch_bets_for_user_inc(user_id, last_id, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

/// Get all last bets
///
/// Gets 10 of the latest bets from all networks for all games
#[utoipa::path(
        tag="bets",
        get,
        path = "/api/bets/list",
        responses(
            (status = 200, description = "Bets", body = Bets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_all_last_bets(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .fetch_all_latest_bets(*config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

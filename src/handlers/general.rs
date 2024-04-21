use crate::models::{
    db_models::TimeBoundaries,
    json_responses::{LeaderboardResponse, PromTokens},
    LeaderboardType,
};

use super::*;

/// Get leaderboard data
///
/// Gets the leaderboard
#[utoipa::path(
        tag="general",
        get,
        path = "/api/general/leaderboard/{type}/{time_boundaries}",
        responses(
            (status = 200, description = "Leaderboard data, 20 records max", body = Vec<Leaderboard>),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("type" = LeaderboardType, Path, description = "Type of the leaderboard data volume/profit"),
            ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch leaderboard info"),
        ),
    )]
pub async fn get_leaderboard(
    leaderboard_type: LeaderboardType,
    time_boundaries: TimeBoundaries,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let leaderboard = match leaderboard_type {
        LeaderboardType::Volume => db.fetch_leaderboard_volume(time_boundaries, 20).await,
        LeaderboardType::Profit => db.fetch_leaderboard_profit(time_boundaries, 20).await,
    }
    .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Leaderboard(
        LeaderboardResponse { leaderboard },
    )))
}

/// Get tokens data
///
/// Gets the tokens data
#[utoipa::path(
        tag="general",
        get,
        path = "/api/general/promtokens",
        responses(
            (status = 200, description = "Tokens", body = PromTokens),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_prom_tokens(
    mut dexs: dexscreener::DexScreener,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let tokens = db
        .fetch_tokens_to_track()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    let tokens = dexs
        .fetch_pairs_raw(&tokens)
        .await
        .map_err(|e| reject::custom(ApiError::DexScreener(e)))?;
    Ok(gen_arbitrary_response(ResponseBody::PromTokens(
        PromTokens { tokens: &tokens },
    )))
}

/// Get totals
///
///
#[utoipa::path(
        tag="general",
        get,
        path = "/api/general/totals",
        responses(
            (status = 200, description = "Get Total values", body = Seed),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_totals(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let totals = db
        .fetch_totals()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
    Ok(gen_arbitrary_response(ResponseBody::Totals(totals)))
}

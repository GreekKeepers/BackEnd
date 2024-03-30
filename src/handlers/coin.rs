
use crate::models::json_responses::Coins;

use super::*;

/// Get all coins
///
/// Get all coins records
#[utoipa::path(
        tag="coin",
        get,
        path = "/api/coin/list",
        responses(
            (status = 200, description = "All coins records", body = Coin),
            (status = 500, description = "Internal server error", body = ErrorText),
        )
    )]
pub async fn get_all_coins(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let coins = db
        .fetch_coins()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Coins(Coins { coins })))
}

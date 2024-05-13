use crate::config::PASSWORD_SALT;
use crate::jwt;
use crate::models::db_models::{
    Partner, PartnerInfo, PartnerProgram, PartnerSiteInfo, PlayersTotals, TimeBoundaries,
};
use crate::models::json_requests::WithdrawRequest;
use crate::models::json_responses::{
    AccessToken, ClicksTimeMapped, ConnectedWalletInfo, ConnectedWalletsTimeMapped,
};
use crate::tools::blake_hash;
use blake2::{Blake2b512, Digest};
use chrono::{TimeZone, Utc};
use hex::ToHex;
use tracing::debug;

use self::json_requests::ChangePasswordRequest;

use super::*;

/// Register new partner account
///
/// Registers new partner account, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/register",
        request_body = RegisterPartner,
        responses(
            (status = 200, description = "Partner account was created", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn register_partner(
    data: json_requests::RegisterPartner,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let mut hasher = Blake2b512::new();

    hasher.update(data.password.as_bytes());

    let res = hasher.finalize().encode_hex();

    let partner_id = db
        .create_partner(
            Partner {
                name: data.name,
                country: data.country,
                traffic_source: data.traffic_source,
                users_amount_a_month: data.users_amount_a_month,
                id: 0,
                program: PartnerProgram::firstMonth,
                is_verified: false,
                login: data.login,
                password: res,
                registration_time: Default::default(),
                language: data.language,
            },
            &[],
        )
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    debug!("Partner with id {} created", partner_id);

    Ok(gen_info_response("Partner account has been created"))
}

/// Submit question
///
/// Submits question to be answered later
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/question",
        request_body = SubmitQuestion,
        responses(
            (status = 200, description = "Partner account was created", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn submit_question(
    _data: json_requests::SubmitQuestion,
    _db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    //db.submit_question(&data.name, &data.email, &data.message)
    //    .await
    //    .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Question submitted"))
}

/// Adds contacts to the account
///
/// Adds contact info to the existinf partner account, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/contacts/add",
        request_body = AddPartnerContacts,
        responses(
            (status = 200, description = "Partner account was created", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn add_contacts(
    partner_id: i64,
    data: json_requests::AddPartnerContacts,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.add_partner_contacts(
        partner_id,
        &data
            .contacts
            .into_iter()
            .map(|c| (c.name, c.url))
            .collect::<Vec<(String, String)>>(),
    )
    .await
    .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Contacts were added"))
}

/// Submits a new withdrawal request
///
/// Submits a new withdrawal request
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/withdraw",
        request_body = WithdrawRequest,
        responses(
            (status = 200, description = "Withdraw request was submitted", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn submit_withdrawal(
    partner_id: i64,
    data: WithdrawRequest,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.create_withdraw_request(partner_id, &data)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Contacts were added"))
}

/// Adds new site to the partner
///
/// Adds new site instaance to the partner account, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/site/add",
        request_body = AddPartnerSite,
        responses(
            (status = 200, description = "Site was added", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn add_partner_site(
    partner_id: i64,
    data: json_requests::AddPartnerSite,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.add_partner_site(partner_id, &data.url, &data.name)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Site was added"))
}

/// Adds new subb id
///
/// Adds new sub id to the existing site on partner's account, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/site/subid/add",
        request_body = AddPartnerSubid,
        responses(
            (status = 200, description = "SubId was added", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn add_partner_subid(
    partner_id: i64,
    data: json_requests::AddPartnerSubid,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.add_partner_subid(data.internal_site_id, partner_id, &data.url, &data.name)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Sub id was added"))
}

/// Adds click to subid
///
/// Adds click to sub id of the user's site
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/site/subid/click/{partner_address}/{site_id}/{sub_id}",
        responses(
            (status = 200, description = "Click was accepted", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("partner_address" = String, Path, description = "ETH address of the partner's account"),
            ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
            ("sub_id" = i64, Path, description = "Relative subid ofthe site, registered on partner's account"),
        ),
    )]
pub async fn click_partner_subid(
    partner_id: i64,
    site_id: i64,
    sub_id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let subid = db
        .get_subid(partner_id, site_id, sub_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    db.add_click(partner_id, subid.internal_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Click was successfully added"))
}

/// Connects new wallet with the given subid of the partner
///
/// Connects new wallet with the given subid of the partner, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/site/subid/connect",
        request_body = ConnectWallet,
        responses(
            (status = 200, description = "Wallet was connected", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn connect_wallet(
    user_id: i64,
    data: json_requests::ConnectWallet,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let time = chrono::offset::Utc::now();
    let subid = db
        .get_subid(data.partner_id, data.site_id, data.sub_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    db.add_ref_wallet(user_id, time, subid.internal_id, data.partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Wallet was successfully connected"))
}

/// Gets partner account info
///
/// Gets all basic info about partner account, requires signed signature from the user
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/get",
        responses(
            (status = 200, description = "Partner account was created", body = PartnerInfo),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_partner(partner_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let basic = db
        .get_partner(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
    let contacts = db
        .get_partner_contacts(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    let sites = db
        .get_partner_sites(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
    let mut sites_info: Vec<PartnerSiteInfo> = Vec::with_capacity(sites.len());
    for site in sites {
        let sub_ids = db
            .get_site_subids(site.internal_id)
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?;
        sites_info.push(PartnerSiteInfo {
            basic: site,
            sub_ids,
        })
    }

    Ok(gen_arbitrary_response(ResponseBody::PartnerInfo(
        PartnerInfo {
            basic,
            contacts,
            sites: sites_info,
        },
    )))
}

/// Gets partner contacts
///
/// Gets all contacts of the user
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/contacts/get",
        responses(
            (status = 200, description = "Partner account was created", body = PartnerContact),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_partner_contacts(
    partner_id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let contacts = db
        .get_partner_contacts(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::PartnerContacts(
        contacts,
    )))
}

/// Gets amount of connected wallets
///
/// Gets amount of wallets that connected to the partner
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/connected/{time_boundaries}",
        responses(
            (status = 200, description = "Connected wallets", body = AmountConnectedWallets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
        ),
    )]
pub async fn get_partner_connected_wallets(
    partner_id: i64,
    time_boundaries: TimeBoundaries,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let connected_wallets = db
        .get_partner_connected_wallets_amount(partner_id, time_boundaries)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(
        ResponseBody::AmountConnectedWallets(connected_wallets),
    ))
}

/// Gets amount of connected wallets that made deposits
///
/// Gets amount of wallets that connected to the partner and made at least one bet
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/connected_betted/{time_boundaries}",
        responses(
            (status = 200, description = "Connected wallets", body = AmountConnectedWallets),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
        ),
    )]
pub async fn get_partner_connected_wallets_with_deposits_amount(
    partner_id: i64,
    time_boundaries: TimeBoundaries,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let connected_wallets = db
        .get_partner_connected_wallets_with_deposits_amount(partner_id, time_boundaries)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(
        ResponseBody::AmountConnectedWallets(connected_wallets),
    ))
}

/// Gets connected wallets
///
/// Gets wallets that connected to the partner
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/wallets/{time_boundaries}",
        responses(
            (status = 200, description = "Connected wallets", body = ConnectedWalletInfo),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
        ),
    )]
pub async fn get_partner_connected_wallets_info(
    partner_id: i64,
    time_boundaries: TimeBoundaries,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let connected_wallets = db
        .get_partner_connected_wallets_info(partner_id, time_boundaries)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    let mut connected_wallets_stats: Vec<ConnectedWalletInfo> =
        Vec::with_capacity(connected_wallets.len());

    for wallet in connected_wallets {
        let stats = db
            .fetch_user_totals(wallet.user_id)
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?;

        connected_wallets_stats.push(ConnectedWalletInfo {
            id: wallet.id,
            user_id: wallet.user_id,
            timestamp: wallet.timestamp,
            site_id: wallet.site_id,
            sub_id: wallet.sub_id,
            bets_amount: stats.bets_amount,
            lost_bets: stats.lost_bets,
            won_bets: stats.won_bets,
            total_wagered_sum: stats.total_wagered_sum,
            gross_profit: stats.gross_profit,
            net_profit: stats.net_profit,
            highest_win: stats.highest_win,
        });
    }

    Ok(gen_arbitrary_response(ResponseBody::ConnectedWallets(
        connected_wallets_stats,
    )))
}

/// Gets amount of connected wallets
///
/// Gets amount of wallets that connected to the partner, withing specified time boundaries
/// time boundaries are specified as UNIX timestamps un UTC
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/connected/{start}/{end}/{step}",
        responses(
            (status = 200, description = "Connected wallets", body = ConnectedWalletsTimeMapped),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("start" = u64, Path, description = "Starting timestamp for the search"),
            ("end" = u64, Path, description = "Ending timestamp for the search"),
            ("step" = u64, Path, description = "Step from start to end"),
        ),
    )]
pub async fn get_partner_connected_wallets_exact_date(
    partner_id: i64,
    begin: u64,
    end: u64,
    step: u64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let capacity = ((end - begin) / step) as usize;
    if capacity > 100 {
        return Err(reject::custom(ApiError::BadRange));
    }
    let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

    for start in (begin..end).step_by(step as usize) {
        connected_wallets.push(
            db.get_partner_connected_wallets_amount_exact_date(
                partner_id,
                Utc.timestamp_opt(start as i64, 0).unwrap(),
                Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
            )
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?
            .connected_users,
        );
    }

    Ok(gen_arbitrary_response(
        ResponseBody::AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped {
            amount: connected_wallets,
        }),
    ))
}

/// Gets amount of connected wallets with bets
///
/// Gets amount of wallets that connected to the partner and made at least one bet, withing specified time boundaries
/// time boundaries are specified as UNIX timestamps un UTC
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/connected_betted/betted/{start}/{end}/{step}",
        responses(
            (status = 200, description = "Connected wallets", body = ConnectedWalletsTimeMapped),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("start" = u64, Path, description = "Starting timestamp for the search"),
            ("end" = u64, Path, description = "Ending timestamp for the search"),
            ("step" = u64, Path, description = "Step from start to end"),
        ),
    )]
pub async fn get_partner_connected_wallets_betted_exact_date(
    partner_id: i64,
    begin: u64,
    end: u64,
    step: u64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let capacity = ((end - begin) / step) as usize;
    if capacity > 100 {
        return Err(reject::custom(ApiError::BadRange));
    }
    let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

    for start in (begin..end).step_by(step as usize) {
        connected_wallets.push(
            db.get_partner_connected_wallets_with_bets_amount_exact_date(
                partner_id,
                Utc.timestamp_opt(start as i64, 0).unwrap(),
                Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
            )
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?
            .connected_users,
        );
    }

    Ok(gen_arbitrary_response(
        ResponseBody::AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped {
            amount: connected_wallets,
        }),
    ))
}

/// Gets totals for the partner
///
/// Gets totals on lost bets of the connected wallets
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/connected/totals",
        responses(
            (status = 200, description = "Totals", body = PlayersTotals),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_connected_totals(user_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let totals = db
        .fetch_user_totals(user_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::UserTotals(totals)))
}

/// Gets withdrawal history
///
/// Gets withdrawals of the partner
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/withdrawals/{time_boundaries}",
        responses(
            (status = 200, description = "Totals", body = WithdrawRequest),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch withdrawal requests"),
        ),
    )]
pub async fn get_withdrawal_requests(
    partner_id: i64,
    time_boundaries: TimeBoundaries,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let withdrawals = db
        .get_partner_withdrawal_requests(partner_id, time_boundaries)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Withdrawals(
        withdrawals,
    )))
}

/// Gets amount of clicks
///
/// Gets amount of click for the partner links, within specified time boundaries
/// time boundaries are specified as UNIX timestamps un UTC
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/clicks/{start}/{end}/{step}",
        responses(
            (status = 200, description = "Clicks", body = ClicksTimeMapped),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("start" = u64, Path, description = "Starting timestamp for the search"),
            ("end" = u64, Path, description = "Ending timestamp for the search"),
            ("step" = u64, Path, description = "Step from start to end"),
        ),
    )]
pub async fn get_partner_clicks_exact_date(
    partner_id: i64,
    begin: u64,
    end: u64,
    step: u64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let capacity = ((end - begin) / step) as usize;
    if capacity > 100 {
        return Err(reject::custom(ApiError::BadRange));
    }
    let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

    for start in (begin..end).step_by(step as usize) {
        connected_wallets.push(
            db.get_partner_clicks_exact_date(
                partner_id,
                Utc.timestamp_opt(start as i64, 0).unwrap(),
                Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
            )
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?
            .clicks,
        );
    }

    Ok(gen_arbitrary_response(
        ResponseBody::AmountClicksTimeMapped(ClicksTimeMapped {
            amount: connected_wallets,
        }),
    ))
}

/// Gets partner sites
///
/// Gets all sites of the user
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/site/get",
        responses(
            (status = 200, description = "Partner's site", body = PartnerSiteInfo),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_partner_sites(partner_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let sites = db
        .get_partner_sites(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;
    let mut sites_info: Vec<PartnerSiteInfo> = Vec::with_capacity(sites.len());
    for site in sites {
        let sub_ids = db
            .get_site_subids(site.internal_id)
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?;
        sites_info.push(PartnerSiteInfo {
            basic: site,
            sub_ids,
        })
    }

    Ok(gen_arbitrary_response(ResponseBody::PartnerSiteInfo(
        sites_info,
    )))
}

/// Remove partner contacts
///
/// Gets all contacts of the user
#[utoipa::path(
        tag="partner",
        delete,
        path = "/api/partner/contacts/delete",
        responses(
            (status = 200, description = "Partner contact was removed", body = DeletePartnerContacts),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn delete_partner_contacts(
    partner_id: i64,
    contacts: json_requests::DeletePartnerContacts,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.delete_partner_contacts(partner_id, &contacts.contacts)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("Contact was deleted"))
}

/// Gets clicks for the subid
///
/// Gets all the clicks accumulated for subid
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/site/subid/clicks/{site_id}/{sub_id}",
        responses(
            (status = 200, description = "Partner's subid clicks", body = RefClicks),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
            ("sub_id" = i64, Path, description = "Relative subid ofthe site, registered on partner's account"),
        ),
    )]
pub async fn get_clicks(
    partner_id: i64,
    site_id: i64,
    sub_id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let clicks = db
        .get_subid_clicks(partner_id, site_id, sub_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
}

/// Gets clicks for the site
///
/// Gets all the clicks accumulated for site
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/site/clicks/{site_id}",
        responses(
            (status = 200, description = "Partner's site clicks", body = RefClicks),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
        params(
            ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
        ),
    )]
pub async fn get_site_clicks(
    partner_id: i64,
    site_id: i64,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let clicks = db
        .get_site_clicks(partner_id, site_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
}

/// Gets clicks for the partner
///
/// Gets all the clicks accumulated for partner
#[utoipa::path(
        tag="partner",
        get,
        path = "/api/partner/clicks",
        responses(
            (status = 200, description = "Partner's site clicks", body = RefClicks),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn get_partner_clicks(partner_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let clicks = db
        .get_partner_clicks(partner_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
}

/// Change password of the partner
///
/// Changes the password of the partner
#[utoipa::path(
        tag="partner",
        put,
        path = "/api/partner/change/password",
        request_body = ChangePasswordRequest,
        responses(
            (status = 200, description = "Partner's site clicks", body = InfoText),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn partner_change_password(
    partner_id: i64,
    data: ChangePasswordRequest,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let old_hashed = blake_hash(&data.old_password);
    let new_hashed = blake_hash(&data.new_password);
    if !db
        .partner_change_password(partner_id, &old_hashed, &new_hashed)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
    {
        return Err(reject::custom(ApiError::BadPassword));
    }

    Ok(gen_info_response("Password was changed successfully"))
}

/// Login partner
///
/// Logins partner with provided login/password
#[utoipa::path(
        tag="partner",
        post,
        path = "/api/partner/login",
        request_body = Login,
        responses(
            (status = 200, description = "Access token", body = AccessToken),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
pub async fn login_partner(login: Login, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let hashed_password = blake_hash(&login.password);
    let partner = db
        .login_partner(&login.login, &hashed_password)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(reject::custom(ApiError::WrongLoginPassword))?;

    let token = jwt::generate_token(
        &jwt::Payload {
            iss: "local".into(),
            sub: partner.id,
            exp: 100,
            iat: 100,
            aud: "Auth".into(),
        },
        &format!("{:?}{:?}", *PASSWORD_SALT, hashed_password),
    );

    Ok(gen_arbitrary_response(ResponseBody::AccessToken(
        AccessToken {
            access_token: token.clone(),
            token_type: "Bearer".into(),
            expires_in: 100,
            refresh_token: token,
        },
    )))
}

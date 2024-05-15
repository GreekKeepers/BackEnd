use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
#[schema(rename_all = "lowercase")]
pub enum LeaderboardType {
    Volume,
    Profit,
}

impl FromStr for LeaderboardType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "volume" => Ok(Self::Volume),
            "profit" => Ok(Self::Profit),
            _ => Err("No such variant was found in enum LeaderboardType"),
        }
    }
}

pub mod db_models {

    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;

    #[derive(Debug, Clone, ToSchema)]
    #[schema(rename_all = "lowercase")]
    pub enum TimeBoundaries {
        Daily,
        Weekly,
        Monthly,
        All,
    }

    impl FromStr for TimeBoundaries {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "daily" => Ok(Self::Daily),
                "weekly" => Ok(Self::Weekly),
                "monthly" => Ok(Self::Monthly),
                "all" => Ok(Self::All),
                _ => Err("No such variant was found in enum TimeBoundaries"),
            }
        }
    }
    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct ConnectedWallet {
        pub id: i64,
        pub user_id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub site_id: i64,
        pub sub_id: i64,
    }
    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Leaderboard {
        pub user_id: i64,
        pub total: Decimal,
        pub username: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Payout {
        pub id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub amount: Decimal,
        pub status: i32,
        pub additional_data: String,
        pub user_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    pub struct PlayersTotals {
        pub bets_amount: i64,
        pub lost_bets: i64,
        pub won_bets: i64,
        pub total_wagered_sum: Option<f64>,
        pub gross_profit: Option<f64>,
        pub net_profit: Option<f64>,
        pub highest_win: Option<f64>,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, sqlx::Type, PartialEq, PartialOrd)]
    #[sqlx(type_name = "oauth_provider", rename_all = "lowercase")]
    pub enum OauthProvider {
        Local,
        Google,
        Facebook,
        Twitter,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
    pub struct User {
        pub id: i64,
        #[serde(with = "ts_seconds")]
        pub registration_time: DateTime<Utc>,

        pub login: String,
        pub username: String,
        pub password: String,
        pub user_level: i64,
        pub provider: OauthProvider,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct RefreshToken {
        pub token: String,
        #[serde(with = "ts_seconds")]
        pub creation_date: DateTime<Utc>,
        pub user_id: i64,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
    pub struct Coin {
        pub id: i64,
        pub name: String,
        pub price: Decimal,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct Amount {
        pub name: String,
        pub amount: Decimal,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct Game {
        pub id: i64,
        pub name: String,
        pub parameters: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Totals {
        pub bets_amount: i64,
        pub player_amount: i64,
        pub sum: Option<Decimal>,
    }

    // #[derive(Deserialize, Serialize, Clone, ToSchema)]
    // pub enum GameParameters{
    //     CoinFlip()
    // }

    pub struct GameResult {
        pub total_profit: Decimal,
        pub outcomes: Vec<u64>,
        pub profits: Vec<Decimal>,
        pub num_games: u32,
        pub data: String,
        pub finished: bool,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct UserSeed {
        pub id: i64,
        //pub relative_id: i64,
        pub user_id: i64,
        pub user_seed: String,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct ReferalLink {
        pub id: i64,
        pub refer_to: i64,
        pub link_name: String,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct ServerSeed {
        pub id: i64,
        //pub relative_id: i64,
        pub user_id: i64,

        pub server_seed: String,
        pub revealed: bool,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, Default)]
    pub struct Bet {
        pub id: i64,
        //pub relative_id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub amount: Decimal,
        pub profit: Decimal,
        pub num_games: i32,
        pub outcomes: String,
        pub profits: Vec<Decimal>,

        pub bet_info: String,
        pub state: Option<String>,

        pub uuid: String,

        pub game_id: i64,
        pub user_id: i64,
        pub coin_id: i64,
        pub userseed_id: i64,
        pub serverseed_id: i64,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, Default)]
    pub struct GameState {
        pub id: i64,
        //pub relative_id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub amount: Decimal,

        pub bet_info: String,
        pub state: String,

        pub uuid: String,

        pub game_id: i64,
        pub user_id: i64,
        pub coin_id: i64,
        pub userseed_id: i64,
        pub serverseed_id: i64,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, Default)]
    pub struct UserTotals {
        pub bets_amount: i64,
        pub lost_bets: i64,
        pub won_bets: i64,
        pub total_wagered_sum: Decimal,
        pub gross_profit: Decimal,
        pub net_profit: Decimal,
        pub highest_win: Decimal,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Default, Debug)]
    pub struct Invoice {
        pub id: String,
        pub merchant_id: String,
        pub order_id: String,
        #[serde(with = "ts_seconds")]
        pub create_date: DateTime<Utc>,
        pub status: i32,
        pub pay_url: String,
        pub user_id: i64,
        pub amount: Decimal,
        pub currency: String,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, sqlx::Type, Debug, strum::Display)]
    #[sqlx(type_name = "billine_status", rename_all = "lowercase")]
    pub enum BillineInvoiceStatus {
        #[strum(to_string = "pending")]
        Pending,
        #[strum(to_string = "success")]
        Success,
        #[strum(to_string = "failed")]
        Failed,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
    pub struct BillineInvoice {
        pub id: String,
        pub merchant_id: String,
        pub order_id: String,
        #[serde(with = "ts_seconds")]
        pub create_date: DateTime<Utc>,
        pub status: BillineInvoiceStatus,
        pub user_id: i64,
        pub amount: Decimal,
        pub currency: String,
    }

    impl Into<Invoice> for thedex::models::Invoice {
        fn into(self) -> Invoice {
            Invoice {
                id: self.order_id.clone().unwrap_or_default(),
                merchant_id: self.merchant_id,
                order_id: self.order_id.unwrap_or_default(),
                create_date: Default::default(),
                status: self.status as i32,
                pay_url: self.purse,
                user_id: i64::from_str_radix(&self.unique_user_id.unwrap_or_default(), 10)
                    .unwrap_or_default(),
                amount: self.amount,
                currency: self.currency,
            }
        }
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct RefClicks {
        //pub id: i64,
        pub clicks: i64,
        // pub sub_id_internal: i64,
        // pub partner_id: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct PartnerSite {
        pub internal_id: i64,
        pub id: i64,
        pub name: String,
        pub url: String,
        pub partner_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct SiteSubId {
        pub internal_id: i64,
        pub id: i64,
        pub name: String,
        pub url: String,
        pub site_id: i64,
        pub partner_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct PartnerContact {
        pub id: i64,
        pub name: String,
        pub url: String,
        pub partner_id: i64,
    }
    #[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize, ToSchema)]
    #[sqlx(type_name = "partnerprogram")]
    #[allow(non_camel_case_types)]
    pub enum PartnerProgram {
        firstMonth,
        novice,
        beginner,
        intermediate,
        advanced,
        pro,
        god,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Partner {
        //pub id: i64,
        pub name: String,
        pub country: String,
        pub traffic_source: String,
        pub users_amount_a_month: i64,
        pub id: i64,
        pub program: PartnerProgram,
        pub is_verified: bool,
        pub login: String,
        pub password: String,
        #[serde(with = "ts_seconds")]
        pub registration_time: DateTime<Utc>,
        pub language: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct PartnerSiteInfo {
        pub basic: PartnerSite,
        pub sub_ids: Vec<SiteSubId>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct PartnerInfo {
        pub basic: Partner,
        pub contacts: Vec<PartnerContact>,
        pub sites: Vec<PartnerSiteInfo>,
    }
    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    pub struct PlayerTotals {
        pub bets_amount: i64,
        pub lost_bets: i64,
        pub won_bets: i64,
        pub total_wagered_sum: Option<f64>,
        pub gross_profit: Option<f64>,
        pub net_profit: Option<f64>,
        pub highest_win: Option<f64>,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Withdrawal {
        pub id: i64,
        pub start_time: DateTime<Utc>,
        pub token: String,
        pub network: String,
        pub wallet_address: String,
        pub status: String,
        pub partner_id: i64,
        pub amount: String,
    }
}

pub mod json_responses {

    use crate::WsData;

    use self::db_models::{
        Amount, Bet, Coin, Game, GameState, Invoice, Leaderboard, PartnerContact, PartnerInfo,
        PartnerSite, PartnerSiteInfo, PlayerTotals, RefClicks, SiteSubId, Totals, UserTotals,
        Withdrawal,
    };

    // use super::db_models::{
    //     AmountConnectedWallets, Bet, BetInfo, BlockExplorerUrl, Game, GameAbi, Leaderboard,
    //     NetworkInfo, Nickname, Partner, PartnerContact, PartnerSite, Player, PlayerTotals,
    //     PlayersTotals, RefClicks, RpcUrl, SiteSubId, Token, Totals, Withdrawal,
    // };
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use thedex::models::Price;

    #[derive(Serialize, Deserialize, ToSchema)]
    pub enum Status {
        OK,
        Err,
    }

    #[derive(Serialize, Deserialize)]
    pub struct TextResponse {
        // OK/ERR
        //#[schema(example = "OK")]
        pub status: String,

        //#[schema(example = "Info message")]
        pub message: String,
    }

    #[derive(Serialize, ToSchema)]
    pub struct JsonResponse<'a> {
        pub status: Status,
        pub body: ResponseBody<'a>,
    }

    #[derive(Serialize, ToSchema)]
    #[serde(tag = "type")]
    pub enum ResponseBody<'a> {
        ErrorText(ErrorText),
        InfoText(InfoText),
        Amounts(Amounts),
        User(UserStripped),
        Invoice(Invoice),
        ClientSeed(Seed),
        Games(Games),
        Uuid(UuidToken),
        Coins(Coins),
        Prices(Prices),
        // Networks(Networks),
        // Rpcs(Rpcs),
        // BlockExplorers(BlockExplorers),
        // Tokens(Tokens),
        // Game(Game),
        // Nickname(Nickname),
        // Player(Player),
        Bets(Bets),
        Bet(BetExpanded),
        State(GameState),
        ServerSeedHidden(Seed),
        // Abi(GameAbi),
        Totals(Totals),
        LatestGames(LatestGames),
        OneTimeToken(OneTimeToken),
        PlayerTotals(PlayerTotals),
        // TokenPrice(TokenPrice),
        PartnerInfo(PartnerInfo),
        PartnerContacts(Vec<PartnerContact>),
        PartnerSiteInfo(Vec<PartnerSiteInfo>),
        Leaderboard(LeaderboardResponse),
        Clicks(RefClicks),
        AmountConnectedWallets(AmountConnectedWallets),
        AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped),
        AmountClicksTimeMapped(ClicksTimeMapped),
        ConnectedWallets(Vec<ConnectedWalletInfo>),
        AccessToken(AccessToken),
        UserTotals(UserTotals),
        ChatMessage(PropagatedChatMessage),
        BillineCreateInvoice(BillineCreateInvoiceResponse), // Withdrawals(Vec<Withdrawal>),
        Withdrawals(Vec<Withdrawal>),
        // TODO: idk, fix that
        PromTokens(PromTokens<'a>),
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct ClicksTimeMapped {
        pub amount: Vec<i64>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct AmountConnectedWallets {
        pub connected_users: i64,
    }
    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct ConnectedWalletsTimeMapped {
        pub amount: Vec<i64>,
    }
    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct ConnectedWalletInfo {
        pub id: i64,
        pub user_id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub site_id: i64,
        pub sub_id: i64,
        pub bets_amount: i64,
        pub lost_bets: i64,
        pub won_bets: i64,
        pub total_wagered_sum: Decimal,
        pub gross_profit: Decimal,
        pub net_profit: Decimal,
        pub highest_win: Decimal,
    }

    #[derive(Serialize, Clone, ToSchema)]
    pub struct PromTokens<'a> {
        pub tokens: &'a [dexscreener::models::Pair],
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct LeaderboardResponse {
        pub leaderboard: Vec<Leaderboard>,
    }

    impl<'a> From<WsData> for ResponseBody<'a> {
        fn from(value: WsData) -> Self {
            match value {
                WsData::NewBet(bet) => ResponseBody::Bet(bet),
                WsData::ServerSeed(seed) => ResponseBody::ServerSeedHidden(Seed { seed }),
                WsData::StateUpdate(state) => ResponseBody::State(state),
                WsData::NewMessage(m) => ResponseBody::ChatMessage(m),
                WsData::Invoice(invoice) => ResponseBody::Invoice(invoice),
            }
        }
    }

    impl<'a> From<&WsData> for ResponseBody<'a> {
        fn from(value: &WsData) -> Self {
            match value {
                WsData::NewBet(bet) => ResponseBody::Bet(bet.clone()),
                WsData::ServerSeed(seed) => {
                    ResponseBody::ServerSeedHidden(Seed { seed: seed.clone() })
                }
                WsData::StateUpdate(state) => ResponseBody::State(state.clone()),
                WsData::NewMessage(m) => ResponseBody::ChatMessage(m.clone()),
                WsData::Invoice(invoice) => ResponseBody::Invoice(invoice.clone()),
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Prices {
        pub prices: Vec<Price>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct UuidToken {
        pub uuid: String,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct Games {
        pub games: Vec<Game>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct Coins {
        pub coins: Vec<Coin>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct Amounts {
        pub amounts: Vec<Amount>,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct Seed {
        pub seed: String,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct UserStripped {
        pub id: i64,
        #[serde(with = "ts_seconds")]
        pub registration_time: DateTime<Utc>,
        pub username: String,
    }

    // #[derive(Serialize, Deserialize, Clone, ToSchema)]
    // pub struct ConnectedWalletInfo {
    //     pub id: i64,
    //     pub address: String,
    //     #[serde(with = "ts_seconds")]
    //     pub timestamp: DateTime<Utc>,
    //     pub site_id: i64,
    //     pub sub_id: i64,
    //     pub bets_amount: i64,
    //     pub lost_bets: i64,
    //     pub won_bets: i64,
    //     pub total_wagered_sum: Option<f64>,
    //     pub gross_profit: Option<f64>,
    //     pub net_profit: Option<f64>,
    //     pub highest_win: Option<f64>,
    // }

    // #[derive(Serialize, Deserialize, Clone, ToSchema)]
    // pub struct ConnectedWalletsTimeMapped {
    //     pub amount: Vec<i64>,
    // }

    // #[derive(Serialize, Deserialize, Clone, ToSchema)]
    // pub struct ClicksTimeMapped {
    //     pub amount: Vec<i64>,
    // }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct AccessToken {
        pub access_token: String,
        pub token_type: String,
        pub expires_in: usize,
        pub refresh_token: String,
    }

    // #[derive(Serialize, Deserialize, Clone, ToSchema)]
    // pub struct PartnerInfo {
    //     pub basic: Partner,
    //     pub contacts: Vec<PartnerContact>,
    //     pub sites: Vec<PartnerSiteInfo>,
    // }

    // #[derive(Serialize, Deserialize, Clone, ToSchema)]
    // pub struct PartnerSiteInfo {
    //     pub basic: PartnerSite,
    //     pub sub_ids: Vec<SiteSubId>,
    // }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct ErrorText {
        pub error: String,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct InfoText {
        pub message: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct LatestGames {
        pub games: Vec<String>,
    }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct TokenPrice {
    //     pub token_price: f64,
    // }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct NetworkFullInfo {
    //     pub basic_info: NetworkInfo,
    //     pub rpcs: Vec<RpcUrl>,
    //     pub explorers: Vec<BlockExplorerUrl>,
    // }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct Networks {
    //     pub networks: Vec<NetworkFullInfo>,
    // }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct Rpcs {
    //     pub rpcs: Vec<RpcUrl>,
    // }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct BlockExplorers {
    //     pub explorers: Vec<BlockExplorerUrl>,
    // }

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct Tokens {
    //     pub tokens: Vec<Token>,
    // }

    // #[serde_as]
    // #[derive(Deserialize, Serialize, Clone, Debug)]
    // pub struct Card {
    //     pub number: u8,
    //     pub suit: u8,
    // }

    // #[serde_as]
    // #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    // pub struct BetInfoResponse {
    //     pub id: i64,
    //     pub transaction_hash: String,
    //     pub player: String,
    //     pub player_nickname: Option<String>,
    //     #[serde(with = "ts_seconds")]
    //     pub timestamp: DateTime<Utc>,
    //     pub game_id: i64,
    //     pub game_name: String,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub wager: BigDecimal,
    //     pub token_address: String,
    //     pub token_name: String,
    //     pub network_id: i64,
    //     pub network_name: String,
    //     pub bets: i64,
    //     pub multiplier: f64,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub profit: BigDecimal,
    //     pub player_hand: Option<Vec<Card>>,
    // }

    // impl From<BetInfo> for BetInfoResponse {
    //     fn from(value: BetInfo) -> Self {
    //         BetInfoResponse {
    //             id: value.id,
    //             transaction_hash: value.transaction_hash,
    //             player: value.player,
    //             player_nickname: value.player_nickname,
    //             timestamp: value.timestamp,
    //             game_id: value.game_id,
    //             game_name: value.game_name,
    //             wager: value.wager,
    //             token_address: value.token_address,
    //             token_name: value.token_name,
    //             network_id: value.network_id,
    //             network_name: value.network_name,
    //             bets: value.bets,
    //             multiplier: value.multiplier,
    //             profit: value.profit,
    //             player_hand: None,
    //         }
    //     }
    // }

    // impl From<BetInfoResponse> for BetInfo {
    //     fn from(value: BetInfoResponse) -> Self {
    //         BetInfo {
    //             id: value.id,
    //             transaction_hash: value.transaction_hash,
    //             player: value.player,
    //             player_nickname: value.player_nickname,
    //             timestamp: value.timestamp,
    //             game_id: value.game_id,
    //             game_name: value.game_name,
    //             wager: value.wager,
    //             token_address: value.token_address,
    //             token_name: value.token_name,
    //             network_id: value.network_id,
    //             network_name: value.network_name,
    //             bets: value.bets,
    //             multiplier: value.multiplier,
    //             profit: value.profit,
    //         }
    //     }
    // }

    // impl From<Bet> for BetInfoResponse {
    //     fn from(value: Bet) -> Self {
    //         BetInfoResponse {
    //             id: value.id,
    //             transaction_hash: value.transaction_hash,
    //             player: value.player,
    //             player_nickname: Default::default(),
    //             timestamp: value.timestamp,
    //             game_id: value.game_id,
    //             game_name: Default::default(),
    //             wager: value.wager,
    //             token_address: value.token_address,
    //             token_name: Default::default(),
    //             network_id: value.network_id,
    //             network_name: Default::default(),
    //             bets: value.bets,
    //             multiplier: value.multiplier,
    //             profit: value.profit,
    //             player_hand: None,
    //         }
    //     }
    // }
    // impl From<BetInfoResponse> for Bet {
    //     fn from(value: BetInfoResponse) -> Self {
    //         Bet {
    //             id: value.id,
    //             transaction_hash: value.transaction_hash,
    //             player: value.player,
    //             timestamp: value.timestamp,
    //             game_id: value.game_id,
    //             wager: value.wager,
    //             token_address: value.token_address,
    //             network_id: value.network_id,
    //             bets: value.bets,
    //             multiplier: value.multiplier,
    //             profit: value.profit,
    //         }
    //     }
    // }

    // #[serde_as]
    // #[derive(Deserialize, Serialize, Clone, Debug)]
    // pub struct BetIntermidiate {
    //     pub id: i64,
    //     pub transaction_hash: String,
    //     pub player: String,
    //     #[serde(with = "ts_seconds")]
    //     pub timestamp: DateTime<Utc>,
    //     pub game_id: i64,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub wager: BigDecimal,
    //     pub token_address: String,
    //     pub network_id: i64,
    //     pub bets: i64,
    //     pub multiplier: f64,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub profit: BigDecimal,
    //     //pub player_hand: Option<[Card; 5]>,
    // }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, Default)]
    pub struct OneTimeToken {
        pub token: String,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Debug, Default)]
    pub struct BetExpanded {
        pub id: i64,
        //pub relative_id: i64,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub amount: Decimal,
        pub profit: Decimal,
        pub num_games: i32,
        pub outcomes: String,
        pub profits: String,

        pub bet_info: String,
        pub state: Option<String>,

        pub uuid: String,

        pub game_id: i64,
        pub user_id: i64,
        pub username: String,
        pub coin_id: i64,
        pub userseed_id: i64,
        pub serverseed_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Bets {
        pub bets: Vec<BetExpanded>,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct PropagatedChatMessage {
        pub room_id: i64,
        pub user_id: i64,
        pub username: String,
        pub level: i64,
        pub avatar: Option<String>,
        pub message: String,
        pub attached_media: Option<String>,
        pub mentions: Vec<i64>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct BillineCreateInvoiceResponse {
        pub data: billine::RequestIframe,
        pub sign: String,
    }
}

pub mod json_requests {
    use rust_decimal::Decimal;
    use serde_repr::{Deserialize_repr, Serialize_repr};

    use super::*;

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct SetNickname {
    //     pub address: String,
    //     pub nickname: String,
    //     pub signature: String,
    // }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct QrRequest {
        pub data: String,
    }

    #[derive(Serialize_repr, Deserialize_repr, ToSchema, Clone)]
    #[repr(u32)]
    pub enum InvoiceAmount {
        Ten = 10,
        Fifty = 50,
        Hundred = 100,
        FiveHundred = 500,
        Thousand = 1000,
        TwoThousand = 2000,
    }

    #[derive(Deserialize, Serialize, ToSchema, Clone)]
    pub struct PayoutRequest {
        pub amount: Decimal,
        pub additional_data: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Clone)]
    pub struct CreateInvoice {
        pub amount: InvoiceAmount,
        pub currency: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Clone)]
    pub struct CreateBillineInvoice {
        pub amount: Decimal,
        pub currency: String,
        pub first_name: String,
        pub last_name: String,
        pub country: String,
        pub email: String,
        pub phone: String,
        pub address: String,
        pub city: String,
        pub post_code: String,
        pub region: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct ChangeNickname {
        pub nickname: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Login {
        pub login: String,
        pub password: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct SubmitError {
        pub error: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct CreateReferal {
        pub refer_to: String,
        pub referal: String,
        pub signature: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct ByNetworkId {
        pub network_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct PropagatedBet {
        pub game_id: i64,
        pub amount: Decimal,
        pub coin_id: i64,
        pub user_id: Option<i64>,
        pub uuid: Option<String>,
        pub data: String,
        pub stop_loss: Decimal,
        pub stop_win: Decimal,
        pub num_games: u64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct ChatMessage {
        pub message: String,
        pub attached_media: Option<String>,
        pub mentions: Vec<String>,
        pub chat_room: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct ContinueGame {
        pub game_id: i64,
        pub coin_id: i64,
        pub user_id: Option<i64>,
        pub uuid: Option<String>,
        pub data: String,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    #[serde(tag = "type")]
    pub enum WebsocketsIncommingMessage {
        Auth { token: String },
        SubscribeBets { payload: Vec<i64> },
        UnsubscribeBets { payload: Vec<i64> },
        SubscribeInvoice,
        UnsubscribeInvoice,
        SubscribeAllBets,
        UnsubscribeAllBets,
        Ping,
        NewClientSeed { seed: String },
        NewServerSeed,
        MakeBet(PropagatedBet),
        ContinueGame(ContinueGame),
        GetState(GetState),
        GetUuid,
        NewMessage(ChatMessage),
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct GetState {
        pub game_id: i64,
        pub coin_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct RegisterPartner {
        pub name: String,
        pub country: String,
        pub traffic_source: String,
        pub users_amount_a_month: i64,
        pub main_wallet: String,
        pub login: String,
        pub password: String,
        pub language: Option<String>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct RegisterUser {
        pub username: String,
        pub password: String,
        pub h_captcha_response: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct PartnerContactBasic {
        pub name: String,
        pub url: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct AddPartnerContacts {
        pub contacts: Vec<PartnerContactBasic>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct DeletePartnerContacts {
        pub contacts: Vec<i64>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct AddPartnerSite {
        pub name: String,
        pub url: String,
        pub language: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct AddPartnerSubid {
        pub name: String,
        pub url: String,
        pub internal_site_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct ConnectWallet {
        pub partner_id: i64,
        pub site_id: i64,
        pub sub_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct WithdrawRequest {
        pub token: String,
        pub network: String,
        pub wallet_address: String,
        pub amount: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct ChangePasswordRequest {
        pub old_password: String,
        pub new_password: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct SubmitQuestion {
        pub name: String,
        pub email: String,
        pub message: String,
    }
}

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

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct User {
        pub id: i64,
        #[serde(with = "ts_seconds")]
        pub registration_time: DateTime<Utc>,

        pub login: String,
        pub username: String,
        pub password: String,
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

    // #[derive(Deserialize, Serialize, Clone, ToSchema)]
    // pub enum GameParameters{
    //     CoinFlip()
    // }

    pub struct GameResult {
        pub total_profit: Decimal,
        pub outcomes: Vec<u32>,
        pub num_games: u32,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema)]
    pub struct UserSeed {
        pub id: i64,
        //pub relative_id: i64,
        pub user_id: i64,
        pub user_seed: String,
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

        pub bet_info: String,

        pub uuid: String,

        pub game_id: i64,
        pub user_id: i64,
        pub coin_id: i64,
        pub userseed_id: i64,
        pub serverseed_id: i64,
    }

    #[derive(Deserialize, Serialize, Clone, ToSchema, Default)]
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
}

pub mod json_responses {

    use crate::WsData;

    use self::db_models::{Amount, Bet, Game, Invoice};

    // use super::db_models::{
    //     AmountConnectedWallets, Bet, BetInfo, BlockExplorerUrl, Game, GameAbi, Leaderboard,
    //     NetworkInfo, Nickname, Partner, PartnerContact, PartnerSite, Player, PlayerTotals,
    //     PlayersTotals, RefClicks, RpcUrl, SiteSubId, Token, Totals, Withdrawal,
    // };
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};

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

    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct JsonResponse {
        pub status: Status,
        pub body: ResponseBody,
    }

    #[derive(Serialize, Deserialize, ToSchema)]
    #[serde(tag = "type")]
    pub enum ResponseBody {
        ErrorText(ErrorText),
        InfoText(InfoText),
        Amounts(Amounts),
        User(UserStripped),
        Invoice(Invoice),
        ClientSeed(Seed),
        Games(Games),
        Uuid(UuidToken),

        // Networks(Networks),
        // Rpcs(Rpcs),
        // BlockExplorers(BlockExplorers),
        // Tokens(Tokens),
        // Game(Game),
        // Nickname(Nickname),
        // Player(Player),
        Bets(Bets),
        Bet(Bet),
        ServerSeedHidden(Seed),
        // Abi(GameAbi),
        // Totals(Totals),
        // LatestGames(LatestGames),
        // PlayerTotals(PlayerTotals),
        // TokenPrice(TokenPrice),
        // PartnerInfo(PartnerInfo),
        // PartnerContacts(Vec<PartnerContact>),
        // PartnerSiteInfo(Vec<PartnerSiteInfo>),
        // Leaderboard(Vec<Leaderboard>),
        // Clicks(RefClicks),
        // AmountConnectedWallets(AmountConnectedWallets),
        // AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped),
        // AmountClicksTimeMapped(ClicksTimeMapped),
        // ConnectedWallets(Vec<ConnectedWalletInfo>),
        AccessToken(AccessToken),
        // PlayersTotals(PlayersTotals),
        // Withdrawals(Vec<Withdrawal>),
    }

    impl From<WsData> for ResponseBody {
        fn from(value: WsData) -> Self {
            match value {
                WsData::NewBet(bet) => ResponseBody::Bet(bet),
                WsData::ServerSeed(seed) => ResponseBody::ServerSeedHidden(Seed { seed }),
            }
        }
    }

    impl From<&WsData> for ResponseBody {
        fn from(value: &WsData) -> Self {
            match value {
                WsData::NewBet(bet) => ResponseBody::Bet(bet.clone()),
                WsData::ServerSeed(seed) => {
                    ResponseBody::ServerSeedHidden(Seed { seed: seed.clone() })
                }
            }
        }
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

    // #[derive(Deserialize, Serialize, ToSchema)]
    // pub struct LatestGames {
    //     pub games: Vec<String>,
    // }

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

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Bets {
        pub bets: Vec<Bet>,
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
    pub struct CreateInvoice {
        pub amount: InvoiceAmount,
        pub currency: String,
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

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    #[serde(tag = "type")]
    pub enum WebsocketsIncommingMessage {
        Auth { token: String },
        SubscribeBets { payload: Vec<i64> },
        UnsubscribeBets { payload: Vec<i64> },
        SubscribeAllBets,
        UnsubscribeAllBets,
        Ping,
        NewClientSeed { seed: String },
        NewServerSeed,
        MakeBet(PropagatedBet),
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
        pub partner_wallet: String,
        pub user_wallet: String,
        pub site_id: i64,
        pub sub_id: i64,
        pub signature: String,
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

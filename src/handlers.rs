// use std::collections::HashSet;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;

// use crate::communication::WsDataFeedReceiver;
// use crate::config;
use crate::db::DB;
use crate::errors::ApiError;
// #[allow(unused_imports)]
// use crate::models::db_models::{
//     GameInfo, Leaderboard, Nickname, Partner, PartnerProgram, Player, PlayerTotals, RefClicks,
//     Withdrawal,
// };
use crate::models::json_requests::{self, CreateInvoice, Login, RegisterUser};
// #[allow(unused_imports)]
// use crate::models::json_requests::{
//     AddPartnerContacts, AddPartnerSite, AddPartnerSubid, ChangePasswordRequest, ConnectWallet,
//     DeletePartnerContacts, Login, RegisterPartner, SetNickname, SubmitError, SubmitQuestion,
// };
// #[allow(unused_imports)]
use crate::models::db_models::Invoice;
use crate::models::json_responses::{ErrorText, InfoText, JsonResponse, ResponseBody, Status};
// pub use abi::*;
// pub use bets::*;
// pub use block_explorers::*;
// use futures::stream::SplitStream;
// use futures::{SinkExt, StreamExt};
// pub use game::*;
// pub use general::*;
// pub use network::*;
// pub use nickname::*;
// pub use partner::*;
pub use game::*;
pub use invoice::*;
pub use user::*;
// pub use rpcs::*;
use serde::Serialize;
// pub use token::*;
// use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
// use tokio::time::{sleep, Duration};
// use tracing::{debug, error};
use warp::http::StatusCode;
// use warp::ws::{Message, WebSocket};
use std::time::{SystemTime, UNIX_EPOCH};
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

// pub mod network {
//     use super::*;
//     /// Get list of supported networks
//     ///
//     /// Gets a list of all supported networks
//     #[utoipa::path(
//         tag="network",
//         get,
//         path = "/api/network/list",
//         responses(
//             (status = 200, description = "Networks", body = Networks),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_networks(db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let networks = db
//             .query_all_networks()
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         let mut networks_full_info: Vec<NetworkFullInfo> = Vec::with_capacity(networks.len());
//         for network in networks {
//             let network_id = network.network_id;
//             networks_full_info.push(NetworkFullInfo {
//                 basic_info: network,
//                 rpcs: db
//                     .query_all_rpcs(network_id)
//                     .await
//                     .map_err(|e| reject::custom(ApiError::DbError(e)))?,
//                 explorers: db
//                     .query_block_explorers(network_id)
//                     .await
//                     .map_err(|e| reject::custom(ApiError::DbError(e)))?,
//             });
//         }

//         Ok(gen_arbitrary_response(ResponseBody::Networks(Networks {
//             networks: networks_full_info,
//         })))
//     }
// }

// pub mod rpcs {
//     use super::*;
//     /// Get list of rpcs for the network
//     ///
//     /// Gets a list of rpcs for a chosen network
//     #[utoipa::path(
//         tag="rpcs",
//         get,
//         path = "/api/rpc/get/{network_id}",
//         responses(
//             (status = 200, description = "Rpcs", body = Rpcs),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("network_id" = i64, Path, description = "Chain ID of the network")
//         ),
//     )]
//     pub async fn get_rpcs(network_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let rpcs = db
//             .query_all_rpcs(network_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Rpcs(Rpcs { rpcs })))
//     }
// }

// pub mod block_explorers {
//     use super::*;
//     /// Get list of block explorers for the network
//     ///
//     /// Gets a list of block explorers for a chosen network
//     #[utoipa::path(
//         tag="block explorers",
//         get,
//         path = "/api/block_explorer/get/{network_id}",
//         responses(
//             (status = 200, description = "Block explorers", body = BlockExplorers),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("network_id" = i64, Path, description = "Chain ID of the network")
//         ),
//     )]
//     pub async fn get_block_explorers(
//         network_id: i64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let explorers = db
//             .query_block_explorers(network_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::BlockExplorers(
//             BlockExplorers { explorers },
//         )))
//     }

//     pub async fn get_all_explorers(db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let explorers = db
//             .query_all_block_explorers()
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::BlockExplorers(
//             BlockExplorers { explorers },
//         )))
//     }
// }

// pub mod token {
//     use super::*;
//     /// Get list of tokens for the network
//     ///
//     /// Gets a list of tokens for a chosen network
//     #[utoipa::path(
//         tag="token",
//         get,
//         path = "/api/token/get/{network_id}",
//         responses(
//             (status = 200, description = "Tokens", body = Tokens),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("network_id" = i64, Path, description = "Chain ID of the network")
//         ),
//     )]
//     pub async fn get_tokens(network_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let tokens = db
//             .query_all_tokens(network_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Tokens(Tokens {
//             tokens,
//         })))
//     }

//     #[utoipa::path(
//         tag="token",
//         get,
//         path = "/api/token/price/{token_name}",
//         responses(
//             (status = 200, description = "Token price", body = TokenPrice),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("token_name" = String, Path, description = "Name of the token, always uppercase")
//         ),
//     )]
//     pub async fn get_token_price(
//         token_name: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let price = db
//             .query_token_price(&token_name)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .map_or(0.0f64, |price| price.price);

//         Ok(gen_arbitrary_response(ResponseBody::TokenPrice(
//             TokenPrice { token_price: price },
//         )))
//     }
// }

// pub mod game {
//     use super::*;
//     /// Get game info
//     ///
//     /// Gets a game info for the specified network
//     #[utoipa::path(
//         tag="game",
//         get,
//         path = "/api/game/get/{network_id}/{game_name}",
//         responses(
//             (status = 200, description = "Game", body = GameInfo),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("network_id" = i64, Path, description = "Chain ID of the network"),
//             ("game_name" = String, Path, description = "Name of the game")
//         ),
//     )]
//     pub async fn get_game(
//         network_id: i64,
//         game_name: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let game = db
//             .query_game(network_id, &game_name)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .ok_or(reject::custom(ApiError::GameDoesntExist(
//                 network_id, game_name,
//             )))?;

//         Ok(gen_arbitrary_response(ResponseBody::Game(game)))
//     }

//     pub async fn get_game_by_id(game_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let game = db
//             .query_game_by_id(game_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .ok_or(reject::custom(ApiError::GameWithIDDoesntExist(game_id)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Game(game)))
//     }
// }

// pub mod nickname {
//     use super::*;
//     /// Get player nickname
//     ///
//     /// Gets nickname of the player with address
//     #[utoipa::path(
//         tag="nickname",
//         get,
//         path = "/api/player/nickname/get/{address}",
//         responses(
//             (status = 200, description = "Nickname", body = Nickname),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "Address of the player")
//         ),
//     )]
//     pub async fn get_nickname(address: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let nickname = db
//             .query_nickname(&address)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .unwrap_or({
//                 debug!("Nickname for an address `{}` wasn't found", address);
//                 Nickname {
//                     id: 0,
//                     address: address.clone(),
//                     nickname: address,
//                 }
//             });

//         Ok(gen_arbitrary_response(ResponseBody::Nickname(nickname)))
//     }

//     /// Set player nickname
//     ///
//     /// Sets player request, requires signed signature from the user
//     #[utoipa::path(
//         tag="nickname",
//         post,
//         path = "/api/player/nickname/set",
//         request_body = SetNickname,
//         responses(
//             (status = 200, description = "Nickname was set", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn set_nickname(
//         credentials: json_requests::SetNickname,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.set_nickname(&credentials.address, &credentials.nickname)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("The nickname has been changed"))
//     }
// }

// pub mod player {
//     use crate::models::json_responses::LatestGames;

//     use super::*;
//     /// Get user by address
//     ///
//     /// Gets user info by user's address
//     #[utoipa::path(
//         tag="player",
//         get,
//         path = "/api/player/get/{address}",
//         responses(
//             (status = 200, description = "User info", body = Player),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "User address")
//         ),
//     )]
//     pub async fn get_player(address: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let player = db
//             .query_player(&address)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .unwrap_or_else(|| {
//                 //debug!("Player with address `{}` wasn't foung", address);
//                 Default::default()
//             });

//         Ok(gen_arbitrary_response(ResponseBody::Player(player)))
//     }

//     /// Get user totals
//     ///
//     /// Gets user's statistics
//     #[utoipa::path(
//         tag="player",
//         get,
//         path = "/api/player/totals/{address}",
//         responses(
//             (status = 200, description = "User statistics", body = PlayerTotals),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "User address")
//         ),
//     )]
//     pub async fn get_player_totals(
//         address: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let totals = db
//             .query_player_totals(&address)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::PlayerTotals(totals)))
//     }

//     /// Get latest games of the user
//     ///
//     /// Gets 2 latest games played by a user
//     #[utoipa::path(
//         tag="player",
//         get,
//         path = "/api/player/latest_games/{address}",
//         responses(
//             (status = 200, description = "Latest games", body = LatestGames),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "User address")
//         ),
//     )]
//     pub async fn get_latest_games(
//         address: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let latest_games = db
//             .get_latest_games(&address)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::LatestGames(
//             LatestGames {
//                 games: latest_games,
//             },
//         )))
//     }

//     /// Subscribe to the referal owner
//     ///
//     /// Subscribes to the owner of the referal wallet
//     #[utoipa::path(
//         tag="referal",
//         get,
//         path = "/api/player/referal/subscribe",
//         responses(
//             (status = 200, description = "Ok", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//         ),
//     )]
//     pub async fn create_referal(
//         data: json_requests::CreateReferal,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         if data.refer_to.to_lowercase() == data.referal.to_lowercase() {
//             return Err(reject::custom(ApiError::ArbitraryError(
//                 "Referer and referal are the same".into(),
//             )));
//         }
//         db.create_referal(&data.refer_to, &data.referal)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Referal has been created"))
//     }
// }

// pub mod bets {
//     use super::*;
//     /// Get player bets
//     ///
//     /// Gets bets of the player by player address, max amount of returned bets per call is 10
//     #[utoipa::path(
//         tag="bets",
//         get,
//         path = "/api/bets/player/{address}/{last_id}",
//         responses(
//             (status = 200, description = "User's bets", body = Bets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "User address"),
//             ("last_id" = Option<i64>, Path, description = "last bet id")
//         ),
//     )]
//     pub async fn get_player_bets(
//         address: String,
//         last_id: Option<i64>,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_bets_for_address(&address, last_id, *config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }

//     /// Get player bets in increasing order
//     ///
//     /// Gets bets of the player by player address, max amount of returned bets per call is 10. Bets are returned in increasing order.
//     #[utoipa::path(
//         tag="bets",
//         get,
//         path = "/api/bets/player/inc/{address}/{last_id}",
//         responses(
//             (status = 200, description = "User's bets", body = Bets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("address" = String, Path, description = "User address"),
//             ("last_id" = Option<i64>, Path, description = "last bet id")
//         ),
//     )]
//     pub async fn get_player_bets_inc(
//         address: String,
//         first_id: Option<i64>,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_bets_for_address_inc(&address, first_id, *config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }

//     pub async fn get_game_bets(game_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_bets_for_game(game_id, *config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }

//     /// Get all last bets for a game
//     ///
//     /// Gets 10 of the latest bets from the game
//     #[utoipa::path(
//         tag="bets",
//         get,
//         path = "/api/bets/game/{game_name}",
//         responses(
//             (status = 200, description = "Bets", body = Bets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("game_name" = String, Path, description = "Name of the game")
//         ),
//     )]
//     pub async fn get_bets_for_game(
//         game_name: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_bets_for_game_name(&game_name, *config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }

//     pub async fn get_network_bets(
//         netowork_id: i64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_bets_for_network(netowork_id, *config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }

//     /// Get all last bets
//     ///
//     /// Gets 10 of the latest bets from all networks for all games
//     #[utoipa::path(
//         tag="bets",
//         get,
//         path = "/api/bets/list",
//         responses(
//             (status = 200, description = "Bets", body = Bets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_all_last_bets(db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let bets = db
//             .query_all_latest_bets(*config::PAGE_SIZE)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
//     }
// }

// pub mod abi {
//     use super::*;
//     pub async fn get_abi(signature: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let abi = db
//             .query_abi(&signature)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Abi(abi)))
//     }
// }

pub mod game {
    use std::{
        net::SocketAddr,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    use crate::{
        config::PASSWORD_SALT,
        models::db_models::Bet,
        tools::{self, blake_hash_256},
        ChannelType, EngineBetSender, WsData, WsDataFeedReceiver, WsEventSender, WsManagerEvent,
        WsManagerEventSender,
    };

    use self::json_requests::WebsocketsIncommingMessage;

    use super::*;
    use crate::jwt::Payload;
    use crate::tools::blake_hash;
    use base64::{engine::general_purpose, Engine};
    use futures::{
        stream::{self, SplitStream},
        SinkExt, StreamExt,
    };
    use game::json_requests::PropagatedBet;
    use tokio::{sync::mpsc::unbounded_channel, time::sleep};
    use tracing::{debug, error};
    use warp::filters::ws::{Message, WebSocket};

    pub async fn websockets_reader(
        mut socket: SplitStream<WebSocket>,
        event_propagation: WsEventSender,
        _db: DB,
        run: Arc<AtomicBool>,
    ) {
        while run.load(Ordering::Relaxed) {
            let message = socket.next().await;
            match message {
                Some(m) => match m {
                    Ok(message) => {
                        if let Ok(message) = message.to_str() {
                            let message: WebsocketsIncommingMessage =
                                match serde_json::from_str(message) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        error!(
                                            "Error parsing message `{}` | Error: {:?}",
                                            message, e
                                        );
                                        continue;
                                    }
                                };

                            if let Err(e) = event_propagation.send(message) {
                                error!("Error propagating message {:?}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error on a websocket: {:?}", e);
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
    }

    async fn auth(db: &DB, token: &str) -> Result<i64, ApiError> {
        let parts = token.split('.').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(ApiError::MalformedToken);
        }
        let decoded = serde_json::from_str::<Payload>(
            std::str::from_utf8(
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
            .map_err(|e| ApiError::DbError(e))?
            .ok_or(ApiError::ArbitraryError(
                "Wrong username or password".into(),
            ))?;
        let _token_serialized = tools::serialize_token(
            &token,
            &format!("{}{}{}", *PASSWORD_SALT, user.password, decoded.iat),
        )
        .map_err(|_| ApiError::MalformedToken)?;

        Ok(user.id)
    }

    pub async fn websockets_handler(
        socket: WebSocket,
        address: SocketAddr,
        db: DB,
        manager_writer: WsManagerEventSender,
        engine_sender: EngineBetSender,
    ) {
        let (data_feed_tx, mut data_feed) = unbounded_channel();

        manager_writer
            .send(WsManagerEvent::SubscribeFeed {
                id: address.ip(),
                feed: data_feed_tx,
            })
            .unwrap();
        debug!("New connection {:?}", &socket);
        let (mut ws_tx, ws_rx) = socket.split();

        let (reader_tx, mut reader_rx) = unbounded_channel();

        let run = Arc::new(AtomicBool::new(true));
        tokio::spawn(websockets_reader(ws_rx, reader_tx, db.clone(), run.clone()));

        let mut events = Vec::<WsData>::with_capacity(10);

        let mut user_id: Option<i64> = None;

        while run.load(Ordering::Relaxed) {
            tokio::select! {
                    events_amount = data_feed.recv_many(&mut events, 10) => {
                        if events_amount == 0{
                            break;

                        }else{
                            for event in events.iter() {
                                let e: ResponseBody = event.into();
                                ws_tx.start_send_unpin(Message::text(serde_json::to_string(&e).unwrap())).unwrap();
                            }
                            events.clear();
                            ws_tx.flush().await;
                        }
                    }
                    _ = sleep(Duration::from_millis(5000)) => {
                        ws_tx
                            .send(Message::text(serde_json::to_string(&WebsocketsIncommingMessage::Ping).unwrap()))
                            .await
                            .unwrap();

                    }

                    msg = reader_rx.recv() => {
                        match msg{
                            Some(msg) => {
                                match msg{
                                    WebsocketsIncommingMessage::Auth { token } => {
                                        if user_id.is_none(){
                                            match auth(&db, &token).await{
                                                Ok(id) => {
                                                    user_id.replace(id);
                                                    if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::InfoText(InfoText { message: "Logined successfully".into() })).unwrap())).await{
                                                        error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                        break;
                                                    }
                                                },
                                                Err(_) => break
                                            }
                                        }
                                    },
                                    WebsocketsIncommingMessage::SubscribeBets { payload } => {
                                        for id in payload{
                                            if let Err(_) = manager_writer.send(WsManagerEvent::SubscribeChannel { id: address.ip(), channel: ChannelType::Bets(id) }){
                                                break;
                                            }
                                        }
                                    },
                                    WebsocketsIncommingMessage::UnsubscribeBets { payload } => {
                                        for id in payload{
                                            if let Err(_) = manager_writer.send(WsManagerEvent::UnsubscribeChannel { id: address.ip(), channel: ChannelType::Bets(id) }){
                                                break;
                                            }
                                        }
                                    },
                                    WebsocketsIncommingMessage::SubscribeAllBets => {},
                                    WebsocketsIncommingMessage::UnsubscribeAllBets => {},
                                    WebsocketsIncommingMessage::Ping => {
                                        if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&WebsocketsIncommingMessage::Ping).unwrap())).await{
                                            error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                            break;
                                        }
                                    },
                                    WebsocketsIncommingMessage::NewClientSeed { seed } => {
                                        let seed = blake_hash_256(&seed);
                                        if let Some(user_id) = user_id {
                                            if let Err(e) = db.new_user_seed(user_id, &seed).await {
                                                if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::ErrorText(ErrorText { error: format!("{:?}",e) })).unwrap())).await{
                                                    error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                    break;
                                                }
                                            }
                                        }

                                    },
                                    WebsocketsIncommingMessage::NewServerSeed => {
                                        if let Some(user_id) = user_id {
                                            let seed_hex = blake_hash(&format!("{}{}{}",user_id,chrono::offset::Utc::now(),*PASSWORD_SALT));
                                            let mut seed_hex_chars = seed_hex.chars();
                                            seed_hex_chars.next();
                                            seed_hex_chars.next();
                                            let seed = seed_hex_chars.as_str();
                                            let _ = db.reveal_last_seed(user_id).await;
                                            if let Err(e) = db.new_server_seed(user_id, seed).await{
                                                if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::ErrorText(ErrorText { error: format!("{:?}",e) })).unwrap())).await{
                                                    error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                    break;
                                                }
                                            }
                                            let seed_hex = blake_hash(seed);
                                            let mut seed_hex_chars = seed_hex.chars();
                                            seed_hex_chars.next();
                                            seed_hex_chars.next();
                                            let seed = seed_hex_chars.as_str();
                                            if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::ServerSeedHidden { seed: seed.into() }).unwrap())).await{
                                                error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                break;
                                            }
                                        }
                                    },
                                    WebsocketsIncommingMessage::MakeBet(mut bet) => {
                                        if let Some(user_id) = user_id{
                                            bet.user_id = user_id;
                                            engine_sender.send(bet).await;
                                        }
                                    },
                                }
                            },
                            None => {
                                break;
                            },
                        }

                    }
            }
        }

        manager_writer
            .send(WsManagerEvent::UnsubscribeFeed(address.ip()))
            .unwrap();
    }
}

pub mod invoice {
    use self::json_requests::QrRequest;
    use crate::tools::blake_hash;
    use qrcode_generator::QrCodeEcc;
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use thedex::{models::CreateInvoice as CreateInvoiceRequest, TheDex};

    use super::*;
    /// Create a new invoice
    ///
    /// Creates a new invoice
    #[utoipa::path(
        tag="invoice",
        post,
        path = "/api/invoice/create",
        request_body = CreateInvoice,
        responses(
            (status = 200, description = "User account was created", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
    pub async fn create_invoice(
        data: CreateInvoice,
        id: i64,
        db: DB,
        dex: TheDex,
    ) -> Result<WarpResponse, warp::Rejection> {
        // let order_id = blake_hash(&format!(
        //     "{}{}{}{}",
        //     id,
        //     data.amount.clone() as u32,
        //     data.currency,
        //     chrono::offset::Utc::now().timestamp_millis()
        // ));
        // let result = dex
        //     .create_invoice(
        //         CreateInvoiceRequest {
        //             amount: Decimal::from_u32(data.amount.clone() as u32).unwrap(),
        //             currency: data.currency,
        //             merchant_id: String::new(),
        //             order_id: Some(order_id),
        //             email: None,
        //             client_id: Some(id.to_string()),
        //             title: Some(format!("Bying for ${}", data.amount as u32)),
        //             description: None,
        //             recalculation: Some(true),
        //             needs_email_confirmation: Some(false),
        //             success_url: Some(String::from(
        //                 "https://game.greekkeepers.io/api/invoice/success",
        //             )),
        //             failure_url: Some(String::from(
        //                 "https://game.greekkeepers.io/api/invoice/failure",
        //             )),
        //             callback_url: Some(String::from(
        //                 "https://game.greekkeepers.io/api/invoice/callback",
        //             )),
        //         },
        //         chrono::offset::Utc::now().timestamp_millis() as u64,
        //     )
        //     .await
        //     .map_err(ApiError::CreateInvoiceError)?;
        Ok(gen_arbitrary_response(ResponseBody::Invoice(
            Default::default(),
        )))
    }

    /// Generate qr code
    ///
    /// Generates qr code from the specified data
    #[utoipa::path(
        tag="invoice",
        get,
        path = "/api/invoice/qr",
        request_body = QrRequest,
        responses(
            (status = 200, description = "User account was created", body = Invoice),
            (status = 500, description = "Internal server error", body = ErrorText),
        ),
    )]
    pub async fn generate_qr(
        data: QrRequest,
        id: i64,
        db: DB,
    ) -> Result<WarpResponse, warp::Rejection> {
        Ok(get_pgn_response(
            qrcode_generator::to_png_to_vec(&data.data, QrCodeEcc::Low, 1024)
                .map_err(|_| ApiError::QrGenerationError(data.data))?,
        ))
    }
}

pub mod user {
    use crate::jwt;
    use crate::models::json_responses::{Amounts, UserStripped};
    use crate::tools::blake_hash;
    use crate::{config::PASSWORD_SALT, models::json_responses::AccessToken};
    use blake2::{Blake2b512, Digest};
    use chrono::{TimeZone, Utc};
    use hex::ToHex;
    use rust_decimal::prelude::FromPrimitive;
    use sqlx::types::BigDecimal;
    use tracing::debug;

    use self::json_requests::ChangeNickname;

    use super::*;

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
    ) -> Result<WarpResponse, warp::Rejection> {
        let mut hasher = Blake2b512::new();

        hasher.update(data.password.as_bytes());

        let res: String = hasher.finalize().encode_hex();

        debug!("res {:?}", res);
        let user = db
            .register_user(&data.username, &res)
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?;

        let coins = db
            .fetch_coins()
            .await
            .map_err(|e| reject::custom(ApiError::DbError(e)))?;

        debug!("Coins {:?}", coins);

        for coin in coins {
            if coin.id == 1 {
                db.init_amount(user.id, coin.id, BigDecimal::from_u64(1000).unwrap())
                    .await
                    .map_err(|e| reject::custom(ApiError::DbError(e)))?;
            } else {
                db.init_amount(user.id, coin.id, BigDecimal::from_u64(0).unwrap())
                    .await
                    .map_err(|e| reject::custom(ApiError::DbError(e)))?;
            }
        }

        Ok(gen_info_response("User account has been created"))
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

        let token = jwt::generate_token(
            &jwt::Payload {
                iss: None,
                sub: user.id,
                exp: 100,
                iat: iat,
                aud: "".into(),
            },
            &format!("{}{}{}", *PASSWORD_SALT, hashed_password, iat),
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

    /// Get user info
    ///
    ///
    #[utoipa::path(
        tag="user",
        get,
        path = "/api/user/{user_id}",
        responses(
            (status = 200, description = "Coins amounts", body = UserStripped),
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
}

// pub mod partner {

//     use crate::config::PASSWORD_SALT;
//     use crate::jwt;
//     use crate::models::db_models::{PlayersTotals, TimeBoundaries};
//     use crate::models::json_requests::WithdrawRequest;
//     use crate::models::json_responses::{
//         ClicksTimeMapped, ConnectedWalletInfo, ConnectedWalletsTimeMapped, PartnerInfo,
//         PartnerSiteInfo,
//     };
//     use crate::tools::blake_hash;
//     use blake2::{Blake2b512, Digest};
//     use chrono::{TimeZone, Utc};
//     use hex::ToHex;

//     use super::*;

//     /// Register new partner account
//     ///
//     /// Registers new partner account, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/register",
//         request_body = RegisterPartner,
//         responses(
//             (status = 200, description = "Partner account was created", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn register_partner(
//         data: json_requests::RegisterPartner,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let mut hasher = Blake2b512::new();

//         hasher.update(data.password.as_bytes());

//         let res = hasher.finalize().encode_hex();

//         debug!("res {:?}", res);
//         db.create_partner(
//             Partner {
//                 name: data.name,
//                 country: data.country,
//                 traffic_source: data.traffic_source,
//                 users_amount_a_month: data.users_amount_a_month,
//                 main_wallet: data.main_wallet,
//                 program: PartnerProgram::firstMonth,
//                 is_verified: false,
//                 login: data.login,
//                 password: res,
//                 registration_time: Default::default(),
//                 language: data.language,
//             },
//             &[],
//         )
//         .await
//         .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Partner account has been created"))
//     }

//     /// Submit question
//     ///
//     /// Submits question to be answered later
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/question",
//         request_body = SubmitQuestion,
//         responses(
//             (status = 200, description = "Partner account was created", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn submit_question(
//         data: json_requests::SubmitQuestion,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.submit_question(&data.name, &data.email, &data.message)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Contacts were added"))
//     }

//     /// Adds contacts to the account
//     ///
//     /// Adds contact info to the existinf partner account, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/contacts/add",
//         request_body = AddPartnerContacts,
//         responses(
//             (status = 200, description = "Partner account was created", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn add_contacts(
//         wallet: String,
//         data: json_requests::AddPartnerContacts,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.add_partner_contacts(
//             &wallet,
//             &data
//                 .contacts
//                 .into_iter()
//                 .map(|c| (c.name, c.url))
//                 .collect::<Vec<(String, String)>>(),
//         )
//         .await
//         .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Contacts were added"))
//     }

//     /// Submits a new withdrawal request
//     ///
//     /// Submits a new withdrawal request
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/withdraw",
//         request_body = WithdrawRequest,
//         responses(
//             (status = 200, description = "Withdraw request was submitted", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn submit_withdrawal(
//         wallet: String,
//         data: WithdrawRequest,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.create_withdraw_request(&wallet, &data)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Contacts were added"))
//     }

//     /// Adds new site to the partner
//     ///
//     /// Adds new site instaance to the partner account, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/site/add",
//         request_body = AddPartnerSite,
//         responses(
//             (status = 200, description = "Site was added", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn add_partner_site(
//         wallet: String,
//         data: json_requests::AddPartnerSite,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.add_partner_site(&wallet, &data.url, &data.name, &data.language)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Site was added"))
//     }

//     /// Adds new subb id
//     ///
//     /// Adds new sub id to the existing site on partner's account, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/site/subid/add",
//         request_body = AddPartnerSubid,
//         responses(
//             (status = 200, description = "SubId was added", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn add_partner_subid(
//         wallet: String,
//         data: json_requests::AddPartnerSubid,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.add_partner_subid(data.internal_site_id, &wallet, &data.url, &data.name)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Sub id was added"))
//     }

//     /// Adds click to subid
//     ///
//     /// Adds click to sub id of the user's site
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/site/subid/click/{partner_address}/{site_id}/{sub_id}",
//         responses(
//             (status = 200, description = "Click was accepted", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("partner_address" = String, Path, description = "ETH address of the partner's account"),
//             ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
//             ("sub_id" = i64, Path, description = "Relative subid ofthe site, registered on partner's account"),
//         ),
//     )]
//     pub async fn click_partner_subid(
//         wallet: String,
//         site_id: i64,
//         sub_id: i64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let subid = db
//             .get_subid(&wallet, site_id, sub_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         db.add_click(&wallet, subid.internal_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Click was successfully added"))
//     }

//     /// Connects new wallet with the given subid of the partner
//     ///
//     /// Connects new wallet with the given subid of the partner, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/site/subid/connect",
//         request_body = ConnectWallet,
//         responses(
//             (status = 200, description = "Wallet was connected", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn connect_wallet(
//         data: json_requests::ConnectWallet,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let time = chrono::offset::Utc::now();
//         let subid = db
//             .get_subid(&data.partner_wallet, data.site_id, data.sub_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         db.add_ref_wallet(
//             &data.user_wallet,
//             time,
//             subid.internal_id,
//             &data.partner_wallet,
//         )
//         .await
//         .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Wallet was successfully connected"))
//     }

//     /// Gets partner account info
//     ///
//     /// Gets all basic info about partner account, requires signed signature from the user
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/get",
//         responses(
//             (status = 200, description = "Partner account was created", body = PartnerInfo),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_partner(wallet: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let basic = db
//             .get_partner(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;
//         let contacts = db
//             .get_partner_contacts(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         let sites = db
//             .get_partner_sites(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;
//         let mut sites_info: Vec<PartnerSiteInfo> = Vec::with_capacity(sites.len());
//         for site in sites {
//             let sub_ids = db
//                 .get_site_subids(site.internal_id)
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?;
//             sites_info.push(PartnerSiteInfo {
//                 basic: site,
//                 sub_ids,
//             })
//         }

//         Ok(gen_arbitrary_response(ResponseBody::PartnerInfo(
//             PartnerInfo {
//                 basic,
//                 contacts,
//                 sites: sites_info,
//             },
//         )))
//     }

//     /// Gets partner contacts
//     ///
//     /// Gets all contacts of the user
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/contacts/get",
//         responses(
//             (status = 200, description = "Partner account was created", body = PartnerContact),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_partner_contacts(
//         wallet: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let contacts = db
//             .get_partner_contacts(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::PartnerContacts(
//             contacts,
//         )))
//     }

//     /// Gets amount of connected wallets
//     ///
//     /// Gets amount of wallets that connected to the partner
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/connected/{time_boundaries}",
//         responses(
//             (status = 200, description = "Connected wallets", body = AmountConnectedWallets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
//         ),
//     )]
//     pub async fn get_partner_connected_wallets(
//         wallet: String,
//         time_boundaries: TimeBoundaries,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let connected_wallets = db
//             .get_partner_connected_wallets_amount(&wallet, time_boundaries)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(
//             ResponseBody::AmountConnectedWallets(connected_wallets),
//         ))
//     }

//     /// Gets amount of connected wallets that made deposits
//     ///
//     /// Gets amount of wallets that connected to the partner and made at least one bet
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/connected_betted/{time_boundaries}",
//         responses(
//             (status = 200, description = "Connected wallets", body = AmountConnectedWallets),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
//         ),
//     )]
//     pub async fn get_partner_connected_wallets_with_deposits_amount(
//         wallet: String,
//         time_boundaries: TimeBoundaries,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let connected_wallets = db
//             .get_partner_connected_wallets_with_deposits_amount(&wallet, time_boundaries)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(
//             ResponseBody::AmountConnectedWallets(connected_wallets),
//         ))
//     }

//     /// Gets connected wallets
//     ///
//     /// Gets wallets that connected to the partner
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/wallets/{time_boundaries}",
//         responses(
//             (status = 200, description = "Connected wallets", body = ConnectedWalletInfo),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch connected wallets"),
//         ),
//     )]
//     pub async fn get_partner_connected_wallets_info(
//         wallet: String,
//         time_boundaries: TimeBoundaries,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let connected_wallets = db
//             .get_partner_connected_wallets_info(&wallet, time_boundaries)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         let mut connected_wallets_stats: Vec<ConnectedWalletInfo> =
//             Vec::with_capacity(connected_wallets.len());

//         for wallet in connected_wallets {
//             let stats = db
//                 .query_player_totals(&wallet.address)
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//             connected_wallets_stats.push(ConnectedWalletInfo {
//                 id: wallet.id,
//                 address: wallet.address,
//                 timestamp: wallet.timestamp,
//                 site_id: wallet.site_id,
//                 sub_id: wallet.sub_id,
//                 bets_amount: stats.bets_amount,
//                 lost_bets: stats.lost_bets,
//                 won_bets: stats.won_bets,
//                 total_wagered_sum: stats.total_wagered_sum,
//                 gross_profit: stats.gross_profit,
//                 net_profit: stats.net_profit,
//                 highest_win: stats.highest_win,
//             });
//         }

//         Ok(gen_arbitrary_response(ResponseBody::ConnectedWallets(
//             connected_wallets_stats,
//         )))
//     }

//     /// Gets amount of connected wallets
//     ///
//     /// Gets amount of wallets that connected to the partner, withing specified time boundaries
//     /// time boundaries are specified as UNIX timestamps un UTC
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/connected/{start}/{end}/{step}",
//         responses(
//             (status = 200, description = "Connected wallets", body = ConnectedWalletsTimeMapped),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("start" = u64, Path, description = "Starting timestamp for the search"),
//             ("end" = u64, Path, description = "Ending timestamp for the search"),
//             ("step" = u64, Path, description = "Step from start to end"),
//         ),
//     )]
//     pub async fn get_partner_connected_wallets_exact_date(
//         wallet: String,
//         begin: u64,
//         end: u64,
//         step: u64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let capacity = ((end - begin) / step) as usize;
//         if capacity > 100 {
//             return Err(reject::custom(ApiError::BadRange));
//         }
//         let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

//         for start in (begin..end).step_by(step as usize) {
//             connected_wallets.push(
//                 db.get_partner_connected_wallets_amount_exact_date(
//                     &wallet,
//                     Utc.timestamp_opt(start as i64, 0).unwrap(),
//                     Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
//                 )
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?
//                 .connected_wallets,
//             );
//         }

//         Ok(gen_arbitrary_response(
//             ResponseBody::AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped {
//                 amount: connected_wallets,
//             }),
//         ))
//     }

//     /// Gets amount of connected wallets with bets
//     ///
//     /// Gets amount of wallets that connected to the partner and made at least one bet, withing specified time boundaries
//     /// time boundaries are specified as UNIX timestamps un UTC
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/connected_betted/betted/{start}/{end}/{step}",
//         responses(
//             (status = 200, description = "Connected wallets", body = ConnectedWalletsTimeMapped),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("start" = u64, Path, description = "Starting timestamp for the search"),
//             ("end" = u64, Path, description = "Ending timestamp for the search"),
//             ("step" = u64, Path, description = "Step from start to end"),
//         ),
//     )]
//     pub async fn get_partner_connected_wallets_betted_exact_date(
//         wallet: String,
//         begin: u64,
//         end: u64,
//         step: u64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let capacity = ((end - begin) / step) as usize;
//         if capacity > 100 {
//             return Err(reject::custom(ApiError::BadRange));
//         }
//         let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

//         for start in (begin..end).step_by(step as usize) {
//             connected_wallets.push(
//                 db.get_partner_connected_wallets_with_bets_amount_exact_date(
//                     &wallet,
//                     Utc.timestamp_opt(start as i64, 0).unwrap(),
//                     Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
//                 )
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?
//                 .connected_wallets,
//             );
//         }

//         Ok(gen_arbitrary_response(
//             ResponseBody::AmountConnectedWalletsTimeMapped(ConnectedWalletsTimeMapped {
//                 amount: connected_wallets,
//             }),
//         ))
//     }

//     /// Gets totals for the partner
//     ///
//     /// Gets totals on lost bets of the connected wallets
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/connected/totals",
//         responses(
//             (status = 200, description = "Totals", body = PlayersTotals),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_connected_totals(
//         wallet: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let totals: PlayersTotals = db
//             .query_players_totals(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::PlayersTotals(totals)))
//     }

//     /// Gets withdrawal history
//     ///
//     /// Gets withdrawals of the partner
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/withdrawals/{time_boundaries}",
//         responses(
//             (status = 200, description = "Totals", body = WithdrawRequest),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch withdrawal requests"),
//         ),
//     )]
//     pub async fn get_withdrawal_requests(
//         wallet: String,
//         time_boundaries: TimeBoundaries,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let withdrawals = db
//             .get_partner_withdrawal_requests(&wallet, time_boundaries)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Withdrawals(
//             withdrawals,
//         )))
//     }

//     /// Gets amount of clicks
//     ///
//     /// Gets amount of click for the partner links, within specified time boundaries
//     /// time boundaries are specified as UNIX timestamps un UTC
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/clicks/{start}/{end}/{step}",
//         responses(
//             (status = 200, description = "Clicks", body = ClicksTimeMapped),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("start" = u64, Path, description = "Starting timestamp for the search"),
//             ("end" = u64, Path, description = "Ending timestamp for the search"),
//             ("step" = u64, Path, description = "Step from start to end"),
//         ),
//     )]
//     pub async fn get_partner_clicks_exact_date(
//         wallet: String,
//         begin: u64,
//         end: u64,
//         step: u64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let capacity = ((end - begin) / step) as usize;
//         if capacity > 100 {
//             return Err(reject::custom(ApiError::BadRange));
//         }
//         let mut connected_wallets: Vec<i64> = Vec::with_capacity(capacity);

//         for start in (begin..end).step_by(step as usize) {
//             connected_wallets.push(
//                 db.get_partner_clicks_exact_date(
//                     &wallet,
//                     Utc.timestamp_opt(start as i64, 0).unwrap(),
//                     Utc.timestamp_opt((start + step) as i64, 0).unwrap(),
//                 )
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?
//                 .clicks,
//             );
//         }

//         Ok(gen_arbitrary_response(
//             ResponseBody::AmountClicksTimeMapped(ClicksTimeMapped {
//                 amount: connected_wallets,
//             }),
//         ))
//     }

//     /// Gets partner sites
//     ///
//     /// Gets all sites of the user
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/site/get",
//         responses(
//             (status = 200, description = "Partner's site", body = PartnerSiteInfo),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_partner_sites(
//         wallet: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let sites = db
//             .get_partner_sites(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;
//         let mut sites_info: Vec<PartnerSiteInfo> = Vec::with_capacity(sites.len());
//         for site in sites {
//             let sub_ids = db
//                 .get_site_subids(site.internal_id)
//                 .await
//                 .map_err(|e| reject::custom(ApiError::DbError(e)))?;
//             sites_info.push(PartnerSiteInfo {
//                 basic: site,
//                 sub_ids,
//             })
//         }

//         Ok(gen_arbitrary_response(ResponseBody::PartnerSiteInfo(
//             sites_info,
//         )))
//     }

//     /// Remove partner contacts
//     ///
//     /// Gets all contacts of the user
//     #[utoipa::path(
//         tag="partner",
//         delete,
//         path = "/api/partner/contacts/delete",
//         responses(
//             (status = 200, description = "Partner contact was removed", body = DeletePartnerContacts),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn delete_partner_contacts(
//         wallet: String,
//         contacts: json_requests::DeletePartnerContacts,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         db.delete_partner_contacts(&wallet, &contacts.contacts)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Contact was deleted"))
//     }

//     /// Gets clicks for the subid
//     ///
//     /// Gets all the clicks accumulated for subid
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/site/subid/clicks/{site_id}/{sub_id}",
//         responses(
//             (status = 200, description = "Partner's subid clicks", body = RefClicks),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
//             ("sub_id" = i64, Path, description = "Relative subid ofthe site, registered on partner's account"),
//         ),
//     )]
//     pub async fn get_clicks(
//         wallet: String,
//         site_id: i64,
//         sub_id: i64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let clicks = db
//             .get_subid_clicks(&wallet, site_id, sub_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
//     }

//     /// Gets clicks for the site
//     ///
//     /// Gets all the clicks accumulated for site
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/site/clicks/{site_id}",
//         responses(
//             (status = 200, description = "Partner's site clicks", body = RefClicks),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("site_id" = i64, Path, description = "Relative id of the site, registered on partner's account"),
//         ),
//     )]
//     pub async fn get_site_clicks(
//         wallet: String,
//         site_id: i64,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let clicks = db
//             .get_site_clicks(&wallet, site_id)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
//     }

//     /// Gets clicks for the partner
//     ///
//     /// Gets all the clicks accumulated for partner
//     #[utoipa::path(
//         tag="partner",
//         get,
//         path = "/api/partner/clicks",
//         responses(
//             (status = 200, description = "Partner's site clicks", body = RefClicks),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_partner_clicks(
//         wallet: String,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let clicks = db
//             .get_partner_clicks(&wallet)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Clicks(clicks)))
//     }

//     /// Change password of the partner
//     ///
//     /// Changes the password of the partner
//     #[utoipa::path(
//         tag="partner",
//         put,
//         path = "/api/partner/change/password",
//         request_body = ChangePasswordRequest,
//         responses(
//             (status = 200, description = "Partner's site clicks", body = InfoText),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn partner_change_password(
//         wallet: String,
//         data: ChangePasswordRequest,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let old_hashed = blake_hash(&data.old_password);
//         let new_hashed = blake_hash(&data.new_password);
//         if !db
//             .partner_change_password(&wallet, &old_hashed, &new_hashed)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//         {
//             return Err(reject::custom(ApiError::BadPassword));
//         }

//         Ok(gen_info_response("Password was changed successfully"))
//     }

//     /// Login partner
//     ///
//     /// Logins partner with provided login/password
//     #[utoipa::path(
//         tag="partner",
//         post,
//         path = "/api/partner/login",
//         request_body = Login,
//         responses(
//             (status = 200, description = "Access token", body = AccessToken),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn login_partner(login: Login, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let hashed_password = blake_hash(&login.password);
//         let partner = db
//             .login_partner(&login.login, &hashed_password)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?
//             .ok_or(reject::custom(ApiError::WrongLoginPassword))?;

//         let token = jwt::generate_token(
//             &jwt::Payload {
//                 iss: None,
//                 sub: partner.login,
//                 exp: 100,
//                 iat: 100,
//                 aud: "".into(),
//             },
//             &format!("{:?}{:?}", *PASSWORD_SALT, hashed_password),
//         );

//         Ok(gen_arbitrary_response(ResponseBody::AccessToken(
//             AccessToken {
//                 access_token: token.clone(),
//                 token_type: "Bearer".into(),
//                 expires_in: 100,
//                 refresh_token: token,
//             },
//         )))
//     }
// }

// pub mod general {
//     use crate::models::{db_models::TimeBoundaries, LeaderboardType};

//     use super::*;

//     pub async fn submit_error(data: SubmitError, db: DB) -> Result<WarpResponse, warp::Rejection> {
//         db.submit_error(&data.error)
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_info_response("Error submitted"))
//     }

//     /// Get totals
//     ///
//     /// Gets total bets, wagered sum, players
//     #[utoipa::path(
//         tag="general",
//         get,
//         path = "/api/general/totals",
//         responses(
//             (status = 200, description = "Totals", body = Totals),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//     )]
//     pub async fn get_totals(db: DB) -> Result<WarpResponse, warp::Rejection> {
//         let totals = db
//             .get_totals()
//             .await
//             .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Totals(totals)))
//     }

//     /// Get leaderboard data
//     ///
//     /// Gets the leaderboard
//     #[utoipa::path(
//         tag="general",
//         get,
//         path = "/api/general/leaderboard/{type}/{time_boundaries}",
//         responses(
//             (status = 200, description = "Leaderboard data, 20 records max", body = Vec<Leaderboard>),
//             (status = 500, description = "Internal server error", body = ErrorText),
//         ),
//         params(
//             ("type" = LeaderboardType, Path, description = "Type of the leaderboard data volume/profit"),
//             ("time_boundaries" = TimeBoundaries, Path, description = "Time boundaries in which to fetch leaderboard info"),
//         ),
//     )]
//     pub async fn get_leaderboard(
//         leaderboard_type: LeaderboardType,
//         time_boundaries: TimeBoundaries,
//         db: DB,
//     ) -> Result<WarpResponse, warp::Rejection> {
//         let leaderboard = match leaderboard_type {
//             LeaderboardType::Volume => db.query_leaderboard_volume(time_boundaries, 20).await,
//             LeaderboardType::Profit => db.query_leaderboard_profit(time_boundaries, 20).await,
//         }
//         .map_err(|e| reject::custom(ApiError::DbError(e)))?;

//         Ok(gen_arbitrary_response(ResponseBody::Leaderboard(
//             leaderboard,
//         )))
//     }
// }

// pub async fn websockets_subscriptions_reader(
//     mut socket: SplitStream<WebSocket>,
//     subscriptions_propagation: UnboundedSender<WebsocketsIncommingMessage>,
//     _db: DB,
//     run: Arc<AtomicBool>,
// ) {
//     while run.load(Ordering::Relaxed) {
//         let message = socket.next().await;
//         match message {
//             Some(m) => match m {
//                 Ok(message) => {
//                     if let Ok(message) = message.to_str() {
//                         let message: WebsocketsIncommingMessage =
//                             match serde_json::from_str(message) {
//                                 Ok(m) => m,
//                                 Err(e) => {
//                                     error!("Error parsing message `{}` | Error: {:?}", message, e);
//                                     continue;
//                                 }
//                             };

//                         if let Err(e) = subscriptions_propagation.send(message) {
//                             error!("Error propagating message {:?}", e);
//                             break;
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     error!("Error on a websocket: {:?}", e);
//                     break;
//                 }
//             },
//             None => {
//                 break;
//             }
//         }
//     }
// }

// pub async fn websockets_handler(socket: WebSocket, db: DB, mut channel: WsDataFeedReceiver) {
//     debug!("New connection {:?}", &socket);
//     let (mut ws_tx, ws_rx) = socket.split();
//     let mut subscriptions: HashSet<String> = Default::default();
//     let mut subscribed_all: bool = false;

//     let (subscriptions_tx, mut subscriptions_rx) = unbounded_channel();

//     let run = Arc::new(AtomicBool::new(true));
//     tokio::spawn(websockets_subscriptions_reader(
//         ws_rx,
//         subscriptions_tx,
//         db,
//         run.clone(),
//     ));

//     loop {
//         tokio::select! {
//             bet = channel.recv() => {
//                 match bet{
//                     Ok(bet) => {
//                         if !subscribed_all && subscriptions.get(&bet.game_name).is_none(){
//                             continue;
//                         }

//                         ws_tx
//                             .send(Message::text(serde_json::to_string(&bet).unwrap()))
//                             .await
//                             .unwrap();
//                     },
//                     Err(e) => {
//                         error!("Error recieving bet {:?}", e);
//                         break;
//                     },
//                 }
//             }
//             _ = sleep(Duration::from_millis(5000)) => {
//                 ws_tx
//                     .send(Message::text(serde_json::to_string(&WebsocketsIncommingMessage::Ping).unwrap()))
//                     .await
//                     .unwrap();
//             }
//             msg = subscriptions_rx.recv() => {
//                 match msg{
//                     Some(subs) => {
//                         match subs{
//                             WebsocketsIncommingMessage::Subscribe{payload: s} => {
//                                 if subscribed_all{
//                                     continue;
//                                 }
//                                 let mut end = 100-subscriptions.len();
//                                 if end > s.len(){
//                                     end = s.len();
//                                 }
//                                 for sub in &s[0..end]{
//                                     subscriptions.insert(sub.clone());

//                                 }
//                             },
//                             WebsocketsIncommingMessage::Unsubscribe{payload: s} => {
//                                 if subscribed_all{
//                                     continue;
//                                 }
//                                 for sub in s {
//                                     subscriptions.remove(&sub);
//                                 }
//                             },
//                             WebsocketsIncommingMessage::SubscribeAll => {
//                                 subscribed_all = true;
//                             },
//                             WebsocketsIncommingMessage::UnsubscribeAll => {
//                                 subscribed_all = false;
//                             },
//                             WebsocketsIncommingMessage::Ping => {}
//                         }
//                     },
//                     None => {
//                         break;
//                     },
//                 }
//             }
//         }
//     }

//     run.store(false, Ordering::Relaxed);
// }

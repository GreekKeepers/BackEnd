
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
    models::json_responses::{Games, Seed, UuidToken},
    tools::{self, blake_hash_256},
    ChannelType, EngineBetSender, WsData, WsDataFeedReceiver, WsDataFeedSender, WsEventSender,
    WsManagerEvent, WsManagerEventSender,
};

use self::json_requests::WebsocketsIncommingMessage;

use super::*;
use crate::jwt::Payload;
use crate::tools::blake_hash;
use base64::{engine::general_purpose, Engine};
use futures::{stream::SplitStream, SinkExt, StreamExt};

use tokio::{sync::mpsc::unbounded_channel, time::sleep};
use tracing::{debug, error};
use uuid::Uuid;
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
                                    error!("Error parsing message `{}` | Error: {:?}", message, e);
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
        .map_err(ApiError::DbError)?
        .ok_or(ApiError::ArbitraryError(
            "Wrong username or password".into(),
        ))?;
    let _token_serialized = tools::serialize_token(
        token,
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
    let (data_feed_tx, mut data_feed): (WsDataFeedSender, WsDataFeedReceiver) = unbounded_channel();

    let uuid = Uuid::new_v4().to_string();

    manager_writer
        .send(WsManagerEvent::SubscribeFeed {
            id: uuid.clone(),
            feed: data_feed_tx,
        })
        .unwrap();
    debug!("New connection {:?}: {:?}", &socket, &uuid);
    let (mut ws_tx, ws_rx) = socket.split();

    let (reader_tx, mut reader_rx) = unbounded_channel();

    let run = Arc::new(AtomicBool::new(true));
    tokio::spawn(websockets_reader(ws_rx, reader_tx, db.clone(), run.clone()));

    let mut events = Vec::<WsData>::with_capacity(10);

    let mut user_id: Option<i64> = None;

    if let Err(e) = ws_tx
        .send(Message::text(
            serde_json::to_string(&ResponseBody::Uuid(UuidToken { uuid: uuid.clone() })).unwrap(),
        ))
        .await
    {
        error!("Error on socket `{:?}`: `{:?}`", ws_tx, e);
    }

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
                        if let Err(_) = ws_tx.flush().await{
                            break;
                        };
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
                            debug!("{:?}: {:?}", &address, &msg);
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
                                            Err(e) => {
                                                error!("Auth error {:?}: {:?}", &address, e);
                                                break
                                            }
                                        }
                                    }
                                },
                                WebsocketsIncommingMessage::SubscribeBets { payload } => {
                                    for id in payload{
                                        if let Err(_) = manager_writer.send(WsManagerEvent::SubscribeChannel { id: uuid.clone(), channel: ChannelType::Bets(id) }){
                                            break;
                                        }
                                    }
                                },
                                WebsocketsIncommingMessage::UnsubscribeBets { payload } => {
                                    for id in payload{
                                        if let Err(_) = manager_writer.send(WsManagerEvent::UnsubscribeChannel { id: uuid.clone(), channel: ChannelType::Bets(id) }){
                                            break;
                                        }
                                    }
                                },
                                WebsocketsIncommingMessage::SubscribeAllBets => {
                                    if let Err(_) = manager_writer.send(WsManagerEvent::SubscribeAllBets { id: uuid.clone() }){
                                        break;
                                    }

                                },
                                WebsocketsIncommingMessage::UnsubscribeAllBets => {
                                    if let Err(_) = manager_writer.send(WsManagerEvent::UnsubscribeAllBets { id: uuid.clone() }){
                                        break;
                                    }

                                },
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
                                        let seed = blake_hash(&format!("{}{}{}",user_id,chrono::offset::Utc::now(),*PASSWORD_SALT));
                                        let _ = db.reveal_last_seed(user_id).await;
                                        if let Err(e) = db.new_server_seed(user_id, &seed).await{
                                            if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::ErrorText(ErrorText { error: format!("{:?}",e) })).unwrap())).await{
                                                error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                break;
                                            }
                                        }
                                        let seed = blake_hash(&seed);
                                        if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::ServerSeedHidden (Seed{ seed })).unwrap())).await{
                                            error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                            break;
                                        }
                                    }
                                },
                                WebsocketsIncommingMessage::MakeBet(mut bet) => {
                                    if let Some(user_id) = user_id{
                                        bet.user_id.replace(user_id);
                                        bet.uuid.replace(uuid.clone());
                                        if let Err(_) = engine_sender.send(EnginePropagatedBet::NewBet(bet)).await{
                                            break;
                                        };
                                    }
                                },
                                WebsocketsIncommingMessage::ContinueGame(mut bet) => {
                                    if let Some(user_id) = user_id{
                                        bet.user_id.replace(user_id);
                                        bet.uuid.replace(uuid.clone());
                                        if let Err(_) = engine_sender.send(EnginePropagatedBet::ContinueGame(bet)).await{
                                            break;
                                        };
                                    }

                                }
                                WebsocketsIncommingMessage::GetState(request) => {
                                    if let Some(user_id) = user_id {
                                        if let Ok(Some(state)) = db.fetch_game_state(request.game_id, user_id, request.coin_id).await{
                                            if let Err(e) = ws_tx
                                                .send(Message::text(
                                                    serde_json::to_string(&ResponseBody::State(state))
                                                        .unwrap(),
                                                ))
                                                .await
                                            {
                                                error!("Error on socket `{:?}`: `{:?}`", ws_tx, e);
                                            }

                                        } else {
                                            if let Err(e) = ws_tx.send(Message::text(serde_json::to_string(&ResponseBody::InfoText(InfoText { message: "No state found".into() })).unwrap())).await{
                                                error!("Error on socket `{:?}`: `{:?}`",ws_tx,e);
                                                break;
                                            }

                                        }
                                    }
                                }

                                WebsocketsIncommingMessage::GetUuid => {
                                    if let Err(e) = ws_tx
                                        .send(Message::text(
                                            serde_json::to_string(&ResponseBody::Uuid(UuidToken { uuid: uuid.clone() }))
                                                .unwrap(),
                                        ))
                                        .await
                                    {
                                        error!("Error on socket `{:?}`: `{:?}`", ws_tx, e);
                                        break;
                                    }

                                }

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
        .send(WsManagerEvent::UnsubscribeFeed(uuid))
        .unwrap();
}

/// Get all games
///
/// Get all games records
#[utoipa::path(
        tag="game",
        get,
        path = "/api/game/list",
        responses(
            (status = 200, description = "All games records", body = Game),
            (status = 500, description = "Internal server error", body = ErrorText),
        )
    )]
pub async fn get_all_games(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let games = db
        .fetch_all_games()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Games(Games { games })))
}

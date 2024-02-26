use rust_decimal::Decimal;
use sqlx::types::BigDecimal;
use tokio::select;
//pub use tokio::sync::broadcast::{channel, Receiver, Sender};

// use crate::models::db_models::{Bet, TokenPrice};
// use crate::models::json_responses::BetInfoResponse;

use crate::models::db_models::Bet;
use crate::models::json_requests::PropagatedBet;
use crate::{errors::ManagerError, models::json_requests::WebsocketsIncommingMessage};
pub use async_channel::{Receiver, Sender};
pub use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};
// pub struct DbPropagatedBet {
//     pub bet: Bet,
//     pub block_id: u64,
// }

// pub type DbReceiver = UnboundedReceiver<DbMessage>;
// pub type DbSender = UnboundedSender<DbMessage>;

// pub type BetReceiver = Receiver<PropagatedBet>;
// pub type BetSender = Sender<PropagatedBet>;

// pub enum DbMessage {
//     PlaceBet(DbPropagatedBet),
//     NewPrice(TokenPrice),
// }

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChannelType {
    Bets(i64),
}

pub enum WsData {
    NewBet(Bet),
    ServerSeed(String),
}

// pub enum WsEvent {
//     Subscribe(ChannelType),
//     UnsubscribeBets(ChannelType),
//     Auth(String),
//     NewClientSeed(String),
//     NewServerSeed,
//     MakeBet {
//         game_id: i64,
//         amount: Decimal,
//         difficulty: u64,
//     },
// }

pub type WsDataFeedReceiver = UnboundedReceiver<WsData>;
pub type WsDataFeedSender = UnboundedSender<WsData>;

pub type WsEventReceiver = UnboundedReceiver<WebsocketsIncommingMessage>;
pub type WsEventSender = UnboundedSender<WebsocketsIncommingMessage>;

pub type EngineBetReciever = Receiver<PropagatedBet>;
pub type EngineBetSender = Sender<PropagatedBet>;

#[derive(Debug)]
pub enum WsManagerEvent {
    SubscribeFeed {
        id: SocketAddr,
        feed: WsDataFeedSender,
    },
    UnsubscribeFeed(SocketAddr),
    SubscribeChannel {
        id: SocketAddr,
        channel: ChannelType,
    },
    UnsubscribeChannel {
        id: SocketAddr,
        channel: ChannelType,
    },
    PropagateBet(Bet),
}

pub type WsManagerEventReceiver = UnboundedReceiver<WsManagerEvent>;
pub type WsManagerEventSender = UnboundedSender<WsManagerEvent>;

pub struct Manager {
    feeds: HashMap<SocketAddr, WsDataFeedSender>,
    subscriptions: HashMap<ChannelType, HashSet<SocketAddr>>,
    manager_rx: WsManagerEventReceiver,
}

impl Manager {
    pub fn new(manager_rx: WsManagerEventReceiver) -> Self {
        let mut subscriptions: HashMap<ChannelType, HashSet<SocketAddr>> =
            HashMap::with_capacity(1);
        subscriptions.insert(ChannelType::Bets(1), Default::default());

        Self {
            feeds: Default::default(),
            subscriptions,
            manager_rx,
        }
    }

    fn propagate_bet(&self, bet: &Bet) -> Result<(), ManagerError> {
        match self.subscriptions.get(&ChannelType::Bets(bet.game_id)) {
            Some(subs) => {
                for sub in subs.iter() {
                    if let Some(feed) = self.feeds.get(sub) {
                        if let Err(e) = feed.send(WsData::NewBet(bet.clone())) {
                            error!("Error propagating bet to feed `{:?}`: `{:?}`", sub, e);
                        }
                    }
                }
            }
            None => {
                return Err(ManagerError::ChannelIsNotPresent(ChannelType::Bets(
                    bet.game_id,
                )))
            }
        }
        Ok(())
    }

    fn process_event(&mut self, event: &WsManagerEvent) -> Result<(), ManagerError> {
        debug!("Got event: {:?}", event);
        match event {
            WsManagerEvent::SubscribeFeed { id, feed } => {
                match self.feeds.insert(*id, feed.clone()) {
                    Some(_) => {
                        //debug!("Channel for ip `{:?}` got removed", id);
                        self.subscriptions.iter_mut().for_each(|(_, ids)| {
                            ids.remove(id);
                        });
                    }
                    None => {}
                }
            }
            WsManagerEvent::UnsubscribeFeed(id) => {
                self.subscriptions.iter_mut().for_each(|(_, ids)| {
                    ids.remove(id);
                });
                self.feeds.remove(id);
            }
            WsManagerEvent::SubscribeChannel { id, channel } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(*id));
                }
                match self.subscriptions.get_mut(channel) {
                    Some(subs) => {
                        subs.insert(*id);
                    }
                    None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                }
            }
            WsManagerEvent::UnsubscribeChannel { id, channel } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(*id));
                }
                match self.subscriptions.get_mut(channel) {
                    Some(subs) => {
                        subs.remove(id);
                    }
                    None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                }
            }
            WsManagerEvent::PropagateBet(bet) => {
                self.propagate_bet(bet)?;
            }
        }
        Ok(())
    }

    pub async fn run(mut self) -> Result<(), ManagerError> {
        info!("Starting WS manager");

        let mut events: Vec<WsManagerEvent> = Vec::with_capacity(50);
        loop {
            let amount = self.manager_rx.recv_many(&mut events, 50).await;
            debug!("Got total `{}` events", amount);
            if amount == 0 {
                continue;
            }

            for event in events.iter() {
                if let Err(e) = self.process_event(event) {
                    error!("Error processing event: `{:?}`", e);
                }
            }

            events.clear();
        }
        //Ok(())
    }
}

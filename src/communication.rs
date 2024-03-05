//pub use tokio::sync::broadcast::{channel, Receiver, Sender};

// use crate::models::db_models::{Bet, TokenPrice};
// use crate::models::json_responses::BetInfoResponse;

use crate::db::DB;
use crate::models::db_models::Bet;
use crate::models::json_requests::PropagatedBet;
use crate::models::json_responses::BetExpanded;
use crate::{errors::ManagerError, models::json_requests::WebsocketsIncommingMessage};
pub use async_channel::{Receiver, Sender};
pub use std::collections::{HashMap, HashSet};

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
    NewBet(BetExpanded),
    ServerSeed(String),
}

pub type WsDataFeedReceiver = UnboundedReceiver<WsData>;
pub type WsDataFeedSender = UnboundedSender<WsData>;

pub type WsEventReceiver = UnboundedReceiver<WebsocketsIncommingMessage>;
pub type WsEventSender = UnboundedSender<WebsocketsIncommingMessage>;

pub type EngineBetReciever = Receiver<PropagatedBet>;
pub type EngineBetSender = Sender<PropagatedBet>;

#[derive(Debug)]
pub enum WsManagerEvent {
    SubscribeFeed { id: String, feed: WsDataFeedSender },
    UnsubscribeFeed(String),
    SubscribeChannel { id: String, channel: ChannelType },
    UnsubscribeChannel { id: String, channel: ChannelType },
    PropagateBet(BetExpanded),
}

pub type WsManagerEventReceiver = UnboundedReceiver<WsManagerEvent>;
pub type WsManagerEventSender = UnboundedSender<WsManagerEvent>;

pub struct Manager {
    feeds: HashMap<String, WsDataFeedSender>,
    subscriptions: HashMap<ChannelType, HashSet<String>>,
    manager_rx: WsManagerEventReceiver,
}

impl Manager {
    pub async fn new(manager_rx: WsManagerEventReceiver, db: &DB) -> Self {
        let games = db.fetch_all_games().await.expect("Unable to fetch games");
        let mut subscriptions: HashMap<ChannelType, HashSet<String>> =
            HashMap::with_capacity(games.len());
        for game in games {
            subscriptions.insert(ChannelType::Bets(game.id), Default::default());
        }

        Self {
            feeds: Default::default(),
            subscriptions,
            manager_rx,
        }
    }

    fn propagate_bet(&self, bet: &BetExpanded) -> Result<(), ManagerError> {
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
                match self.feeds.insert(id.to_owned(), feed.clone()) {
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
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }
                match self.subscriptions.get_mut(channel) {
                    Some(subs) => {
                        subs.insert(id.to_owned());
                    }
                    None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                }
            }
            WsManagerEvent::UnsubscribeChannel { id, channel } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
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

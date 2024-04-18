use crate::db::DB;
use crate::models::db_models::{GameState, Invoice};
use crate::models::json_requests::{ChatMessage, ContinueGame, PropagatedBet};
use crate::models::json_responses::{BetExpanded, PropagatedChatMessage};
use crate::{errors::ManagerError, models::json_requests::WebsocketsIncommingMessage};
pub use async_channel::{Receiver, Sender};
pub use std::collections::{HashMap, HashSet};

pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChannelType {
    Bets(i64),
    ChatRoom(i64),
    Invoice(i64),
}

#[derive(Debug, Clone)]
pub enum WsData {
    NewBet(BetExpanded),
    ServerSeed(String),
    StateUpdate(GameState),
    NewMessage(PropagatedChatMessage),
    Invoice(Invoice),
}

pub type WsDataFeedReceiver = UnboundedReceiver<WsData>;
pub type WsDataFeedSender = UnboundedSender<WsData>;

pub type WsEventReceiver = UnboundedReceiver<WebsocketsIncommingMessage>;
pub type WsEventSender = UnboundedSender<WebsocketsIncommingMessage>;

pub enum EnginePropagatedBet {
    NewBet(PropagatedBet),
    ContinueGame(ContinueGame),
}

pub type EngineBetReciever = Receiver<EnginePropagatedBet>;
pub type EngineBetSender = Sender<EnginePropagatedBet>;

pub type StatefulEngineBetReciever = UnboundedReceiver<EnginePropagatedBet>;
pub type StatefulEngineBetSender = UnboundedSender<EnginePropagatedBet>;

#[derive(Debug)]
pub enum WsManagerEvent {
    SubscribeFeed {
        id: String,
        feed: WsDataFeedSender,
    },
    UnsubscribeFeed(String),
    SubscribeChannel {
        id: String,
        channel: ChannelType,
    },
    UnsubscribeChannel {
        id: String,
        channel: ChannelType,
    },
    SubscribeAllBets {
        id: String,
    },
    UnsubscribeAllBets {
        id: String,
    },
    SendMessage {
        id: String,
        user_id: i64,
        username: String,

        message: ChatMessage,
        level: i64,
        avatar: Option<String>,
        mentions: Vec<i64>,
    },
    PropagateBet(BetExpanded),
    PropagateState(GameState),
    PropagateInvoice(Invoice),
}

pub type WsManagerEventReceiver = UnboundedReceiver<WsManagerEvent>;
pub type WsManagerEventSender = UnboundedSender<WsManagerEvent>;

pub struct Manager {
    feeds: HashMap<String, WsDataFeedSender>,
    subscriptions_bets: HashMap<i64, HashSet<String>>,
    subscriptions_chat: HashMap<i64, HashSet<String>>,
    subscriptions_invoices: HashMap<i64, HashSet<String>>,
    manager_rx: WsManagerEventReceiver,
}

impl Manager {
    pub async fn new(manager_rx: WsManagerEventReceiver, db: &DB) -> Self {
        let games = db.fetch_all_games().await.expect("Unable to fetch games");
        let mut subscriptions: HashMap<i64, HashSet<String>> = HashMap::with_capacity(games.len());
        for game in games.iter() {
            subscriptions.insert(game.id, Default::default());
        }

        Self {
            feeds: Default::default(),
            subscriptions_bets: subscriptions.clone(),
            subscriptions_chat: subscriptions,
            subscriptions_invoices: HashMap::default(),
            manager_rx,
        }
    }

    fn propagate_bet(&self, bet: &BetExpanded) -> Result<(), ManagerError> {
        match self.subscriptions_bets.get(&bet.game_id) {
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

    fn propagate_chat_message(
        &self,
        message: &ChatMessage,
        user_id: i64,
        username: String,
        level: i64,
        avatar: Option<String>,
        mentions: Vec<i64>,
    ) -> Result<(), ManagerError> {
        match self.subscriptions_chat.get(&message.chat_room) {
            Some(subs) => {
                let msg = WsData::NewMessage(PropagatedChatMessage {
                    room_id: message.chat_room,
                    user_id,
                    username: username.clone(),
                    level,
                    avatar,
                    message: message.message.clone(),
                    attached_media: message.attached_media.clone(),
                    mentions,
                });
                for sub in subs.iter() {
                    if let Some(feed) = self.feeds.get(sub) {
                        if let Err(e) = feed.send(msg.clone()) {
                            error!("Error propagating message to feed `{:?}`: `{:?}`", sub, e);
                        }
                    }
                }
            }
            None => {
                return Err(ManagerError::ChannelIsNotPresent(ChannelType::ChatRoom(
                    message.chat_room,
                )))
            }
        }
        Ok(())
    }

    fn propagate_state(&self, state: &GameState) -> Result<(), ManagerError> {
        match self.subscriptions_bets.get(&state.game_id) {
            Some(subs) => {
                for sub in subs.iter() {
                    if let Some(feed) = self.feeds.get(sub) {
                        if let Err(e) = feed.send(WsData::StateUpdate(state.clone())) {
                            error!("Error propagating bet to feed `{:?}`: `{:?}`", sub, e);
                        }
                    }
                }
            }
            None => {
                return Err(ManagerError::ChannelIsNotPresent(ChannelType::Bets(
                    state.game_id,
                )))
            }
        }
        Ok(())
    }

    fn propagate_invoice(&self, invoice: &Invoice) -> Result<(), ManagerError> {
        match self.subscriptions_invoices.get(&invoice.user_id) {
            Some(subs) => {
                for sub in subs.iter() {
                    if let Some(feed) = self.feeds.get(sub) {
                        if let Err(e) = feed.send(WsData::Invoice(invoice.clone())) {
                            error!("Error propagating invoice to feed `{:?}`: `{:?}`", sub, e);
                        }
                    }
                }
            }
            None => {
                return Err(ManagerError::ChannelIsNotPresent(ChannelType::Invoice(
                    invoice.user_id,
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
                        self.subscriptions_bets.iter_mut().for_each(|(_, ids)| {
                            ids.remove(id);
                        });
                    }
                    None => {}
                }
            }
            WsManagerEvent::UnsubscribeFeed(id) => {
                self.subscriptions_bets.iter_mut().for_each(|(_, ids)| {
                    ids.remove(id);
                });
                self.feeds.remove(id);
            }
            WsManagerEvent::SubscribeChannel { id, channel } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }
                match channel {
                    ChannelType::Bets(channel_id) => {
                        match self.subscriptions_bets.get_mut(channel_id) {
                            Some(subs) => {
                                subs.insert(id.to_owned());
                            }
                            None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                        }
                    }
                    ChannelType::ChatRoom(channel_id) => {
                        match self.subscriptions_chat.get_mut(channel_id) {
                            Some(subs) => {
                                subs.insert(id.to_owned());
                            }
                            None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                        }
                    }
                    ChannelType::Invoice(user_id) => {
                        self.subscriptions_invoices
                            .entry(*user_id)
                            .and_modify(|subs| {
                                subs.insert(id.clone());
                            })
                            .or_insert(HashSet::from([id.clone()]));
                    }
                }
            }
            WsManagerEvent::UnsubscribeChannel { id, channel } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }
                match channel {
                    ChannelType::Bets(channel_id) => {
                        match self.subscriptions_bets.get_mut(channel_id) {
                            Some(subs) => {
                                subs.remove(id);
                            }
                            None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                        }
                    }
                    ChannelType::ChatRoom(channel_id) => {
                        match self.subscriptions_chat.get_mut(channel_id) {
                            Some(subs) => {
                                subs.remove(id);
                            }
                            None => return Err(ManagerError::ChannelIsNotPresent(channel.clone())),
                        }
                    }
                    ChannelType::Invoice(user_id) => {
                        let mut remove = false;
                        self.subscriptions_invoices
                            .entry(*user_id)
                            .and_modify(|subs| {
                                subs.remove(id);
                                if subs.len() == 0 {
                                    remove = true;
                                }
                            });
                        if remove {
                            self.subscriptions_invoices.remove(&user_id);
                        }
                    }
                }
            }
            WsManagerEvent::PropagateBet(bet) => {
                self.propagate_bet(bet)?;
            }
            WsManagerEvent::PropagateState(state) => self.propagate_state(state)?,
            WsManagerEvent::SubscribeAllBets { id } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }

                for (_channel, subs) in self.subscriptions_bets.iter_mut() {
                    subs.insert(id.to_owned());
                }
            }
            WsManagerEvent::UnsubscribeAllBets { id } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }

                for (_channel, subs) in self.subscriptions_bets.iter_mut() {
                    subs.remove(id);
                }
            }
            WsManagerEvent::SendMessage {
                id,
                user_id,
                message,
                username,
                level,
                avatar,
                mentions,
            } => {
                if !self.feeds.contains_key(id) {
                    return Err(ManagerError::FeedDoesntExist(id.to_owned()));
                }
                self.propagate_chat_message(
                    message,
                    *user_id,
                    username.clone(),
                    *level,
                    avatar.clone(),
                    mentions.to_vec(),
                )?;
            }
            WsManagerEvent::PropagateInvoice(invoice) => {
                self.propagate_invoice(invoice)?;
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
    }
}

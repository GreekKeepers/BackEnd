use crate::games::{
    CoinFlip, Dice, Mines, Plinko, Poker, Race, Rocket, StatefulGameEng, StatefullTest, Wheel, RPS,
};
use crate::models::db_models::GameState;
use crate::models::json_responses::BetExpanded;
use crate::tools::blake_hash_256_u64;
use crate::DB;
use crate::{communication::*, games::GameEng};
use rust_decimal::Decimal;
use serde_json::Error;
use tracing::{debug, error, info, warn};

pub fn parse_stateless_game(
    game_name: &str,
    params: &str,
) -> Result<Option<Box<dyn GameEng>>, Error> {
    match game_name {
        "CoinFlip" => match serde_json::from_str::<CoinFlip>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing CoinFlip game: `{:?}`", e);
                Err(e)
            }
        },
        "Dice" => match serde_json::from_str::<Dice>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Dice game: `{:?}`", e);
                Err(e)
            }
        },
        "RPS" => match serde_json::from_str::<RPS>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing RPS game: `{:?}`", e);
                Err(e)
            }
        },
        "Race" => match serde_json::from_str::<Race>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Race game: `{:?}`", e);
                Err(e)
            }
        },
        "Wheel" => match serde_json::from_str::<Wheel>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Wheel game: `{:?}`", e);
                Err(e)
            }
        },
        "Rocket" => match serde_json::from_str::<Rocket>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Rocket game: `{:?}`", e);
                Err(e)
            }
        },
        "Thimbles" => match serde_json::from_str::<Race>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Thimbles game: `{:?}`", e);
                Err(e)
            }
        },
        "Crash" => match serde_json::from_str::<Rocket>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Crash game: `{:?}`", e);
                Err(e)
            }
        },
        "Plinko" => match serde_json::from_str::<Plinko>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Plinko game: `{:?}`", e);
                Err(e)
            }
        },
        "CarRace" => match serde_json::from_str::<Plinko>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing CarRace game: `{:?}`", e);
                Err(e)
            }
        },

        _ => Ok(None),
    }
}

pub fn parse_statefull_game(
    game_name: &str,
    params: &str,
) -> Result<Option<Box<dyn StatefulGameEng>>, Error> {
    match game_name {
        //"StatefullTest" => match serde_json::from_str::<StatefullTest>(params) {
        //    Ok(gm) => Ok(Some(Box::new(gm))),
        //    Err(e) => {
        //        error!("Error deserializing StatefullTest game: `{:?}`", e);
        //        Err(e)
        //    }
        //},
        "Mines" => match serde_json::from_str::<Mines>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Mines game: `{:?}`", e);
                Err(e)
            }
        },
        "Poker" => match serde_json::from_str::<Poker>(params) {
            Ok(gm) => Ok(Some(Box::new(gm))),
            Err(e) => {
                error!("Error deserializing Poker game: `{:?}`", e);
                Err(e)
            }
        },

        _ => Ok(None),
    }
}

pub fn generate_random_numbers(
    client_seed: &str,
    server_seed: &str,
    timestamp: u64,
    amount: u64,
) -> Vec<u64> {
    let postfix = format!("{timestamp}{client_seed}{server_seed}");

    (0..amount)
        .map(|index| blake_hash_256_u64(&format!("{index}{postfix}")))
        .collect()
}

pub struct Engine {
    db: DB,
    manager_sender: WsManagerEventSender,
    bet_reciever: EngineBetReciever,
    game_engines: HashMap<u64, Box<dyn GameEng>>,
    stateful_bet_sender: StatefulEngineBetSender,
}

impl Engine {
    pub async fn new(
        db: DB,
        manager_sender: WsManagerEventSender,
        bet_reciever: EngineBetReciever,
        stateful_bet_sender: StatefulEngineBetSender,
    ) -> Self {
        let games: HashMap<u64, Box<dyn GameEng>> = db
            .fetch_all_games()
            .await
            .expect("Error fetching games from db")
            .into_iter()
            .map(|game| {
                (
                    game.id as u64,
                    parse_stateless_game(&game.name, &game.parameters).unwrap(),
                )
            })
            .filter_map(|(game_id, game_data)| {
                if game_data.is_some() {
                    Some((game_id, game_data.unwrap()))
                } else {
                    None
                }
            })
            .collect();

        Self {
            db,
            manager_sender,
            bet_reciever,
            game_engines: games,
            stateful_bet_sender,
        }
    }

    pub async fn run(self) {
        info!("Starting engine");
        loop {
            let orig_bet = match self.bet_reciever.recv().await {
                Ok(bet) => bet,
                Err(e) => {
                    error!("Error recieving bet {:?}", e);
                    break;
                }
            };
            match &orig_bet {
                EnginePropagatedBet::NewBet(bet) => {
                    if bet.num_games > 100 {
                        continue;
                    }

                    let game_eng =
                        if let Some(game_eng) = self.game_engines.get(&(bet.game_id as u64)) {
                            game_eng
                        } else {
                            warn!("Stateless game `{:?}` not found", bet.game_id);
                            if let Err(e) = self.stateful_bet_sender.send(orig_bet) {
                                error!("Error propagating bet to the stateful engine: {:?}", e);
                                break;
                            }
                            continue;
                        };

                    let amount = if let Ok(Some(amount)) = self
                        .db
                        .fetch_amount(bet.user_id.unwrap(), bet.coin_id)
                        .await
                    {
                        amount
                    } else {
                        continue;
                    };

                    if bet.amount > amount {
                        continue;
                    }

                    let user_seed =
                        match self.db.fetch_current_user_seed(bet.user_id.unwrap()).await {
                            Ok(seed) => seed,
                            Err(e) => {
                                error!(
                                    "Error getting user seed for user `{}`: {:?}",
                                    bet.user_id.unwrap(),
                                    e
                                );
                                continue;
                            }
                        };

                    let server_seed = match self
                        .db
                        .fetch_current_server_seed(bet.user_id.unwrap())
                        .await
                    {
                        Ok(seed) => seed,
                        Err(e) => {
                            error!(
                                "Error getting server seed for user `{}`: {:?}",
                                bet.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    };

                    let timestamp = chrono::offset::Utc::now();

                    let random_numbers = generate_random_numbers(
                        &user_seed.user_seed,
                        &server_seed.server_seed,
                        timestamp.timestamp_millis() as u64,
                        game_eng.numbers_per_bet() * bet.num_games,
                    );

                    let game_result = if let Some(res) = game_eng.play(&bet, &random_numbers) {
                        res
                    } else {
                        warn!("Couldn't proccess bet");
                        continue;
                    };

                    // Apply taking money/sending profit

                    match self
                        .db
                        .decrease_balance(
                            bet.user_id.unwrap(),
                            bet.coin_id,
                            bet.amount * Decimal::from(game_result.num_games),
                        )
                        .await
                    {
                        Ok(success) => {
                            if !success {
                                continue;
                            }
                        }
                        Err(e) => {
                            error!(
                                "Error decreasing balance for user `{}`: {:?}",
                                bet.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    }

                    match self
                        .db
                        .increase_balance(
                            bet.user_id.unwrap(),
                            bet.coin_id,
                            &game_result.total_profit,
                        )
                        .await
                    {
                        Ok(success) => {
                            if !success {
                                warn!(
                            "Increasing balance wasn't successful bet data: `{:?}` amount: `{}`",
                            bet, &game_result.total_profit
                        );
                                continue;
                            }
                        }
                        Err(e) => {
                            error!(
                                "Error increasing balance for user `{}`: {:?}",
                                bet.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    }

                    let outcomes = format!("{:?}", game_result.outcomes);
                    let profits = format!("{:?}", game_result.profits);

                    if let Err(e) = self
                        .db
                        .place_bet(
                            bet.amount,
                            game_result.total_profit,
                            game_result.num_games as i32,
                            &outcomes,
                            &profits,
                            &bet.data,
                            None,
                            bet.uuid.as_ref().unwrap(),
                            bet.game_id,
                            bet.user_id.unwrap(),
                            bet.coin_id,
                            user_seed.id,
                            server_seed.id,
                        )
                        .await
                    {
                        error!("Error adding bet to the db: {:?}", e);
                    };

                    let user =
                        if let Ok(Some(user)) = self.db.fetch_user(bet.user_id.unwrap()).await {
                            user
                        } else {
                            error!("Unable to find user: {:?}", bet.user_id);
                            continue;
                        };

                    let constructed_bet = BetExpanded {
                        id: 0,
                        timestamp,
                        amount: bet.amount,
                        profit: game_result.total_profit,
                        bet_info: game_result.data,
                        state: None,
                        game_id: bet.game_id,
                        user_id: bet.user_id.unwrap(),
                        username: user.username,
                        coin_id: bet.coin_id,
                        userseed_id: user_seed.id,
                        serverseed_id: server_seed.id,
                        outcomes,
                        num_games: game_result.num_games as i32,
                        uuid: bet.uuid.clone().unwrap(),
                        profits,
                    };

                    if let Err(e) = self
                        .manager_sender
                        .send(WsManagerEvent::PropagateBet(constructed_bet))
                    {
                        error!("Error propagating bet: {:?}", e);
                        break;
                    }
                }
                EnginePropagatedBet::ContinueGame(_) => {
                    if let Err(e) = self.stateful_bet_sender.send(orig_bet) {
                        error!("Error propagating bet to the stateful engine: {:?}", e);
                        break;
                    }
                }
            }
        }
    }
}

pub struct StatefulGameEngine {
    bet_reciever: StatefulEngineBetReciever,

    game_engines: HashMap<u64, Box<dyn StatefulGameEng>>,
    db: DB,
    manager_sender: WsManagerEventSender,
}

impl StatefulGameEngine {
    pub async fn new(
        db: DB,
        manager_sender: WsManagerEventSender,
        bet_reciever: StatefulEngineBetReciever,
    ) -> Self {
        let games: HashMap<u64, Box<dyn StatefulGameEng>> = db
            .fetch_all_games()
            .await
            .expect("Error fetching games from db")
            .into_iter()
            .map(|game| {
                (
                    game.id as u64,
                    parse_statefull_game(&game.name, &game.parameters).unwrap(),
                )
            })
            .filter_map(|(game_id, game_data)| {
                if game_data.is_some() {
                    Some((game_id, game_data.unwrap()))
                } else {
                    None
                }
            })
            .collect();

        Self {
            bet_reciever,
            db,
            manager_sender,
            game_engines: games,
        }
    }

    pub async fn run(mut self) {
        info!("Starting engine");
        loop {
            let bet = match self.bet_reciever.recv().await {
                Some(bet) => bet,
                None => {
                    error!("Error recieving bet");
                    break;
                }
            };

            match bet {
                EnginePropagatedBet::NewBet(bet) => {
                    debug!("recieved `NewBet` event: {:?}", bet);
                    let game_eng =
                        if let Some(game_eng) = self.game_engines.get(&(bet.game_id as u64)) {
                            game_eng
                        } else {
                            warn!("Statefull game `{:?}` not found", bet.game_id);
                            continue;
                        };

                    if let Ok(Some(state)) = self
                        .db
                        .fetch_game_state(bet.game_id, bet.user_id.unwrap(), bet.coin_id)
                        .await
                    {
                        warn!("State already exists for the bet `{:?}`: {:?}", bet, state);
                        continue;
                    }

                    let amount = if let Ok(Some(amount)) = self
                        .db
                        .fetch_amount(bet.user_id.unwrap(), bet.coin_id)
                        .await
                    {
                        amount
                    } else {
                        continue;
                    };

                    if bet.amount > amount {
                        continue;
                    }

                    let user_seed =
                        match self.db.fetch_current_user_seed(bet.user_id.unwrap()).await {
                            Ok(seed) => seed,
                            Err(e) => {
                                error!(
                                    "Error getting user seed for user `{}`: {:?}",
                                    bet.user_id.unwrap(),
                                    e
                                );
                                continue;
                            }
                        };

                    let server_seed = match self
                        .db
                        .fetch_current_server_seed(bet.user_id.unwrap())
                        .await
                    {
                        Ok(seed) => seed,
                        Err(e) => {
                            error!(
                                "Error getting server seed for user `{}`: {:?}",
                                bet.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    };

                    let timestamp = chrono::offset::Utc::now();

                    let random_numbers = generate_random_numbers(
                        &user_seed.user_seed,
                        &server_seed.server_seed,
                        timestamp.timestamp_millis() as u64,
                        game_eng.numbers_per_bet(),
                    );

                    let game_result =
                        if let Some(res) = game_eng.start_playing(&bet, &random_numbers) {
                            res
                        } else {
                            warn!("Couldn't proccess bet {:?}", bet);
                            continue;
                        };

                    if let Err(e) = self
                        .db
                        .decrease_balance(bet.user_id.unwrap(), bet.coin_id, bet.amount)
                        .await
                    {
                        error!("Error decreasing amount for the bet {:?}: {:?}", bet, e);
                        continue;
                    }

                    if game_result.finished {
                        // game finished

                        let outcomes = format!("{:?}", game_result.outcomes);
                        let profits = format!("{:?}", game_result.profits);
                        if let Err(e) = self
                            .db
                            .place_bet(
                                bet.amount,
                                game_result.total_profit,
                                game_result.num_games as i32,
                                &outcomes,
                                &profits,
                                &bet.data,
                                Some(&game_result.data),
                                bet.uuid.as_ref().unwrap(),
                                bet.game_id,
                                bet.user_id.unwrap(),
                                bet.coin_id,
                                user_seed.id,
                                server_seed.id,
                            )
                            .await
                        {
                            error!("Error adding bet to the db: {:?}", e);
                        };

                        if game_result.num_games > 1 {
                            if let Err(e) = self
                                .db
                                .remove_game_state(bet.game_id, bet.user_id.unwrap(), bet.coin_id)
                                .await
                            {
                                error!("Error removing state from the db {:?}: {:?}", bet, e);
                            }
                        }

                        let user = if let Ok(Some(user)) =
                            self.db.fetch_user(bet.user_id.unwrap()).await
                        {
                            user
                        } else {
                            error!("Unable to find user: {:?}", bet.user_id);
                            continue;
                        };

                        let constructed_bet = BetExpanded {
                            id: 0,
                            timestamp,
                            amount: bet.amount,
                            profit: game_result.total_profit,
                            bet_info: bet.data,
                            state: Some(game_result.data),
                            game_id: bet.game_id,
                            user_id: bet.user_id.unwrap(),
                            username: user.username,
                            coin_id: bet.coin_id,
                            userseed_id: user_seed.id,
                            serverseed_id: server_seed.id,
                            outcomes,
                            num_games: game_result.num_games as i32,
                            uuid: bet.uuid.clone().unwrap(),
                            profits,
                        };

                        if let Err(e) = self
                            .manager_sender
                            .send(WsManagerEvent::PropagateBet(constructed_bet))
                        {
                            error!("Error propagating bet: {:?}", e);
                            break;
                        }
                    } else {
                        // game state changed
                        if let Err(e) = self
                            .db
                            .insert_game_state(
                                bet.game_id,
                                bet.user_id.unwrap(),
                                bet.uuid.as_ref().unwrap(),
                                bet.coin_id,
                                &bet.data,
                                &game_result.data,
                                &bet.amount,
                                user_seed.id,
                                server_seed.id,
                            )
                            .await
                        {
                            error!("Error inserting state to the db: {:?}", e);
                            continue;
                        }

                        if let Err(e) =
                            self.manager_sender
                                .send(WsManagerEvent::PropagateState(GameState {
                                    id: 0,
                                    timestamp,
                                    amount,
                                    bet_info: bet.data,
                                    state: game_result.data,
                                    uuid: bet.uuid.unwrap(),
                                    game_id: bet.game_id,
                                    user_id: bet.user_id.unwrap(),
                                    coin_id: bet.coin_id,
                                    userseed_id: user_seed.id,
                                    serverseed_id: server_seed.id,
                                }))
                        {
                            error!("Error propagating game state: {:?}", e);
                            break;
                        }
                    }
                }
                EnginePropagatedBet::ContinueGame(continue_game) => {
                    debug!("Recieved `ContinueGame` event: {:?}", continue_game);

                    let game_eng = if let Some(game_eng) =
                        self.game_engines.get(&(continue_game.game_id as u64))
                    {
                        game_eng
                    } else {
                        warn!("Statefull game `{:?}` not found", continue_game.game_id);
                        continue;
                    };

                    let mut state = if let Ok(Some(state)) = self
                        .db
                        .fetch_game_state(
                            continue_game.game_id,
                            continue_game.user_id.unwrap(),
                            continue_game.coin_id,
                        )
                        .await
                    {
                        state
                    } else {
                        warn!("State not found for the bet: {:?}", continue_game);
                        continue;
                    };

                    let user_seed = match self
                        .db
                        .fetch_current_user_seed(continue_game.user_id.unwrap())
                        .await
                    {
                        Ok(seed) => seed,
                        Err(e) => {
                            error!(
                                "Error getting user seed for user `{}`: {:?}",
                                continue_game.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    };

                    let server_seed = match self
                        .db
                        .fetch_current_server_seed(continue_game.user_id.unwrap())
                        .await
                    {
                        Ok(seed) => seed,
                        Err(e) => {
                            error!(
                                "Error getting server seed for user `{}`: {:?}",
                                continue_game.user_id.unwrap(),
                                e
                            );
                            continue;
                        }
                    };

                    let timestamp = chrono::offset::Utc::now();

                    let random_numbers = generate_random_numbers(
                        &user_seed.user_seed,
                        &server_seed.server_seed,
                        timestamp.timestamp_millis() as u64,
                        game_eng.numbers_per_bet(),
                    );

                    let game_result = if let Some(game_result) =
                        game_eng.continue_playing(&state, &continue_game, &random_numbers)
                    {
                        game_result
                    } else {
                        warn!("Couldn't proccess bet {:?}", continue_game);
                        continue;
                    };

                    if game_result.finished {
                        // game finished

                        if let Err(e) = self
                            .db
                            .remove_game_state(
                                continue_game.game_id,
                                continue_game.user_id.unwrap(),
                                continue_game.coin_id,
                            )
                            .await
                        {
                            error!("Error removing state: {:?}", e);
                        };

                        if let Err(e) = self
                            .db
                            .increase_balance(
                                continue_game.user_id.unwrap(),
                                continue_game.coin_id,
                                &game_result.total_profit,
                            )
                            .await
                        {
                            error!("Error adding bet to the db: {:?}", e);
                        };

                        let outcomes = format!("{:?}", game_result.outcomes);
                        let profits = format!("{:?}", game_result.profits);
                        if let Err(e) = self
                            .db
                            .place_bet(
                                state.amount,
                                game_result.total_profit,
                                game_result.num_games as i32,
                                &outcomes,
                                &profits,
                                &game_result.data,
                                Some(&game_result.data),
                                continue_game.uuid.as_ref().unwrap(),
                                continue_game.game_id,
                                continue_game.user_id.unwrap(),
                                continue_game.coin_id,
                                user_seed.id,
                                server_seed.id,
                            )
                            .await
                        {
                            error!("Error adding bet to the db: {:?}", e);
                        };

                        let user = if let Ok(Some(user)) =
                            self.db.fetch_user(continue_game.user_id.unwrap()).await
                        {
                            user
                        } else {
                            error!("Unable to find user: {:?}", continue_game.user_id);
                            continue;
                        };

                        let constructed_bet = BetExpanded {
                            id: 0,
                            timestamp,
                            amount: state.amount,
                            profit: game_result.total_profit,
                            bet_info: continue_game.data,
                            state: Some(game_result.data),
                            game_id: continue_game.game_id,
                            user_id: continue_game.user_id.unwrap(),
                            username: user.username,
                            coin_id: continue_game.coin_id,
                            userseed_id: user_seed.id,
                            serverseed_id: server_seed.id,
                            outcomes,
                            num_games: game_result.num_games as i32,
                            uuid: continue_game.uuid.clone().unwrap(),
                            profits,
                        };

                        if let Err(e) = self
                            .manager_sender
                            .send(WsManagerEvent::PropagateBet(constructed_bet))
                        {
                            error!("Error propagating bet: {:?}", e);
                            break;
                        }
                    } else {
                        if let Err(e) = self
                            .db
                            .change_game_state(
                                continue_game.game_id,
                                continue_game.user_id.unwrap(),
                                continue_game.coin_id,
                                &game_result.data,
                            )
                            .await
                        {
                            error!("Error updating state: {:?}", e);
                        };

                        state.state = game_result.data;

                        if let Err(e) = self
                            .manager_sender
                            .send(WsManagerEvent::PropagateState(state))
                        {
                            error!("Error propagating state: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}

use crate::games::{CoinFlip, Dice, Race, RPS};
use crate::models::*;
use crate::tools::blake_hash_256_u64;
use crate::DB;
use crate::{communication::*, games::GameEng};
use rust_decimal::Decimal;
use serde_json::Error;
use tracing::{error, warn};

use self::db_models::Bet;

pub struct Engine {
    db: DB,
    manager_sender: WsManagerEventSender,
    bet_reciever: EngineBetReciever,
    game_engines: HashMap<u64, Box<dyn GameEng>>,
}

impl Engine {
    pub async fn new(
        db: DB,
        manager_sender: WsManagerEventSender,
        bet_reciever: EngineBetReciever,
    ) -> Self {
        let games: HashMap<u64, Box<dyn GameEng>> = db
            .fetch_all_games()
            .await
            .expect("Error fetching games from db")
            .into_iter()
            .map(|game| {
                (
                    game.id as u64,
                    Engine::parse_game(&game.name, &game.parameters)
                        .unwrap()
                        .unwrap(),
                )
            })
            .collect();

        Self {
            db,
            manager_sender,
            bet_reciever,
            game_engines: games,
        }
    }

    pub fn parse_game(game_name: &str, params: &str) -> Result<Option<Box<dyn GameEng>>, Error> {
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

    pub async fn run(self) {
        loop {
            let bet = match self.bet_reciever.recv().await {
                Ok(bet) => bet,
                Err(e) => {
                    error!("Error recieving bet {:?}", e);
                    break;
                }
            };

            if bet.num_games > 100 {
                continue;
            }

            //let total_bet = bet.amount * Decimal::from(bet.num_games);

            let amount =
                if let Ok(Some(amount)) = self.db.fetch_amount(bet.user_id, bet.coin_id).await {
                    amount
                } else {
                    continue;
                };

            if bet.amount > amount {
                continue;
            }

            let game_eng = if let Some(game_eng) = self.game_engines.get(&(bet.game_id as u64)) {
                game_eng
            } else {
                warn!("Game `{:?}` not found", bet.game_id);
                continue;
            };

            let user_seed = match self.db.fetch_current_user_seed(bet.user_id).await {
                Ok(seed) => seed,
                Err(e) => {
                    error!(
                        "Error getting user seed for user `{}`: {:?}",
                        bet.user_id, e
                    );
                    continue;
                }
            };

            let server_seed = match self.db.fetch_current_server_seed(bet.user_id).await {
                Ok(seed) => seed,
                Err(e) => {
                    error!(
                        "Error getting server seed for user `{}`: {:?}",
                        bet.user_id, e
                    );
                    continue;
                }
            };

            let timestamp = chrono::offset::Utc::now();

            let random_numbers = Engine::generate_random_numbers(
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
                    bet.user_id,
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
                        bet.user_id, e
                    );
                    continue;
                }
            }

            match self
                .db
                .increase_balance(bet.user_id, bet.coin_id, &game_result.total_profit)
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
                        bet.user_id, e
                    );
                    continue;
                }
            }

            let outcomes = format!("{:?}", game_result.outcomes);

            if let Err(e) = self
                .db
                .place_bet(
                    bet.amount,
                    game_result.total_profit,
                    game_result.num_games as i32,
                    &outcomes,
                    &bet.data,
                    bet.game_id,
                    bet.user_id,
                    bet.coin_id,
                    user_seed.id,
                    server_seed.id,
                )
                .await
            {
                error!("Error adding bet to the db: {:?}", e);
            };

            let constructed_bet = Bet {
                id: 0,
                timestamp,
                amount: bet.amount,
                profit: game_result.total_profit,
                bet_info: bet.data,
                game_id: bet.game_id,
                user_id: bet.user_id,
                coin_id: bet.coin_id,
                userseed_id: user_seed.id,
                serverseed_id: server_seed.id,
                outcomes,
                num_games: game_result.num_games as i32,
            };

            if let Err(e) = self
                .manager_sender
                .send(WsManagerEvent::PropagateBet(constructed_bet))
            {
                error!("Error propagating bet: {:?}", e);
                break;
            }
        }
    }
}

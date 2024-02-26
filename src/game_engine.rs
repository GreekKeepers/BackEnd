use std::str::FromStr;

use crate::games::CoinFlip;
use crate::models::*;
use crate::tools::blake_hash_256_u64;
use crate::DB;
use crate::{communication::*, games::GameEng};
use serde_json::Error;
use sqlx::types::BigDecimal;
use tracing::{debug, error, info, warn};

use self::db_models::Bet;

pub struct Engine {
    db: DB,
    manager_sender: WsManagerEventSender,
    bet_reciever: EngineBetReciever,
}

impl Engine {
    pub fn new(
        db: DB,
        manager_sender: WsManagerEventSender,
        bet_reciever: EngineBetReciever,
    ) -> Self {
        Self {
            db,
            manager_sender,
            bet_reciever,
        }
    }

    pub fn parse_game(game_name: &str, params: &str) -> Result<Option<impl GameEng>, Error> {
        match game_name {
            "CoinFlip" => match serde_json::from_str::<CoinFlip>(params) {
                Ok(gm) => Ok(Some(gm)),
                Err(e) => {
                    error!("Error deserializing CoinFlip game: `{:?}`", e);
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

            let amount =
                if let Ok(Some(amount)) = self.db.fetch_amount(bet.user_id, bet.coin_id).await {
                    amount
                } else {
                    continue;
                };

            let bet_amount = BigDecimal::from_str(&bet.amount.to_string()).unwrap();

            if bet_amount > amount {
                continue;
            }

            let game = if let Ok(Some(game)) = self.db.fetch_game(bet.game_id).await {
                game
            } else {
                warn!("Could not fetch game `{}`", bet.game_id);
                continue;
            };

            let game_eng = match Engine::parse_game(&game.name, &game.parameters) {
                Ok(eng) => {
                    if let Some(eng) = eng {
                        eng
                    } else {
                        warn!("Game `{:?}` not found", game.name);
                        continue;
                    }
                }
                Err(e) => {
                    error!("Error parsing game parameters for `{}`: {:?}", game.name, e);
                    continue;
                }
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
                game_eng.numbers_per_bet(),
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
                .decrease_balance(bet.user_id, bet.coin_id, bet_amount)
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

            let profit_digd = BigDecimal::from_str(&game_result.total_profit.to_string()).unwrap();

            match self
                .db
                .increase_balance(bet.user_id, bet.coin_id, &profit_digd)
                .await
            {
                Ok(success) => {
                    if !success {
                        warn!(
                            "Increasing balance wasn't successful bet data: `{:?}` amount: `{}`",
                            bet, &profit_digd
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
                outcomes: game_result.outcomes,
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

use crate::{
    db::DB,
    models::{
        db_models::{Bet, GameResult},
        json_requests::PropagatedBet,
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use rust_decimal::Decimal;
use tracing::error;

use super::StatefulGameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct StatefullTestData {
    pub num: Option<u32>,
    pub end_game: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct StatefullTestState {
    pub state: Vec<(u64, u64)>,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct StatefullTest {
    pub multiplier: Decimal,
}

impl StatefulGameEng for StatefullTest {
    fn numbers_per_bet(&self) -> u64 {
        1
    }

    fn start_playing(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: StatefullTestData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Test data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        let generated_num = random_numbers[0];

        if let Some(num) = data.num {
            if num as u64 <= generated_num {
                return Some(GameResult {
                    total_profit: self.multiplier,
                    outcomes: vec![generated_num],
                    profits: vec![self.multiplier],
                    num_games: 1,
                    data: serde_json::to_string(&StatefullTestState {
                        state: vec![(num as u64, generated_num)],
                    })
                    .unwrap(),
                    finished: data.end_game,
                });
            } else {
                return Some(GameResult {
                    total_profit: Decimal::ZERO,
                    outcomes: vec![generated_num],
                    profits: vec![Decimal::ZERO],
                    num_games: 1,
                    data: serde_json::to_string(&StatefullTestState {
                        state: vec![(num as u64, generated_num)],
                    })
                    .unwrap(),
                    finished: true,
                });
            }
        }

        None
    }

    fn continue_playing(
        &self,
        state: &crate::models::db_models::GameState,
        bet: &crate::models::json_requests::ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        let data: StatefullTestData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Test data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        let mut parsed_state: StatefullTestState = serde_json::from_str(&state.state)
            .map_err(|e| {
                error!("Error parsing Test state`{:?}`: {:?}", state.state, e);
                e
            })
            .ok()?;

        let mut total_won = parsed_state.state.len();

        if let Some(num) = data.num {
            let generated_num = random_numbers[0];
            parsed_state.state.push((num as u64, generated_num));
            let outcomes: Vec<u64> = parsed_state.state.iter().map(|v| v.1).collect();

            let state_string = serde_json::to_string(&parsed_state).unwrap();
            total_won += 1;

            if num as u64 <= generated_num {
                // won

                return Some(GameResult {
                    total_profit: state.amount * self.multiplier * Decimal::from(total_won),
                    outcomes,
                    profits: vec![self.multiplier; total_won],
                    num_games: total_won as u32,
                    data: state_string,
                    finished: data.end_game,
                });
            } else {
                return Some(GameResult {
                    total_profit: Decimal::ZERO,
                    outcomes,
                    profits: vec![Decimal::ZERO; total_won + 1],
                    num_games: total_won as u32,
                    data: state_string,
                    finished: true,
                });
            }
        }
        let outcomes: Vec<u64> = parsed_state.state.iter().map(|v| v.1).collect();

        let state_string = serde_json::to_string(&parsed_state).unwrap();
        return Some(GameResult {
            total_profit: state.amount * self.multiplier * Decimal::from(total_won),
            outcomes,
            profits: vec![self.multiplier; total_won],
            num_games: if data.end_game { total_won as u32 } else { 0 },
            data: state_string,
            finished: true,
        });
    }
}

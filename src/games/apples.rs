use crate::models::{db_models::GameResult, json_requests::PropagatedBet};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use rust_decimal::Decimal;
use tracing::error;

use super::StatefulGameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct ApplesData {
    pub difficulty: u8,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct ApplesContinueData {
    pub tile: Option<u8>,
    pub cashout: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct ApplesState {
    pub state: Vec<Vec<bool>>,
    pub picked_tiles: Vec<u8>,
    pub current_multiplier: Decimal,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct ApplesDifficulty {
    pub mines: u8,
    pub total_spaces: u8,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Apples {
    pub difficulties: Vec<ApplesDifficulty>,
    pub multipliers: Vec<[Decimal; 9]>,
}

impl StatefulGameEng for Apples {
    fn start_playing(&self, bet: &PropagatedBet, _: &[u64]) -> Option<GameResult> {
        let data: ApplesData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Apples data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.difficulty as usize >= self.difficulties.len() {
            return None;
        }

        Some(GameResult {
            total_profit: Decimal::ZERO,
            outcomes: Vec::with_capacity(0),
            profits: Vec::with_capacity(0),
            num_games: 1,
            data: serde_json::to_string(&ApplesState {
                state: Vec::with_capacity(0),
                current_multiplier: Decimal::ZERO,
                picked_tiles: Vec::with_capacity(0),
            })
            .unwrap(),
            finished: false,
        })
    }

    fn continue_playing(
        &self,
        state: &crate::models::db_models::GameState,
        bet: &crate::models::json_requests::ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        let data: ApplesContinueData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!(
                    "Error parsing Apples continue data `{:?}`: {:?}",
                    bet.data, e
                );
                e
            })
            .ok()?;

        let mut parsed_state: ApplesState = serde_json::from_str(&state.state)
            .map_err(|e| {
                error!("Error parsing Apples state`{:?}`: {:?}", &state.state, e);
                e
            })
            .ok()?;

        let initial_data: ApplesData = serde_json::from_str(&state.bet_info)
            .map_err(|e| {
                error!(
                    "Error parsing Apples initial data `{:?}`: {:?}",
                    state.bet_info, e
                );
                e
            })
            .ok()?;

        if (data.tile.is_none() || data.cashout) && !parsed_state.current_multiplier.is_zero() {
            let profit = state.amount * parsed_state.current_multiplier;
            return Some(GameResult {
                total_profit: profit,
                outcomes: vec![0; parsed_state.state.len()],
                profits: vec![profit],
                num_games: 1,
                data: state.state.clone(),
                finished: true,
            });
        } else if data.tile.is_none() {
            return None;
        }

        let difficulty = &self.difficulties[initial_data.difficulty as usize];

        let picked_tile = data.tile.unwrap();
        if picked_tile >= difficulty.total_spaces {
            return None;
        }

        let mut row = vec![false; difficulty.total_spaces as usize];

        let rng = random_numbers[0];
        let mut mask = 0x8000000000000000u64;
        let mut mines_amount = difficulty.mines;
        for r in row.iter_mut() {
            if mines_amount == 0 {
                break;
            }
            let res = if rng & mask > 0 {
                mines_amount -= 1;
                true
            } else {
                false
            };
            mask >>= 1;
            *r = res;
        }

        let won = !row[picked_tile as usize];

        parsed_state.state.push(row);
        parsed_state.picked_tiles.push(picked_tile);

        if won {
            parsed_state.current_multiplier =
                self.multipliers[initial_data.difficulty as usize][parsed_state.state.len() - 1];
            let profit = self.multipliers[initial_data.difficulty as usize]
                [parsed_state.state.len() - 1]
                * state.amount;
            Some(GameResult {
                total_profit: profit,
                outcomes: vec![0; parsed_state.state.len()],
                profits: vec![profit],
                num_games: 1,
                data: serde_json::to_string(&parsed_state).unwrap(),
                finished: false,
            })
        } else {
            parsed_state.current_multiplier = Decimal::ZERO;
            let mut outcomes = vec![0; parsed_state.state.len()];
            outcomes[parsed_state.state.len() - 1] = 1;
            Some(GameResult {
                total_profit: Decimal::ZERO,
                outcomes: outcomes,
                profits: vec![Decimal::ZERO],
                num_games: 1,
                data: serde_json::to_string(&parsed_state).unwrap(),
                finished: true,
            })
        }
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

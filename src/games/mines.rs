use std::collections::HashMap;

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
use tracing::{error, warn};

use super::StatefulGameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct MinesData {
    pub num_mines: u32,
    pub tiles: [bool; 25],
    pub cashout: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct MinesContinueData {
    pub tiles: Option<[bool; 25]>,
    pub cashout: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct MinesState {
    pub state: [bool; 25],
    pub mines: [bool; 25],
    pub game_num: u64,
    pub current_multiplier: Decimal,
}

fn is_gem(number_of_tiles_left: usize, number_of_mines_left: usize, rng: usize) -> bool {
    let win_chance = 1000 - (number_of_mines_left * 10000) / number_of_tiles_left;

    if rng % 10000 <= win_chance {
        return true;
    }

    false
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Mines {
    pub multipliers: HashMap<u64, Vec<Decimal>>,
    pub max_reveal: [u32; 24],
}

impl StatefulGameEng for Mines {
    fn start_playing(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: MinesData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Test data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.num_mines == 0 || data.num_mines > 24 {
            return None;
        }

        if data.tiles.iter().filter(|t| **t).count() == 0 {
            return None;
        }

        let mut number_of_revealed_tiles: usize = 0;

        let mut number_of_mines_left: usize = data.num_mines as usize;

        let mut mines: [bool; 25] = [false; 25];
        let mut revealed_tiles: [bool; 25] = [false; 25];

        let mut won = true;

        for i in 0usize..25 {
            if number_of_mines_left == 0 || 25 - number_of_revealed_tiles == number_of_mines_left {
                if data.tiles[i] {
                    revealed_tiles[i] = true;
                }
                continue;
            }

            if data.tiles[i] {
                let gem = is_gem(
                    25 - number_of_revealed_tiles,
                    number_of_mines_left,
                    random_numbers[i] as usize,
                );

                if !gem {
                    number_of_mines_left -= 1;
                    mines[i] = true;
                    won = false;
                }

                revealed_tiles[i] = true;
                number_of_revealed_tiles += 1;
            }
        }

        if !won {
            return Some(GameResult {
                total_profit: Decimal::ZERO,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![Decimal::ZERO],
                num_games: 1,
                data: serde_json::to_string(&MinesState {
                    state: revealed_tiles,
                    mines,
                    game_num: 1,
                    current_multiplier: Decimal::ZERO,
                })
                .unwrap(),
                finished: true,
            });
        }

        let multiplier = if let Some(mults) = self.multipliers.get(&(number_of_mines_left as u64)) {
            mults[number_of_revealed_tiles - 1]
        } else {
            warn!("Multiplier `{:?}` not found", number_of_mines_left);
            return None;
        };

        if !data.cashout {
            let profit = multiplier * bet.amount;
            return Some(GameResult {
                total_profit: profit,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![profit],
                num_games: 1,
                data: serde_json::to_string(&MinesState {
                    state: revealed_tiles,
                    mines,
                    game_num: 1,
                    current_multiplier: multiplier,
                })
                .unwrap(),
                finished: false,
            });
        } else {
            let profit = multiplier * bet.amount;
            return Some(GameResult {
                total_profit: profit,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![profit],
                num_games: 1,
                data: serde_json::to_string(&MinesState {
                    state: revealed_tiles,
                    mines,
                    game_num: 1,
                    current_multiplier: multiplier,
                })
                .unwrap(),
                finished: true,
            });
        }
    }

    fn continue_playing(
        &self,
        state: &crate::models::db_models::GameState,
        bet: &crate::models::json_requests::ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        let data: MinesContinueData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Mines data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        let mut parsed_state: MinesState = serde_json::from_str(&state.state)
            .map_err(|e| {
                error!("Error parsing Mines state`{:?}`: {:?}", state.state, e);
                e
            })
            .ok()?;
        let initial_bet_data: MinesData = serde_json::from_str(&state.bet_info)
            .map_err(|e| {
                error!(
                    "Error parsing Mines initial data`{:?}`: {:?}",
                    state.bet_info, e
                );
                e
            })
            .ok()?;

        let picked_tiles = if let Some(picked_tiles) = data.tiles {
            picked_tiles
        } else {
            // cashout
            let profit = parsed_state.current_multiplier * state.amount;
            return Some(GameResult {
                total_profit: profit,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![profit],
                num_games: 1,
                data: serde_json::to_string(&MinesState {
                    state: parsed_state.state,
                    mines: parsed_state.mines,
                    game_num: 1,
                    current_multiplier: parsed_state.current_multiplier,
                })
                .unwrap(),
                finished: true,
            });
        };

        let mut num_tiles_revealed: usize = 0;
        let mut num_tiles_to_reveal: usize = 0;

        for (_, (picked_tile, revealed_tile)) in picked_tiles
            .iter()
            .zip(parsed_state.state.iter())
            .enumerate()
        {
            if *picked_tile {
                if *revealed_tile {
                    return None;
                }
                num_tiles_to_reveal += 1;
            }
            if *revealed_tile {
                num_tiles_revealed += 1;
            }
        }

        if num_tiles_to_reveal == 0
            || num_tiles_to_reveal + num_tiles_revealed
                > self.max_reveal[initial_bet_data.num_mines as usize] as usize
        {
            return None;
        }

        // logic
        let mut number_of_revealed_tiles: usize = 0;

        let mut number_of_mines_left: usize = initial_bet_data.num_mines as usize;

        //let mut mines: [bool; 25] = [false; 25];
        //let mut revealed_tiles: [bool; 25] = [false; 25];

        let mut won = true;

        for i in 0usize..25 {
            if number_of_mines_left == 0 || 25 - number_of_revealed_tiles == number_of_mines_left {
                if picked_tiles[i] {
                    parsed_state.state[i] = true;
                }
                continue;
            }

            if picked_tiles[i] {
                let gem = is_gem(
                    25 - number_of_revealed_tiles,
                    number_of_mines_left,
                    random_numbers[i] as usize,
                );

                if !gem {
                    number_of_mines_left -= 1;
                    parsed_state.mines[i] = true;
                    won = false;
                }

                parsed_state.state[i] = true;
                number_of_revealed_tiles += 1;
            }
        }

        if !won {
            return Some(GameResult {
                total_profit: Decimal::ZERO,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![Decimal::ZERO],
                num_games: parsed_state.game_num + 1,
                data: serde_json::to_string(&parsed_state).unwrap(),
                finished: true,
            });
        }

        let multiplier = if let Some(mults) = self.multipliers.get(&(number_of_mines_left as u64)) {
            mults[number_of_revealed_tiles - 1]
        } else {
            warn!("Multiplier `{:?}` not found", number_of_mines_left);
            return None;
        };

        if !data.cashout {
            let profit = multiplier * state.amount;
            return Some(GameResult {
                total_profit: profit,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![profit],
                num_games: parsed_state.game_num + 1,
                data: serde_json::to_string(&parsed_state).unwrap(),
                finished: false,
            });
        } else {
            let profit = multiplier * bet.amount;
            return Some(GameResult {
                total_profit: profit,
                outcomes: random_numbers.iter().cloned().collect(),
                profits: vec![profit],
                num_games: 1,
                data: serde_json::to_string(&MinesState {
                    state: revealed_tiles,
                    mines,
                    game_num: 1,
                    current_multiplier: multiplier,
                })
                .unwrap(),
                finished: true,
            });
        }

        None
    }

    fn numbers_per_bet(&self) -> u64 {
        25
    }
}

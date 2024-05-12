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
use tracing::error;

use super::StatefulGameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct BigSlotsData {
    buy_free_spins: bool,
    use_free_spins: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BigSlots {
    pub tiles: Vec<HashMap<u8, Decimal>>,
    pub multipliers: Vec<Decimal>,
    pub multiplier_chance: u64,
    pub free_spins_prices: HashMap<i64, Decimal>,
    pub free_spins_reward_amount: u32,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct BigSlotsContinueData {
    pub buy_free_spins: bool,
    pub use_free_spins: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct BigSlotsState {
    pub free_spins: u32,
    pub total_win: Decimal,
    pub game_fields: Option<Vec<Vec<Vec<u8>>>>,
    pub multipliers: Option<Vec<Decimal>>,
    pub total_win_per_tumble: Option<Vec<Decimal>>,
}

impl BigSlots {
    fn get_tile_multiplier(&self, index: usize, count: u8) -> Decimal {
        if index < self.tiles.len() {
            return self.tiles[index]
                .get(&count)
                .cloned()
                .unwrap_or(Decimal::ZERO);
        }

        return Decimal::ZERO;
    }
    fn play_normal_bet(
        &self,
        random_numbers: &[u64],
        bet_amount: Decimal,
    ) -> (Vec<Vec<Vec<u8>>>, Decimal, Vec<Decimal>, Vec<Decimal>, u32) {
        let mut random_number_index = 0;
        let mut to_return: Vec<Vec<Vec<u8>>> = Vec::new();
        let mut total_win_per_tumble: Vec<Decimal> = Vec::new();
        let mut multipliers: Vec<Decimal> = Vec::new();
        let mut total_win: Decimal = Decimal::ZERO;
        let mut free_spins: u32 = 0;

        let mut play_field: Vec<Vec<u8>> = Vec::with_capacity(6);
        for _ in 0..6 {
            play_field.push(vec![0; 5]);
        }

        let mut tiles_count: Vec<u8> = vec![0; self.tiles.len() + self.multipliers.len()];
        for column_i in 0..6 {
            for row_i in 0..5 {
                let random_number = random_numbers[random_number_index];
                let lucky_factor = random_numbers[random_number_index] % 10000;

                let multiplier_id = if lucky_factor < self.multiplier_chance {
                    let id = random_number % self.multipliers.len() as u64;
                    multipliers.push(self.multipliers[id as usize]);
                    (id as u8) + self.tiles.len() as u8
                } else if lucky_factor < 200 {
                    self.tiles.len() as u8 - 1
                } else {
                    (random_number % 9) as u8
                };
                tiles_count[multiplier_id as usize] += 1;
                play_field[column_i][row_i] = multiplier_id;
                random_number_index += 1;
            }
        }
        to_return.push(play_field.clone());

        loop {
            let mut tumble = false;
            let mut current_tumble_win: Decimal = Decimal::ZERO;
            // scatter
            if *tiles_count.get(9).unwrap_or(&0) >= 4 {
                println!("Scatter");
                //free_spins = self.free_spins_reward_amount;
            }
            for (index, tile_count) in tiles_count.iter_mut().enumerate() {
                let multiplier = self.get_tile_multiplier(index, *tile_count);
                if !multiplier.is_zero() {
                    for column in play_field.iter_mut() {
                        column.retain(|el| *el as usize != index);
                    }
                    current_tumble_win += bet_amount * multiplier;
                    tumble = true;
                    *tile_count = 0;
                }
            }
            total_win += current_tumble_win;
            total_win_per_tumble.push(current_tumble_win);

            if !tumble {
                break;
            }

            for column in play_field.iter_mut() {
                for _ in 0..5 - column.len() {
                    let random_number = random_numbers[random_number_index];
                    let lucky_factor = random_numbers[random_number_index] % 10000;

                    let multiplier_id = if lucky_factor < self.multiplier_chance {
                        let id = random_number % self.multipliers.len() as u64;
                        multipliers.push(self.multipliers[id as usize]);
                        (id as u8) + self.tiles.len() as u8
                    } else if lucky_factor < 100 {
                        self.tiles.len() as u8 - 1
                    } else if lucky_factor < 6000 {
                        ((random_number % 3) as u8)
                    } else {
                        ((random_number % 3) as u8) + 6
                    };
                    //else if lucky_factor < 3000 {
                    //    ((random_number % 3) as u8) + 3
                    //}
                    tiles_count[multiplier_id as usize] += 1;
                    column.push(multiplier_id);
                    random_number_index += 1;
                }
            }

            to_return.push(play_field.clone());
        }

        let total_multiplier: Decimal = multipliers.iter().sum();

        if !total_multiplier.is_zero() {
            total_win *= total_multiplier;
        }

        (
            to_return,
            total_win,
            multipliers,
            total_win_per_tumble,
            free_spins,
        )
    }
}

impl StatefulGameEng for BigSlots {
    fn start_playing(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: BigSlotsData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing BigSlots data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.use_free_spins {
            return None;
        }

        if data.buy_free_spins {
            if let Some(price) = self.free_spins_prices.get(&bet.coin_id) {
                if !bet.amount.eq(price) {
                    return Some(GameResult {
                        total_profit: Decimal::ZERO,
                        outcomes: Vec::with_capacity(0),
                        profits: Vec::with_capacity(0),
                        num_games: 1,
                        data: serde_json::to_string(&BigSlotsState {
                            free_spins: self.free_spins_reward_amount,
                            total_win: Decimal::ZERO,
                            game_fields: None,
                            multipliers: None,
                            total_win_per_tumble: None,
                        })
                        .unwrap(),
                        finished: false,
                    });
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        let (game_fields, total_win, multipliers, total_win_per_tumble, free_spins) =
            self.play_normal_bet(random_numbers, bet.amount);

        if free_spins != 0 {
            return Some(GameResult {
                total_profit: total_win,
                outcomes: Vec::with_capacity(0),
                profits: total_win_per_tumble.clone(),
                num_games: 1,
                data: serde_json::to_string(&BigSlotsState {
                    free_spins: self.free_spins_reward_amount,
                    total_win,
                    game_fields: Some(game_fields),
                    multipliers: Some(multipliers),
                    total_win_per_tumble: Some(total_win_per_tumble),
                })
                .unwrap(),
                finished: false,
            });
        }
        return Some(GameResult {
            total_profit: total_win,
            outcomes: Vec::with_capacity(0),
            profits: total_win_per_tumble.clone(),
            num_games: 1,
            data: serde_json::to_string(&BigSlotsState {
                free_spins: self.free_spins_reward_amount,
                total_win,
                game_fields: Some(game_fields),
                multipliers: Some(multipliers),
                total_win_per_tumble: Some(total_win_per_tumble),
            })
            .unwrap(),
            finished: true,
        });
    }

    fn continue_playing(
        &self,
        state: &crate::models::db_models::GameState,
        bet: &crate::models::json_requests::ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        todo!()
    }

    fn numbers_per_bet(&self) -> u64 {
        200
    }
}

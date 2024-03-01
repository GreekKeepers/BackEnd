use std::str::FromStr;

use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::models::{db_models::GameResult, json_requests::PropagatedBet};

use lazy_static::lazy_static;
lazy_static! {
    static ref DICE_LOWER_BOUNDARY: Decimal = Decimal::from_str("1.0421").unwrap();
    static ref DICE_UPPER_BOUNDARY: Decimal = Decimal::from_str("99.9999").unwrap();
    static ref DICE_MULT: Decimal = Decimal::from(10000);
    static ref U64_UPPER_BOUNDARY: Decimal = Decimal::from(18446744073709551615u64);
    static ref HUNDRED: Decimal = Decimal::from(100);
    static ref NINTYNINE: Decimal = Decimal::from(99);
}

pub trait GameEng {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult>;

    fn numbers_per_bet(&self) -> u64;
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct CoinFlipData {
    pub is_heads: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct CoinFlip {
    pub profit_coef: Decimal,
}

impl GameEng for CoinFlip {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: CoinFlipData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing CoinFlip data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let profit = bet.amount * self.profit_coef;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let side = (number % 2) as u32;
            outcomes.push(side);

            if (data.is_heads && side == 1) || (!data.is_heads && side == 0) {
                total_profit += profit;
                total_value += profit;
            } else {
                total_value -= bet.amount;
            }

            games = game + 1;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        if games != bet.num_games as usize {
            total_profit += Decimal::from(bet.num_games as usize - games) * bet.amount;
        }

        Some(GameResult {
            total_profit,
            outcomes,
            num_games: games as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct DiceData {
    pub roll_over: bool,
    pub multiplier: Decimal,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Dice {
    pub profit_coef: Decimal,
}

pub fn remap(
    number: Decimal,
    from: Decimal,
    to: Decimal,
    map_from: Decimal,
    map_to: Decimal,
) -> Decimal {
    (number - from) / (to - from) * (map_to - map_from) + map_from
}

impl GameEng for Dice {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: DiceData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Dice data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        if data.multiplier > *DICE_UPPER_BOUNDARY || data.multiplier < *DICE_LOWER_BOUNDARY {
            return None;
        }

        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let number_to_roll = *HUNDRED - (*NINTYNINE / data.multiplier);
        let profit = bet.amount * data.multiplier;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let number = remap(
                Decimal::from_u64(*number).unwrap(),
                Decimal::ZERO,
                *U64_UPPER_BOUNDARY,
                *DICE_LOWER_BOUNDARY,
                *DICE_UPPER_BOUNDARY,
            );
            outcomes.push((number * *DICE_MULT).to_u32().unwrap());

            if (data.roll_over && number >= number_to_roll)
                || (!data.roll_over && number <= number_to_roll)
            {
                total_profit += profit;
                total_value += profit;
            } else {
                total_value -= bet.amount;
            }

            games = game + 1;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        if games != bet.num_games as usize {
            total_profit += Decimal::from(bet.num_games as usize - games) * bet.amount;
        }

        Some(GameResult {
            total_profit,
            outcomes,
            num_games: games as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RPSData {
    pub action: u32, // 0 - rock, 1 - paper, 2 - scissors
}

fn rps_outcome(player: u32, rng: u32) -> u32 {
    if player == rng {
        return 2;
    }
    if player == 0 {
        if rng == 1 {
            return 0;
        } else {
            return 1;
        }
    }

    if player == 1 {
        if rng == 2 {
            return 0;
        } else {
            return 1;
        }
    }

    if player == 2 {
        if rng == 0 {
            return 0;
        } else {
            return 1;
        }
    }

    panic!("Bad input {}, {}", player, rng);
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RPS {
    pub profit_coef: Decimal,
    pub draw_coef: Decimal,
}

impl GameEng for RPS {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: RPSData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing RPS data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        if data.action > 2 {
            return None;
        }

        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let profit = bet.amount * self.profit_coef;
        let draw = bet.amount * self.draw_coef;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let action = (number % 3) as u32;
            outcomes.push(action);

            let rps_result = rps_outcome(data.action, action);

            if rps_result == 2 {
                total_profit += draw;
                total_value += draw;
            } else if rps_result == 1 {
                total_profit += profit;
                total_value += profit;
            } else {
                total_value -= bet.amount;
            }

            games = game + 1;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        if games != bet.num_games as usize {
            total_profit += Decimal::from(bet.num_games as usize - games) * bet.amount;
        }

        Some(GameResult {
            total_profit,
            outcomes,
            num_games: games as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RaceData {
    pub car: u64,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Race {
    pub profit_coef: Decimal,
    pub cars_amount: u64,
}

impl GameEng for Race {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: RaceData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Race data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.car >= self.cars_amount {
            return None;
        }

        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let profit = bet.amount * self.profit_coef;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let winner_car = (number % self.cars_amount) as u32;
            outcomes.push(winner_car);

            if data.car as u32 == winner_car {
                total_profit += profit;
                total_value += profit;
            } else {
                total_value -= bet.amount;
            }

            games = game + 1;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        if games != bet.num_games as usize {
            total_profit += Decimal::from(bet.num_games as usize - games) * bet.amount;
        }

        Some(GameResult {
            total_profit,
            outcomes,
            num_games: games as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

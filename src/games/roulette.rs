use crate::models::{db_models::GameResult, json_requests::PropagatedBet};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use rust_decimal::Decimal;
use tracing::error;

use crate::games::GameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum RouletteBetType {
    /// false - red, true - black
    Color(bool),
    Zero,
    Number(u8),
    Number2 {
        numbers: [u8; 2],
    },
    Number4 {
        numbers: [u8; 4],
    },
    /// false - 1-18
    Number18(bool),
    /// false - odd
    OddEven(bool),
    /// 0 - 1-12, 1 - 13-24
    Number12(u8),
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RouletteBet {
    pub amount: Decimal,
    pub bet: RouletteBetType,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RouletteData {
    bets: Vec<RouletteBet>,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Roulette {
    pub zero_coef: Decimal,
    pub num_coef: Decimal,
    pub num2_coef: Decimal,
    pub num4_coef: Decimal,
    pub num18_coef: Decimal,
    pub num12_coef: Decimal,
}

impl GameEng for Roulette {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: RouletteData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Roulette data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        let calculated_amount: Decimal = data.bets.iter().map(|b| b.amount).sum();
        if calculated_amount != bet.amount {
            return None;
        }

        let outcome = random_numbers[0] % 36;

        let mut profit = Decimal::ZERO;

        for bet in data.bets {
            match bet.bet {
                RouletteBetType::Color(color) => {
                    if outcome != 0 && (color && outcome % 2 == 0) || (!color && outcome % 2 == 1) {
                        profit += bet.amount * self.num18_coef;
                    }
                }
                RouletteBetType::Zero => {
                    if outcome == 0 {
                        profit += bet.amount * self.zero_coef;
                    }
                }
                RouletteBetType::Number(number) => {
                    if outcome == number as u64 {
                        profit += bet.amount * self.num_coef;
                    }
                }
                RouletteBetType::Number2 { numbers } => {
                    if numbers.contains(&(outcome as u8)) {
                        profit += bet.amount * self.num2_coef;
                    }
                }
                RouletteBetType::Number4 { numbers } => {
                    if numbers.contains(&(outcome as u8)) {
                        profit += bet.amount * self.num4_coef;
                    }
                }
                RouletteBetType::Number18(sector) => {
                    if (sector && 1 <= outcome && outcome <= 18)
                        || (!sector && 19 <= outcome && outcome <= 36)
                    {
                        profit += bet.amount * self.num18_coef;
                    }
                }
                RouletteBetType::OddEven(is_even) => {
                    if outcome != 0 && (!is_even && outcome % 2 == 1)
                        || (is_even && outcome % 2 == 0)
                    {
                        profit += bet.amount * self.num18_coef;
                    }
                }
                RouletteBetType::Number12(sector) => match sector {
                    0 => {
                        if 1 <= outcome && outcome <= 12 {
                            profit += bet.amount * self.num12_coef;
                        }
                    }
                    1 => {
                        if 13 <= outcome && outcome <= 24 {
                            profit += bet.amount * self.num12_coef;
                        }
                    }
                    2 => {
                        if 25 <= outcome && outcome <= 36 {
                            profit += bet.amount * self.num12_coef;
                        }
                    }
                    _ => {
                        return None;
                    }
                },
            }
        }

        Some(GameResult {
            total_profit: profit,
            outcomes: vec![outcome],
            profits: vec![profit],
            num_games: 1,
            data: bet.data.clone(),
            finished: true,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

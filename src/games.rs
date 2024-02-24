use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{db_models::GameResult, json_requests::PropagatedBet};

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
        let data: CoinFlipData = serde_json::from_str(&bet.data).ok()?;
        let mut total_profit = Decimal::ZERO;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        for number in random_numbers {
            let side = (number % 2) as u32;
            outcomes.push(side);

            if (data.is_heads && side == 1) || (!data.is_heads && side == 0) {
                total_profit += bet.amount * self.profit_coef;
            }
        }

        Some(GameResult {
            total_profit,
            outcomes,
            num_games: random_numbers.len() as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

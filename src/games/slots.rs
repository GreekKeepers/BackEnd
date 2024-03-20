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

use crate::games::GameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct SlotsData {}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Slots {
    pub num_outcomes: u32,
    pub multipliers: Vec<Decimal>,
}

impl GameEng for Slots {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let mut outcomes: Vec<u64> = Vec::with_capacity(bet.num_games as usize);
        let mut profits: Vec<Decimal> = Vec::with_capacity(bet.num_games as usize);

        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0u32;

        for number in random_numbers.iter() {
            let slot_id = (number % self.num_outcomes as u64) as u64;
            let multiplier = self.multipliers[slot_id as usize];

            let profit = bet.amount * multiplier;
            outcomes.push(multiplier.try_into().unwrap());

            if !multiplier.is_zero() {
                total_profit += profit;
                total_value += profit;
                profits.push(profit);
            } else {
                total_value -= bet.amount;
                profits.push(Decimal::ZERO);
            }

            games += 1;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        Some(GameResult {
            total_profit,
            outcomes,
            profits,
            num_games: games as u32,
            data: bet.data.clone(),
            finished: true,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

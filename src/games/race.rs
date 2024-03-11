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

use super::GameType;

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
    fn play(
        &self,
        prev_bet: Option<&Bet>,
        bet: &PropagatedBet,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
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
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let winner_car = (number % self.cars_amount) as u32;
            outcomes.push(winner_car);

            if data.car as u32 == winner_car {
                total_profit += profit;
                total_value += profit;
                profits.push(profit);
            } else {
                total_value -= bet.amount;
                profits.push(Decimal::ZERO);
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
            profits,
            num_games: games as u32,
            data: bet.data.clone(),
            finished: true,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }

    fn get_type(&self) -> GameType {
        GameType::Stateless
    }
}

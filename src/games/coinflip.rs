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

        let mut outcomes: Vec<u64> = Vec::with_capacity(random_numbers.len());
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let side = (number % 2) as u64;
            outcomes.push(side);

            if (data.is_heads && side == 1) || (!data.is_heads && side == 0) {
                total_profit += profit;
                total_value += profit;
                profits.push(profit);
            } else {
                total_value -= bet.amount;
                profits.push(Decimal::ZERO);
            }

            games = games + 1;

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
}

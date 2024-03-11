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
pub struct WheelData {
    pub risk: u32,
    pub num_sectors: u32,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Wheel {
    pub multipliers: Vec<Vec<Vec<Decimal>>>,
    pub max_risk: u32,
    pub max_num_sectors: u32,
}

impl GameEng for Wheel {
    fn play(
        &self,
        prev_bet: Option<&Bet>,
        bet: &PropagatedBet,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        let data: WheelData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Wheel data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.risk > self.max_risk || data.num_sectors > self.max_num_sectors {
            return None;
        }

        let multipliers = &self.multipliers[data.risk as usize][data.num_sectors as usize];

        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let num_sectors = (data.num_sectors + 1) * 10;

        let mut outcomes: Vec<u32> = Vec::with_capacity(random_numbers.len());
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let sector = (number % num_sectors as u64) as u32;
            outcomes.push(sector);

            let multiplier = multipliers[sector as usize];

            if multiplier.is_zero() {
                total_value -= bet.amount;
                profits.push(Decimal::ZERO);
            } else {
                let profit = bet.amount * multiplier;
                total_profit += profit;
                total_value += profit;
                profits.push(profit);
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

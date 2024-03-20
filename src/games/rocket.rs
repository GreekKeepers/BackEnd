use crate::{
    db::DB,
    models::{
        db_models::{Bet, GameResult},
        json_requests::PropagatedBet,
    },
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use tracing::error;

use crate::games::GameEng;

use lazy_static::lazy_static;

lazy_static! {
    static ref DICE_LOWER_BOUNDARY: Decimal = Decimal::from_str("1.0421").unwrap();
    static ref DICE_UPPER_BOUNDARY: Decimal = Decimal::from_str("99.9999").unwrap();
    static ref DICE_MULT: Decimal = Decimal::from(10000);
    static ref U64_UPPER_BOUNDARY: Decimal = Decimal::from(18446744073709551615u64);
    static ref HUNDRED: Decimal = Decimal::from(100);
    static ref NINTYNINE: Decimal = Decimal::from(99);
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

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RocketData {
    pub multiplier: Decimal,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Rocket {
    pub profit_coef: Decimal,
}

impl GameEng for Rocket {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: RocketData = serde_json::from_str(&bet.data)
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

        let mut outcomes: Vec<u64> = Vec::with_capacity(random_numbers.len());
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let number = remap(
                Decimal::from_u64(*number).unwrap(),
                Decimal::ZERO,
                *U64_UPPER_BOUNDARY,
                *DICE_LOWER_BOUNDARY,
                *DICE_UPPER_BOUNDARY,
            );
            outcomes.push((number * *DICE_MULT).to_u64().unwrap());

            if number >= number_to_roll {
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
}

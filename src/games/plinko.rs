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
pub struct PlinkoData {
    pub num_rows: u64,
    pub risk: u64,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct PlinkoReturnData {
    pub num_rows: u64,
    pub risk: u64,
    pub paths: Vec<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Plinko {
    pub multipliers: [[Vec<Decimal>; 9]; 3],
}

impl Plinko {
    pub fn plinko_game(&self, rng: u64, num_rows: u64, risk: u64) -> (Decimal, Vec<u8>) {
        let mut result = Vec::with_capacity(num_rows as usize);

        let mut mask = 0x8000000000000000u64;
        let mut ended: i8 = 0;
        for _ in 0..num_rows {
            let res = if rng & mask > 0 {
                ended += 1;
                1u8
            } else {
                ended -= 1;
                0u8
            };
            mask >>= 1;
            result.push(res);
        }

        let slot = (ended + num_rows as i8) as u8 >> 1;
        let multiplier = self.multipliers[risk as usize][num_rows as usize - 8][slot as usize];

        (multiplier, result)
    }
}

impl GameEng for Plinko {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let data: PlinkoData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Plinko data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;

        if data.num_rows < 8 || data.num_rows > 16 {
            return None;
        }
        if data.risk >= 3 {
            return None;
        }
        let mut total_profit = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut games = 0;

        let mut outcomes: Vec<u64> = Vec::with_capacity(random_numbers.len());
        let mut paths: Vec<Vec<u8>> = Vec::with_capacity(random_numbers.len());
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (_, number) in random_numbers.iter().enumerate() {
            let (multiplier, path) = self.plinko_game(*number, data.num_rows, data.risk);
            let payout = bet.amount * multiplier;

            paths.push(path);
            profits.push(payout);
            games += 1;
            outcomes.push(*number);

            total_profit += payout;
            total_value += payout - bet.amount;

            if (!bet.stop_win.is_zero() && total_value >= bet.stop_win)
                || (!bet.stop_loss.is_zero() && total_value <= bet.stop_loss)
            {
                break;
            }
        }

        if games != bet.num_games as usize {
            total_profit += Decimal::from(bet.num_games as usize - games) * bet.amount;
        }

        let return_data = serde_json::to_string(&PlinkoReturnData {
            num_rows: data.num_rows,
            risk: data.risk,
            paths,
        })
        .unwrap();

        Some(GameResult {
            total_profit,
            outcomes,
            profits,
            num_games: games as u32,
            data: return_data,
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

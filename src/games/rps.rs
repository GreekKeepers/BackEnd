use crate::models::{db_models::GameResult, json_requests::PropagatedBet};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use rust_decimal::Decimal;
use tracing::error;

use crate::games::GameEng;

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
        let mut profits: Vec<Decimal> = Vec::with_capacity(random_numbers.len());
        for (game, number) in random_numbers.iter().enumerate() {
            let action = (number % 3) as u32;
            outcomes.push(action);

            let rps_result = rps_outcome(data.action, action);

            if rps_result == 2 {
                total_profit += draw;
                total_value += draw;
                profits.push(draw);
            } else if rps_result == 1 {
                total_profit += profit;
                total_value += profit;
                profits.push(profit);
            } else {
                profits.push(Decimal::ZERO);
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
            profits,
            num_games: games as u32,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        1
    }
}

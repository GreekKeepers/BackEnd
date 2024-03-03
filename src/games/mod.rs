mod coinflip;
pub use coinflip::*;

mod dice;
pub use dice::*;

mod rps;
pub use rps::*;

mod race;
pub use race::*;

mod wheel;
pub use wheel::*;

use crate::models::{db_models::GameResult, json_requests::PropagatedBet};

pub trait GameEng {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult>;

    fn numbers_per_bet(&self) -> u64;
}

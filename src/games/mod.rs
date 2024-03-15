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

mod statefultest;
pub use statefultest::*;

mod poker;
pub use poker::*;

mod mines;
pub use mines::*;

mod rocket;
pub use rocket::*;

mod plinko;
pub use plinko::*;

mod apples;
pub use apples::*;

use crate::{
    db::DB,
    models::{
        db_models::{Bet, GameResult, GameState},
        json_requests::{ContinueGame, PropagatedBet},
    },
};

pub trait GameEng {
    fn play(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult>;

    fn numbers_per_bet(&self) -> u64;

    fn get_type(&self) -> GameType;
}

pub trait StatefulGameEng {
    fn start_playing(&self, bet: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult>;

    fn continue_playing(
        &self,
        state: &GameState,
        bet: &ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult>;

    fn numbers_per_bet(&self) -> u64;
}

pub enum GameType {
    Stateful,
    Stateless,
}

use std::collections::HashMap;

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
use tracing::{error, warn};

use super::StatefulGameEng;

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct PokerData {}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct PokerContinueData {
    pub tiles: Option<[bool; 25]>,
    pub cashout: bool,
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Copy)]
pub struct Card {
    pub number: u8,
    pub suit: u8,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct PokerState {
    pub cards_in_hand: [Card; 5],
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Poker {
    pub initial_deck: Vec<Card>,
    pub multipliers: [Decimal; 10],
}

pub fn pick_card(rng: u64, deck: &mut Vec<Card>) -> Card {
    let position = rng as usize % deck.len();
    let card = deck[position];

    deck[position] = deck[deck.len() - 1];
    deck.truncate(deck.len() - 1);

    card
}

impl StatefulGameEng for Poker {
    fn start_playing(&self, _: &PropagatedBet, random_numbers: &[u64]) -> Option<GameResult> {
        let mut deck = self.initial_deck.clone();

        let mut cards_in_hand = [Card { number: 0, suit: 0 }; 5];
        cards_in_hand
            .iter_mut()
            .zip(random_numbers.iter().map(|rng| pick_card(*rng, &mut deck)))
            .for_each(|(orig, new)| *orig = new);
        return Some(GameResult {
            total_profit: Decimal::ZERO,
            outcomes: random_numbers.iter().cloned().collect(),
            profits: vec![Decimal::ZERO],
            num_games: 1,
            data: serde_json::to_string(&PokerState { cards_in_hand }).unwrap(),
            finished: true,
        });
    }

    fn continue_playing(
        &self,
        state: &crate::models::db_models::GameState,
        bet: &crate::models::json_requests::ContinueGame,
        random_numbers: &[u64],
    ) -> Option<GameResult> {
        let data: PokerContinueData = serde_json::from_str(&bet.data)
            .map_err(|e| {
                error!("Error parsing Mines data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        let mut parsed_state: PokerState = serde_json::from_str(&state.state)
            .map_err(|e| {
                error!("Error parsing Mines state`{:?}`: {:?}", state.state, e);
                e
            })
            .ok()?;
    }

    fn numbers_per_bet(&self) -> u64 {
        5
    }
}

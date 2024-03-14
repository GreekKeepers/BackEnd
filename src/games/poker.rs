use std::collections::HashMap;

use crate::{
    db::DB,
    models::{
        db_models::{Bet, Game, GameResult},
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
    pub to_replace: Option<[bool; 5]>,
    //pub cashout: bool,
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
            finished: false,
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
                error!("Error parsing Poker data `{:?}`: {:?}", bet.data, e);
                e
            })
            .ok()?;
        let mut parsed_state: PokerState = serde_json::from_str(&state.state)
            .map_err(|e| {
                error!("Error parsing Poker state`{:?}`: {:?}", state.state, e);
                e
            })
            .ok()?;

        if !data.to_replace.is_none() {
            let mut deck = self.initial_deck.clone();

            for (hand_card, to_replace) in parsed_state
                .cards_in_hand
                .iter()
                .zip(data.to_replace.unwrap().iter())
            {
                if *to_replace {
                    continue;
                }

                let last_card = deck[deck.len() - 1];

                for deck_card in deck.iter_mut() {
                    if deck_card.number == hand_card.number && deck_card.suit == hand_card.suit {
                        *deck_card = last_card;
                        deck.truncate(deck.len() - 1);
                        break;
                    }
                }
            }

            for ((to_replace, rng), card_in_hand) in data
                .to_replace
                .unwrap()
                .iter()
                .zip(random_numbers.iter())
                .zip(parsed_state.cards_in_hand.iter_mut())
            {
                if *to_replace {
                    *card_in_hand = pick_card(*rng, &mut deck);
                }
            }
        }

        let (multiplier, outcome) = determine_payout(parsed_state.cards_in_hand);
        let profit = state.amount * multiplier;
        Some(GameResult {
            total_profit: profit,
            outcomes: vec![outcome as u64],
            profits: vec![profit],
            num_games: 1,
            data: serde_json::to_string(&parsed_state).unwrap(),
            finished: true,
        })
    }

    fn numbers_per_bet(&self) -> u64 {
        5
    }
}

pub fn determine_payout(mut sorted_cards: [Card; 5]) -> (Decimal, u32) {
    sorted_cards.sort_unstable_by(|card_left, card_right| {
        match card_left.number.cmp(&card_right.number) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    });

    //check 4 of a kind
    if (sorted_cards[1].number == sorted_cards[2].number
        && sorted_cards[2].number == sorted_cards[3].number)
    {
        if (sorted_cards[1].number == sorted_cards[0].number
            || sorted_cards[3].number == sorted_cards[4].number)
        {
            return (Decimal::from(30), 7);
        }
    }
    //check full house -> 3 of a kind + pair
    if (sorted_cards[1].number == sorted_cards[0].number
        && sorted_cards[4].number == sorted_cards[3].number)
    {
        if (sorted_cards[1].number == sorted_cards[2].number
            || sorted_cards[3].number == sorted_cards[2].number)
        {
            return (Decimal::from(8), 6);
        }
    }
    //check royal flush + straight flush + flush
    if (sorted_cards[0].suit == sorted_cards[1].suit
        && sorted_cards[2].suit == sorted_cards[3].suit
        && sorted_cards[0].suit == sorted_cards[4].suit
        && sorted_cards[2].suit == sorted_cards[1].suit)
    {
        if (sorted_cards[0].number == 1 && sorted_cards[4].number == 13) {
            if (sorted_cards[2].number == sorted_cards[3].number - 1
                && sorted_cards[3].number == sorted_cards[4].number - 1
                && sorted_cards[1].number == sorted_cards[2].number - 1)
            {
                return (Decimal::from(100), 9);
            }
        }
        if (sorted_cards[0].number == 1 && sorted_cards[1].number == 2) {
            if (sorted_cards[0].number == sorted_cards[1].number - 1
                && sorted_cards[2].number == sorted_cards[3].number - 1
                && sorted_cards[3].number == sorted_cards[4].number - 1
                && sorted_cards[1].number == sorted_cards[2].number - 1)
            {
                return (Decimal::from(50), 8);
            }
        }
        if (sorted_cards[0].number == sorted_cards[1].number - 1
            && sorted_cards[2].number == sorted_cards[3].number - 1
            && sorted_cards[3].number == sorted_cards[4].number - 1
            && sorted_cards[1].number == sorted_cards[2].number - 1)
        {
            return (Decimal::from(50), 8);
        }
        return (Decimal::from(6), 5);
    }

    //check straight
    if (sorted_cards[0].number == 1 && sorted_cards[1].number == 2) {
        if (sorted_cards[0].number == sorted_cards[1].number - 1
            && sorted_cards[2].number == sorted_cards[3].number - 1
            && sorted_cards[3].number == sorted_cards[4].number - 1
            && sorted_cards[1].number == sorted_cards[2].number - 1)
        {
            return (Decimal::from(5), 4);
        }
    }
    if (sorted_cards[0].number == 1 && sorted_cards[4].number == 13) {
        if (sorted_cards[2].number == sorted_cards[3].number - 1
            && sorted_cards[3].number == sorted_cards[4].number - 1
            && sorted_cards[1].number == sorted_cards[2].number - 1)
        {
            return (Decimal::from(5), 4);
        }
    }
    if (sorted_cards[0].number == sorted_cards[1].number - 1
        && sorted_cards[1].number == sorted_cards[2].number - 1
        && sorted_cards[2].number == sorted_cards[3].number - 1
        && sorted_cards[3].number == sorted_cards[4].number - 1)
    {
        return (Decimal::from(5), 4);
    }
    //check three of a kind
    if (sorted_cards[0].number == sorted_cards[1].number
        && sorted_cards[1].number == sorted_cards[2].number)
    {
        return (Decimal::from(3), 3);
    }
    if (sorted_cards[1].number == sorted_cards[2].number
        && sorted_cards[2].number == sorted_cards[3].number)
    {
        return (Decimal::from(3), 3);
    }
    if (sorted_cards[2].number == sorted_cards[3].number
        && sorted_cards[3].number == sorted_cards[4].number)
    {
        return (Decimal::from(3), 3);
    }
    //check two pair
    if (sorted_cards[0].number == sorted_cards[1].number) {
        if (sorted_cards[2].number == sorted_cards[3].number
            || sorted_cards[3].number == sorted_cards[4].number)
        {
            return (Decimal::from(2), 2);
        }
    }

    if (sorted_cards[1].number == sorted_cards[2].number) {
        if (sorted_cards[3].number == sorted_cards[4].number) {
            return (Decimal::from(2), 2);
        }
    }
    //check one pair jacks or higher
    if (sorted_cards[0].number == sorted_cards[1].number) {
        if (sorted_cards[0].number > 10 || sorted_cards[0].number == 1) {
            return (Decimal::from(1), 1);
        }
    }
    if (sorted_cards[1].number == sorted_cards[2].number) {
        if (sorted_cards[1].number > 10 || sorted_cards[1].number == 1) {
            return (Decimal::from(1), 1);
        }
    }
    if (sorted_cards[2].number == sorted_cards[3].number) {
        if (sorted_cards[2].number > 10 || sorted_cards[2].number == 1) {
            return (Decimal::from(1), 1);
        }
    }
    if (sorted_cards[3].number == sorted_cards[4].number) {
        if (sorted_cards[3].number > 10 || sorted_cards[3].number == 1) {
            return (Decimal::from(1), 1);
        }
    }

    (Decimal::ZERO, 0)
}

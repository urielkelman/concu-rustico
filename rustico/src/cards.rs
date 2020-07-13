extern crate rand;
use rand::seq::SliceRandom;

#[derive(Copy, Clone)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamonds,
    Clubs
}

#[derive(Copy, Clone)]
pub struct Card {
    pub number: i32,
    pub suit: CardSuit
}

pub fn random_full_deck() -> Vec<Card>{
    let mut full_deck = Vec::new();
    let mut rng = &mut rand::thread_rng();
    let suits = [CardSuit::Spades, CardSuit::Hearts,
                CardSuit::Diamonds, CardSuit::Clubs];
    for suit in suits.iter(){
        for i in 1..14{
            full_deck.push(Card{number: i, suit: *suit});
        }
    }
    full_deck = full_deck.choose_multiple(&mut rng, full_deck.len()).map(|x| *x).collect();
    return full_deck;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_empty_points_map_len() {
        let deck = random_full_deck();
        assert_eq!(deck.len(), 52);
    }
}

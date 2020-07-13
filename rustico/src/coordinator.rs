use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Barrier, Arc};
use std::collections::HashMap;

use crate::signed_card::SignedCard;
use crate::cards::{Card, random_full_deck};

const FRENCH_DECK_SIZE :i32 = 52;

const POINTS_FASTER_PLAYER :i32 = 1;
const POINTS_SLOWER_PLAYER :i32 = -5;
const POINTS_MAX_CARD :i32 = 10;


fn deal_cards_to_players(players: i32, tx_deck: Sender<Vec<Card>>) -> (i32, i32){
    let deck_size = FRENCH_DECK_SIZE / players;
    let unused_cards = FRENCH_DECK_SIZE % players;
    let full_deck = random_full_deck();

    for p in 0..players {
        let lower_bound = p*deck_size;
        let upper_bound = (p+1)*deck_size;
        let cards = &full_deck[lower_bound as usize..upper_bound as usize];
        tx_deck.send(cards.to_vec()).unwrap();
    }

    return (deck_size, unused_cards);
}

fn empty_points_map(players: i32) -> HashMap<i32, i32>{
    let mut points_by_user = HashMap::new();

    for p in 0..players {
        points_by_user.insert(p, 0);
    }

    return points_by_user
}

fn merge_points_hashmaps(map1: HashMap<i32, i32>, map2: HashMap<i32, i32>) -> HashMap<i32, i32>{
    let mut merged = HashMap::new();
    for key in map1.keys() {
        if map2.contains_key(key){
            merged.insert(*key, *map1.get(key).unwrap()+*map2.get(key).unwrap());
        }
    }
    return merged;
}

fn calculate_normal_hand_points(mut signed_cards: Vec<SignedCard>) -> HashMap<i32, i32>{
    let mut points_by_user = empty_points_map(signed_cards.len() as i32);

    signed_cards.sort_by(|a, b| a.card.number.cmp(&b.card.number));

    let max_card: Card = signed_cards.last().unwrap().card;
    for i in (0..signed_cards.len()).rev() {
        if signed_cards[i].card.number == max_card.number {
            points_by_user.insert(signed_cards[i].player_signature, POINTS_MAX_CARD);
        } else {
            break;
        }
    }

    return points_by_user;
}

fn calculate_rustic_hand_points(signed_cards: Vec<SignedCard>) -> HashMap<i32, i32> {
    let mut points_by_user = empty_points_map(signed_cards.len() as i32);

    points_by_user.insert(signed_cards.first().unwrap().player_signature, POINTS_FASTER_PLAYER);
    points_by_user.insert(signed_cards.last().unwrap().player_signature, POINTS_SLOWER_PLAYER);

    let normal_hand_points = calculate_normal_hand_points(signed_cards);

    return merge_points_hashmaps(normal_hand_points, points_by_user);
}

pub fn coordinator(players: i32, card_receiver: Receiver<SignedCard>, players_barrier: Arc<Barrier>, tx_deck :Sender<Vec<Card>>) {
    let (deck_size, unused_cards) = deal_cards_to_players(players, tx_deck);
    println!("Unused cards {}", unused_cards);

    let mut points_by_user = empty_points_map(players);

    for i in 0..deck_size{
        let mut cards = Vec::new();

        for _ in 0..players{
            cards.push(card_receiver.recv().unwrap());
        }

        let hand_points = calculate_rustic_hand_points(cards);
        points_by_user = merge_points_hashmaps(points_by_user, hand_points);

        players_barrier.wait();

        println!("Finish iteracion {}", i);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_empty_points_map_len() {
        let empty_map = empty_points_map(4);
        assert_eq!(empty_map.len(), 4);
    }

    #[test]
    fn test_merge_same_keys() {
        let mut map1 = HashMap::new();
        let mut map2 = HashMap::new();
        map1.insert(1,2);
        map2.insert(1,6);
        map1.insert(2,1);
        map2.insert(2,1);
        let merged_map = merge_points_hashmaps(map1, map2);
        assert_eq!(*merged_map.get(&(1 as i32)).unwrap(), 8 as i32);
        assert_eq!(*merged_map.get(&(2 as i32)).unwrap(), 2 as i32);
    }

    #[test]
    fn test_merge_same_keys_different_keys() {
        let mut map1 = HashMap::new();
        let mut map2 = HashMap::new();
        map1.insert(1,2);
        map2.insert(1,6);
        map1.insert(2,1);
        map2.insert(2,1);

        map1.insert(5,7);
        map2.insert(7,5);

        let merged_map = merge_points_hashmaps(map1, map2);
        assert_eq!(merged_map.len(), 2);
        assert_eq!(*merged_map.get(&(1 as i32)).unwrap(), 8 as i32);
        assert_eq!(*merged_map.get(&(2 as i32)).unwrap(), 2 as i32);
    }
}

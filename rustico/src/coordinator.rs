use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Barrier, Arc};
use std::collections::HashMap;
use rand::Rng;

use crate::signed_card::SignedCard;

const FRENCH_DECK_SIZE :i32 = 52;

const POINTS_FASTER_PLAYER :i32 = 1;
const POINTS_SLOWER_PLAYER :i32 = -5;


fn deal_cards_to_players(players: i32, tx_deck: Sender<Vec<i32>>) -> i32{
    let deck_size = FRENCH_DECK_SIZE / players;
    let unused_cards = FRENCH_DECK_SIZE % players;

    for p in 0..players {
        let mut rng = rand::thread_rng();
        let cards = (0..deck_size).map(|_| rng.gen_range(1, 12)).collect();
        tx_deck.send(cards).unwrap();
    }

    deck_size
}

fn empty_points_map(players: i32) -> HashMap<i32, i32>{
    let mut points_by_user = HashMap::new();

    for p in 0..players {
        points_by_user.insert(p, 0);
    }

    return points_by_user
}

fn calculate_normal_hand_points(mut signed_cards: Vec<SignedCard>){
    let mut points_by_user = empty_points_map(signed_cards.len() as i32);

    signed_cards.sort_by(|a, b| a.card.cmp(&b.card));



}

fn calculate_rustic_hand_points(mut signed_cards: Vec<SignedCard>) -> HashMap<i32, i32> {
    let mut points_by_user = empty_points_map(signed_cards.len() as i32);

    points_by_user.insert(signed_cards.first().player_signature, POINTS_FASTER_PLAYER);
    points_by_user.insert(signed_cards.last().player_signature, POINTS_SLOWER_PLAYER);


    for card in signed_cards{
        println!("card {}, player {}", card.card, card.player_signature);
    }

    return points_by_user;
}

pub fn coordinator(players: i32, card_receiver: Receiver<SignedCard>, players_barrier: Arc<Barrier>, tx_deck :Sender<Vec<i32>>) {
    let deck_size = deal_cards_to_players(players, tx_deck);

    let mut points_by_user = empty_points_map(players);

    for i in 0..deck_size{
        let mut cards = Vec::new();

        for p in 0..players{
            cards.push(card_receiver.recv().unwrap());
        }

        let hand_points = calculate_rustic_hand_points(cards);

        players_barrier.wait();

        println!("Finish iteracion {}", i);
    }
}
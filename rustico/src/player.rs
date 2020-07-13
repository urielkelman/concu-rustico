use std::sync::mpsc::{Sender, Receiver};

use std::sync::{Barrier, Arc, Mutex};

use crate::signed_card::SignedCard;
use crate::cards::Card;

fn receive_deck(rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>) -> Vec<Card> {
    return rx_deck.lock().unwrap().recv().unwrap();
}

pub fn player(card_sender: Sender<SignedCard>, players_barrier: Arc<Barrier>,
              rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>, player_id: i32) {

    let deck = receive_deck(rx_deck);

    for card in deck {
        card_sender.send(SignedCard { card, player_signature: player_id }).unwrap();
        players_barrier.wait();
    }

    println!("Barrera levantada");
}

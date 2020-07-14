use std::sync::mpsc::{Sender, Receiver};

use std::sync::{Barrier, Arc, Mutex, Condvar};

use crate::signed_card::SignedCard;
use crate::cards::Card;

use crate::logger::{LogFile, info, debug, error};

fn receive_deck(rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>) -> Vec<Card> {
    return rx_deck.lock().unwrap().recv().unwrap();
}

pub fn player(log: LogFile, card_sender: Sender<SignedCard>, players_barrier: Arc<Barrier>,
              rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>, cond_var: Arc<(Mutex<bool>, Condvar)>, player_id: i32) {

    let deck = receive_deck(rx_deck);

    let (lock, cvar) = &*cond_var;

    for card in deck {
        players_barrier.wait();

        let mut can_play = lock.lock().unwrap();

        while !*can_play {
            can_play = cvar.wait(can_play).unwrap();
        }

        card_sender.send(SignedCard { card, player_signature: player_id }).unwrap();

        debug(log.clone(), format!("Player {} throw the card with number {}", player_id, card.number));

        *can_play = false;
    }
}

use std::sync::mpsc::{Sender, Receiver};

use std::sync::{Barrier, Arc, Mutex, Condvar};

use crate::signed_card::SignedCard;
use crate::cards::Card;

use crate::logger::{LogFile, info, debug, error};

fn receive_deck(rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>) -> Vec<Card> {
    return rx_deck.lock().unwrap().recv().unwrap();
}

pub struct RoundPlayerFlags {
    pub is_my_turn: bool,
    pub can_throw_card: bool,
    pub game_ended: bool
}

pub fn player(log: LogFile, card_sender: Sender<SignedCard>, players_barrier: Arc<Barrier>,
              rx_deck: Arc<Mutex<Receiver<Vec<Card>>>>, cond_var: Arc<(Mutex<RoundPlayerFlags>, Condvar)>,
              player_id: i32) {

    let deck = receive_deck(rx_deck);

    let (lock, cvar) = &*cond_var;

    let mut cards_thrown: usize = 0;

    loop {
        debug(log.clone(), format!("jugador {} quiere bajar la barrera", player_id));
        players_barrier.wait();

        let mut round_player_flags = lock.lock().unwrap();

        while !(*round_player_flags).is_my_turn {
            round_player_flags = cvar.wait(round_player_flags).unwrap();
        }

        if (*round_player_flags).game_ended {
            break;
        }

        if (*round_player_flags).can_throw_card {
            card_sender.send(SignedCard { card: deck[cards_thrown], player_signature: player_id }).unwrap();
            cards_thrown += 1;
            debug(log.clone(), format!("El jugador {} tiró su carta número {}.", player_id, cards_thrown));
        } else {
            debug(log.clone(), format!("El jugador {} se encuentra suspendido, no tira carta en esta ronda.", player_id));
        }

        (*round_player_flags).is_my_turn = false;
        debug(log.clone(), format!("jugador {} suelta el turno", player_id));
    }
}

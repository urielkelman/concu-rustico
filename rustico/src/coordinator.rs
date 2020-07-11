use std::sync::mpsc::Receiver;
use std::sync::{Barrier, Arc};

pub fn coordinator(players: i32, card_receiver: Receiver<i32>, players_barrier: Arc<Barrier>) {
    let mut cards = Vec::new();

    for p in 0..players{
        cards.push(card_receiver.recv().unwrap());
    }

    for item in cards {
        println!("{}", item);
    }

    players_barrier.wait();
}
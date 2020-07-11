use std::sync::mpsc::Sender;
use std::sync::{Barrier, Arc};

pub fn player(card_sender: Sender<i32>, players_barrier: Arc<Barrier>) {
    card_sender.send(1);
    players_barrier.wait();
    println!("Barrera levantada");
}



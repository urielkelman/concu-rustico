extern crate clap;

mod player;
mod coordinator;
mod signed_card;
mod cards;

use clap::{App, Arg};
use std::sync::{mpsc, Barrier, Arc, Mutex};
use std::thread;
use player::player;
use coordinator::coordinator;


fn check_player_quantity(players: i32) -> bool {
    return players >= 4 && players % 2 == 0
}

fn set_up_threads(players: i32) {
    let barrier = Arc::new(Barrier::new((players + 1) as usize));
    let (tx_card, rx_card) = mpsc::channel();
    let (tx_deck, rx_deck) = mpsc::channel();
    let shared_rx_deck = Arc::new(Mutex::new(rx_deck));

    let mut threads = Vec::new();

    for p in 0..players {
        let tx_clone_player = mpsc::Sender::clone(&tx_card);
        let barrier_clone = barrier.clone();
        let shared_rx_deck_clone = shared_rx_deck.clone();
        threads.push(thread::spawn(move || {
            player(tx_clone_player, barrier_clone, shared_rx_deck_clone, p);
        }));
    }


    threads.push(thread::spawn(move || {
        coordinator(players, rx_card, barrier, tx_deck)
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}

fn main() {
    let matches = App::new("Rustico simulation")
        .version("1.0")
        .arg(Arg::with_name("players")
            .short("p")
            .long("players")
            .help("Number of players to participate in the game.")
            .takes_value(true)
            .required(true))
        .get_matches();

    let players: i32 = matches.value_of("players").unwrap().trim().parse().unwrap();

    if !check_player_quantity(players){
        println!("ERROR: Number of players should be greater or equal than four and divisible by two.");
        return;
    }

    set_up_threads(players);

    println!("{}", players);

    println!("Hello, world!");
}

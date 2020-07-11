extern crate clap;

mod player;
mod coordinator;

use clap::{App, Arg};
use std::sync::{mpsc, Barrier, Arc};
use std::thread;
use player::player;
use coordinator::coordinator;

fn check_player_quantity(players: i32) -> bool {
    return players >= 4 && players % 2 == 0
}

fn set_up_threads(players: i32) {
    let barrier = Arc::new(Barrier::new((players + 1) as usize));
    let (tx, rx) = mpsc::channel();

    let mut threads = Vec::new();

    for _ in 0..players {
        let tx_clone = mpsc::Sender::clone(&tx);
        let barrier_clone = barrier.clone();
        threads.push(thread::spawn(move || {
            player(tx_clone, barrier_clone)
        }));
    }


    threads.push(thread::spawn(move || {
        coordinator(players,rx, barrier)
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

extern crate clap;

mod player;
mod coordinator;
mod signed_card;
mod cards;
mod logger;

use clap::{App, Arg};
use std::sync::{mpsc, Barrier, Arc, Mutex, Condvar};
use std::thread;
use player::player;
use coordinator::coordinator;
use std::io::{Error, ErrorKind};
use crate::logger::{create_logfile, debug, info, error, LogFile};
use crate::player::RoundPlayerFlags;
use std::collections::HashMap;


fn check_player_quantity(players: i32) -> bool {
    return players >= 4 && players % 2 == 0
}

fn set_up_threads(players: i32, log_file: LogFile) -> std::io::Result<()> {
    let barrier = Arc::new(Barrier::new((players + 1) as usize));

    let (tx_card, rx_card) = mpsc::channel();
    let (tx_deck, rx_deck) = mpsc::channel();
    let shared_rx_deck = Arc::new(Mutex::new(rx_deck));

    let mut threads = Vec::new();
    let mut cond_vars_players = HashMap::new();

    info(log_file.clone(), "Esperando jugadores".to_string())?;
    for p in 0..players {
        let tx_clone_player = mpsc::Sender::clone(&tx_card);
        let barrier_clone = barrier.clone();
        let shared_rx_deck_clone = shared_rx_deck.clone();
        let log_file_clone = log_file.clone();
        let cond_var_pair = Arc::new((Mutex::new(RoundPlayerFlags{is_my_turn: false, can_throw_card: false,
                                                                    game_ended: false}), Condvar::new()));
        let cond_var_pair_clone = cond_var_pair.clone();
        threads.push(thread::spawn(move || {
            player(log_file_clone, tx_clone_player, barrier_clone,
                   shared_rx_deck_clone, cond_var_pair, p).unwrap();
        }));
        cond_vars_players.insert(p, cond_var_pair_clone);
    }

    info(log_file.clone(), "Iniciando coordinador".to_string())?;
    threads.push(thread::spawn(move || {
        coordinator(log_file, players, rx_card, barrier,
                    tx_deck, cond_vars_players).unwrap();
    }));

    for thread in threads {
        thread.join().unwrap();
    }

    return Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Rustico simulation")
        .version("1.0")
        .arg(Arg::with_name("players")
            .short("p")
            .long("players")
            .help("Number of players to participate in the game.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Debug file")
            .takes_value(true)
            .required(false))
        .get_matches();

    let players: i32 = matches.value_of("players").unwrap().trim().parse().unwrap();
    let mut logfile: LogFile = Arc::new(Mutex::new(None));
    if matches.is_present("debug") {
        logfile = create_logfile(matches.value_of("debug").unwrap().to_string()).unwrap();
        debug(logfile.clone(), "Inicio del logfile".to_string())?;
    }


    if !check_player_quantity(players){
        error(logfile.clone(), "ERROR: Number of players should be greater or equal than four and divisible by two.".to_string())?;
        return Err(Error::new(ErrorKind::Other,
                            "Number of players should be greater or equal than four and divisible by two."));
    }

    set_up_threads(players, logfile)?;

    return Ok(());
}

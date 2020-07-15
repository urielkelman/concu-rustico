use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Barrier, Arc, Mutex, Condvar};
use std::collections::HashMap;

use crate::signed_card::SignedCard;
use crate::cards::{Card, random_full_deck};
use crate::player::RoundPlayerFlags;

use crate::logger::{LogFile, info, debug, error};

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

fn player_fixed_values_map(players: i32, value: i32) -> HashMap<i32, i32>{
    let mut points_by_user = HashMap::new();

    for p in 0..players {
        points_by_user.insert(p, value);
    }

    return points_by_user
}

fn merge_points_hashmaps(map1: HashMap<i32, i32>, map2: HashMap<i32, i32>) -> HashMap<i32, i32>{
    let mut merged = HashMap::new();
    for key in map1.keys() {
        if map2.contains_key(key){
            merged.insert(*key, *map1.get(key).unwrap()+*map2.get(key).unwrap());
        } else {
            merged.insert(*key, *map1.get(key).unwrap());
        }
    }
    return merged;
}

fn calculate_normal_hand_points(logfile: LogFile, mut signed_cards: Vec<SignedCard>) -> HashMap<i32, i32>{
    let mut points_by_user = player_fixed_values_map(signed_cards.len() as i32, 0);

    signed_cards.sort_by(|a, b| a.card.number.cmp(&b.card.number));

    let max_card: Card = signed_cards.last().unwrap().card;
    let mut i = signed_cards.len() - 1;
    while i >= 0 && signed_cards[i].card.number == max_card.number {
        points_by_user.insert(signed_cards[i].player_signature, POINTS_MAX_CARD);
        debug(logfile.clone(), format!("El jugador con id {} gana {} puntos por tirar la máxima carta de la ronda.",
                signed_cards[i].player_signature, POINTS_MAX_CARD));
        i-=1;
    }

    return points_by_user;
}

fn register_current_points(logfile: LogFile, points_by_user: &HashMap<i32, i32>){
    for (user, points ) in points_by_user {
        debug(logfile.clone(), format!("El jugador con id {} posee {} puntos.", user, points));
    }
}


fn calculate_rustic_hand_points(logfile: LogFile, signed_cards: Vec<SignedCard>) -> (HashMap<i32, i32>, i32) {
    let mut rustic_hand_points = player_fixed_values_map(signed_cards.len() as i32, 0);

    let first_player = signed_cards.first().unwrap();
    rustic_hand_points.insert(first_player.player_signature, POINTS_FASTER_PLAYER);
    debug(logfile.clone(),format!("Ronda rústica: el jugador con id {} ha sido el mas rapido, \
    sumando {} puntos", first_player.player_signature, POINTS_FASTER_PLAYER));

    let last_player = *signed_cards.last().unwrap();
    rustic_hand_points.insert(last_player.player_signature, POINTS_SLOWER_PLAYER);

    debug(logfile.clone(),format!("Ronda rústica: el jugador con id {} ha sido el mas lento, \
    restando {} puntos y perdiendo su proximo turno", last_player.player_signature, POINTS_SLOWER_PLAYER));

    let normal_hand_points = calculate_normal_hand_points(logfile.clone(), signed_cards);

    return (merge_points_hashmaps(normal_hand_points, rustic_hand_points), last_player.player_signature) ;
}

fn keep_playing(available_cards_by_user: &HashMap<i32,i32>) -> bool{
    for available_cards in available_cards_by_user.values(){
        if *available_cards == 0 {
            return false;
        }
    }
    return true;
}

fn determine_round_points(cards: Vec<SignedCard>, normal: bool, logfile: LogFile) -> (HashMap<i32,i32>, Option<i32>){
    return if normal {
        (calculate_normal_hand_points(logfile, cards), None)
    } else {
        let (points_by_user, suspended_player) = calculate_rustic_hand_points(logfile, cards);
        (points_by_user, Some(suspended_player))
    }
}

pub fn coordinator(logfile: LogFile, players: i32, card_receiver: Receiver<SignedCard>,
                   starting_barrier: Arc<Barrier>, tx_deck :Sender<Vec<Card>>,
                   cond_vars_players: HashMap<i32, Arc<(Mutex<RoundPlayerFlags>, Condvar)>>) {
    let (deck_size, unused_cards) = deal_cards_to_players(players, tx_deck);
    debug(logfile.clone(), format!("Cartas sin usar {}", unused_cards));

    let mut points_by_user = player_fixed_values_map(players, 0);
    let mut available_cards_by_user = player_fixed_values_map(players, deck_size);

    let mut round = 1;

    let mut suspended_player: Option<i32> = None;

    while keep_playing(&available_cards_by_user){
        debug(logfile.clone(), format!("Iniciando ronda {}", round));

        let mut cards = Vec::new();

        let normal = round % 2 == 0;

        if normal {
            debug(logfile.clone(), "La ronda es de tipo normal".to_string());
        } else {
            debug(logfile.clone(), "La ronda es de tipo rústica".to_string());
        }

        if normal {
            starting_barrier.wait();
        }

        for p in 0..players {
            /// Scope artificial creado para poder liberar el lock que se adquiere al obtener la variable
            /// can_play. Si no generamos este scope, el player nunca puede adquirir el lock y hay un deadlock
            /// cuando intenta adquirir el valor de la condition variable.
            {
                let cond_var = cond_vars_players.get(&p).unwrap();
                let (lock, cvar) = &**cond_var;
                let mut round_player_flags = lock.lock().unwrap();
                *round_player_flags = RoundPlayerFlags{is_my_turn: true, can_throw_card: true};
                if suspended_player.is_some() && suspended_player.unwrap() == p {
                    (*round_player_flags).can_throw_card = false;
                }
                cvar.notify_one();
            }

            if suspended_player.is_some() && suspended_player.unwrap() == p {
                continue;
            }

            if normal {
                let signed_card = card_receiver.recv().unwrap();
                debug(logfile.clone(), format!("El jugador {} tiró una carta de número {}",
                                                signed_card.player_signature,
                                                signed_card.card.number));
                cards.push(signed_card);
            }
        }

        if !normal{
            starting_barrier.wait();
            for p in 0..players{
                if suspended_player.is_some() && suspended_player.unwrap() == p {
                    continue;
                }
                let signed_card = card_receiver.recv().unwrap();
                debug(logfile.clone(), format!("El jugador {} tiró una carta de número {}",
                                                signed_card.player_signature,
                                                signed_card.card.number));
                cards.push(signed_card);
            }
        }

        let (hand_points, new_suspended_player) = determine_round_points(cards, normal, logfile.clone());

        points_by_user = merge_points_hashmaps(points_by_user, hand_points);

        register_current_points(logfile.clone(), &points_by_user);

        for p in 0..players {
            if suspended_player.is_some() && suspended_player.unwrap() == p {
                continue;
            }
            let current_cards = available_cards_by_user.get(&p).unwrap();
            available_cards_by_user.insert(p, current_cards - 1);
        }

        suspended_player = new_suspended_player;
        debug(logfile.clone(), format!("Terminando ronda {}.", round));
        round += 1;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_empty_points_map_len() {
        let empty_map = player_fixed_values_map(4, 0);
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
        assert_eq!(merged_map.len(), 3);
        assert_eq!(*merged_map.get(&(1 as i32)).unwrap(), 8 as i32);
        assert_eq!(*merged_map.get(&(2 as i32)).unwrap(), 2 as i32);
        assert_eq!(*merged_map.get(&(5 as i32)).unwrap(), 7 as i32);
    }
}

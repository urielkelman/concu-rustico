use std::sync::mpsc::Receiver;

pub fn coordinator(card_receiver: Receiver<i32>) {
    println!("corrd");
}
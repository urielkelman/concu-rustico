use crate::cards::Card;

#[derive(Copy, Clone)]
pub struct SignedCard {
    pub card: Card,
    pub player_signature: i32
}

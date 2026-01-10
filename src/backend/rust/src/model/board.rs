use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum BoardState {
    Flag = -3,
    Mine = -2,
    Unknown = -1,
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl BoardState {
    pub fn increment(&self) -> BoardState {
        match self {
            BoardState::Zero => BoardState::One,
            BoardState::One => BoardState::Two,
            BoardState::Two => BoardState::Three,
            BoardState::Three => BoardState::Four,
            BoardState::Four => BoardState::Five,
            BoardState::Five => BoardState::Six,
            BoardState::Six => BoardState::Seven,
            BoardState::Seven => BoardState::Eight,
            _ => *self,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

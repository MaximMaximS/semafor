use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Bakalari,
    Static,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub enum Light {
    Off,
    Red,
    Amber,
    RedAmber,
    Green,
    RedGreen,
    AmberGreen,
    #[default]
    RedAmberGreen,
}

impl Light {
    pub const fn to_val(self) -> u8 {
        match self {
            Self::Off => 0b000,
            Self::Red => 0b001,
            Self::Amber => 0b010,
            Self::RedAmber => 0b011,
            Self::Green => 0b100,
            Self::RedGreen => 0b101,
            Self::AmberGreen => 0b110,
            Self::RedAmberGreen => 0b111,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct State {
    pub mode: Mode,
    pub custom: Light,
}

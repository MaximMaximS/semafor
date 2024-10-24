use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Bakalari,
    Static,
    Random,
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

impl Distribution<Light> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Light {
        let val = rng.gen_range(0..=7);
        Light::from_val(val)
    }
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

    pub const fn from_val(val: u8) -> Self {
        match val {
            0b001 => Self::Red,
            0b010 => Self::Amber,
            0b011 => Self::RedAmber,
            0b100 => Self::Green,
            0b101 => Self::RedGreen,
            0b110 => Self::AmberGreen,
            0b111 => Self::RedAmberGreen,
            _ => Self::Off,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct State {
    pub mode: Mode,
    pub custom: Light,
}

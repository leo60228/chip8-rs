use enum_map::EnumMap;
use derive_more::{From, Into, Add, AddAssign, Sub, SubAssign};
use enumn::N;

pub(crate) type Bits<'a> = (&'a [u8], usize);

#[derive(Debug, Default, From, Into, Copy, Clone, Add, AddAssign, Sub, SubAssign, PartialEq, Eq)]
pub struct Address(pub u16);

pub const BUTTON_KEYS: [minifb::Key; 16] = {
    use minifb::Key::*;

    [
        Key1, Key2, Key3, Key4,
        Q, W, E, R,
        A, S, D, F,
        Z, X, C, V,
    ]
};

#[repr(u8)]
#[derive(enum_map::Enum, Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, N)]
pub enum Button {
    B0 = 0x0,
    B1 = 0x1,
    B2 = 0x2,
    B3 = 0x3,
    B4 = 0x4,
    B5 = 0x5,
    B6 = 0x6,
    B7 = 0x7,
    B8 = 0x8,
    B9 = 0x9,
    BA = 0xA,
    BB = 0xB,
    BC = 0xC,
    BD = 0xD,
    BE = 0xE,
    BF = 0xF,
}

impl Button {
    pub fn from_key(key: minifb::Key) -> Option<Self> {
        use minifb::Key::*;
        use Button::*;

        match key {
            Key1 => Some(B1),
            Key2 => Some(B2),
            Key3 => Some(B3),
            Key4 => Some(BC),
            Q => Some(B4),
            W => Some(B5),
            E => Some(B6),
            R => Some(BD),
            A => Some(B7),
            S => Some(B8),
            D => Some(B9),
            F => Some(BE),
            Z => Some(BA),
            X => Some(B0),
            C => Some(BB),
            V => Some(BF),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(enum_map::Enum, Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xA,
    VB = 0xB,
    VC = 0xC,
    VD = 0xD,
    VE = 0xE,
    VF = 0xF,
}

pub use crate::eval::Instruction;

pub struct State {
    pub memory: [u8; 4096],
    pub registers: EnumMap<Register, u8>,
    pub i_reg: Address,
    pub pc: Address,
    pub call_stack: Vec<Address>,
    pub timer: u8,
    pub sound_timer: u8,
    pub bit_gfx: [u8; 256],
    pub pix_gfx: [u32; 2048],
    pub buttons: EnumMap<Button, bool>,
}

impl Default for State {
    fn default() -> Self {
        State {
            memory: [0u8; 4096],
            registers: Default::default(),
            i_reg: Default::default(),
            pc: 0x200.into(),
            call_stack: Default::default(),
            timer: Default::default(),
            sound_timer: Default::default(),
            bit_gfx: [0u8; 256],
            pix_gfx: [0u32; 2048],
            buttons: Default::default(),
        }
    }
}
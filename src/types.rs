use enum_map::EnumMap;
use derive_more::{From, Into, Add, AddAssign, Sub, SubAssign};

pub(crate) type Bits<'a> = (&'a [u8], usize);

#[derive(Debug, Default, From, Into, Copy, Clone, Add, AddAssign, Sub, SubAssign, PartialEq, Eq)]
pub struct Address(pub u16);

#[derive(enum_map::Enum, Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
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
        }
    }
}
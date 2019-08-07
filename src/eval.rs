use crate::types::*;
use bitvec::prelude::Bits;
use bitvec::prelude::*;

fn bcd(n: u8) -> [u8; 3] {
    fn bcd_inner(i: u8, n: u8, xs: &mut [u8; 3]) {
        if n >= 10 {
            bcd_inner(i + 1, n / 10, xs);
        }

        xs[i as usize] = n % 10;
    }

    let mut xs = [0u8; 3];
    bcd_inner(0, n, &mut xs);
    xs
}

#[derive(Debug)]
pub enum Instruction {
    RcaCall(Address),              // 0NNN
    ClearDisplay,                  // 00E0
    Return,                        // 00EE
    Goto(Address),                 // 1NNN
    Call(Address),                 // 2NNN
    SkipEqImm(Register, u8),       // 3XNN
    SkipNeqImm(Register, u8),      // 4XNN
    SkipEqReg(Register, Register), // 5XY0
    SetImm(Register, u8),          // 6XNN
    AddImm(Register, u8),          // 7XNN
    SetReg(Register, Register),    // 8XY0
    OrReg(Register, Register),     // 8XY1
    AndReg(Register, Register),    // 8XY2
    XorReg(Register, Register),    // 8XY3
    AddReg(Register, Register),    // 8XY4
    SubReg(Register, Register),    // 8XY5
    RShiftReg(Register, Register), // 8XY6
    /// .0 = .1 - .0
    RevSubReg(Register, Register), // 8XY7
    LShiftReg(Register, Register), // 8XYE
    SkipNeqReg(Register, Register), // 9XY0
    SetAddr(Address),              // ANNN
    IndexedJump(Address),          // BNNN
    Rand(Register, u8),            // CXNN
    Draw(Register, Register, u8),  // DXYN
    SkipPressed(Register),         // EX9E
    SkipUnpressed(Register),       // EXA1
    GetTimer(Register),            // FX07
    WaitPress(Register),           // FX0A
    SetTimer(Register),            // FX15
    SetSoundTimer(Register),       // FX18
    AddAddr(Register),             // FX1E
    SpriteAddr(Register),          // FX29
    BCD(Register),                 // FX33
    RegDump(Register),             // FX55
    RegLoad(Register),             // FX65
}

impl Instruction {
    pub fn eval(&self, state: &mut State) {
        use Instruction::*;

        state.pc += 2.into();

        match self {
            SetImm(reg, n) => state.registers[*reg] = *n,
            SetAddr(addr) => state.i_reg = *addr,
            Draw(x, y, h) => {
                let x = usize::from(state.registers[*x]);
                let y = usize::from(state.registers[*y]);
                let h: usize = (*h).into();

                let sprite_begin: usize = u16::from(state.i_reg).into();
                let sprite_end = sprite_begin + h;
                let sprite = &state.memory[sprite_begin..sprite_end];

                let gfx_bits = (&mut state.bit_gfx[..]).as_mut_bitslice::<BigEndian>();

                let mut collision = false;

                for (yi, row) in sprite.into_iter().enumerate() {
                    for (xi, bit) in row.as_bitslice::<BigEndian>().into_iter().enumerate() {
                        if bit {
                            let idx = ((x + xi) % 64) + ((y + yi) % 32) * 64;
                            let old = gfx_bits.get(idx).unwrap();

                            if old {
                                collision = true;
                                gfx_bits.set(idx, false);
                            } else {
                                gfx_bits.set(idx, true);
                            }
                        }
                    }
                }

                state.registers[Register::VF] = if collision { 1 } else { 0 };
            }
            Call(addr) => {
                state.call_stack.push(state.pc);
                state.pc = *addr;
            }
            BCD(reg) => {
                let bcd = bcd(state.registers[*reg]);
                let hundreds = bcd[2];
                let tens = bcd[1];
                let ones = bcd[0];
                state.memory[state.i_reg.0 as usize + 0] = hundreds;
                state.memory[state.i_reg.0 as usize + 1] = tens;
                state.memory[state.i_reg.0 as usize + 2] = ones;
            }
            RegLoad(reg) => {
                for (i, (_, reg)) in state
                    .registers
                    .iter_mut()
                    .filter(|r| r.0 <= *reg)
                    .enumerate()
                {
                    *reg = state.memory[state.i_reg.0 as usize + i];
                }
            }
            RegDump(reg) => {
                for (i, (_, reg)) in state
                    .registers
                    .iter_mut()
                    .filter(|r| r.0 <= *reg)
                    .enumerate()
                {
                    state.memory[state.i_reg.0 as usize + i] = *reg;
                }
            }
            SpriteAddr(reg) => {
                assert!(
                    state.registers[*reg] <= 0xF,
                    "{:X} not in font",
                    state.registers[*reg]
                );
                state.i_reg = (5u16 * (state.registers[*reg] as u16)).into();
            }
            AddImm(reg, n) => {
                let (val, carry) = state.registers[*reg].overflowing_add(*n);
                state.registers[*reg] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            }
            Return => state.pc = state.call_stack.pop().expect("Returned with empty stack!"),
            SetTimer(reg) => state.timer = state.registers[*reg],
            SetSoundTimer(reg) => state.sound_timer = state.registers[*reg],
            GetTimer(reg) => state.registers[*reg] = state.timer,
            SkipEqImm(reg, n) => {
                if state.registers[*reg] == *n {
                    state.pc += 2.into();
                }
            }
            SkipEqReg(r1, r2) => {
                if state.registers[*r1] == state.registers[*r2] {
                    state.pc += 2.into();
                }
            }
            SkipNeqImm(reg, n) => {
                if state.registers[*reg] != *n {
                    state.pc += 2.into();
                }
            }
            SkipNeqReg(r1, r2) => {
                if state.registers[*r1] != state.registers[*r2] {
                    state.pc += 2.into();
                }
            }
            Goto(addr) => state.pc = *addr,
            Rand(reg, mask) => state.registers[*reg] = rand::random::<u8>() & mask,
            SkipUnpressed(reg) => {
                let button = Button::n(state.registers[*reg]).unwrap();

                if !state.buttons[button] {
                    state.pc += 2.into()
                }
            }
            SkipPressed(reg) => {
                let button = Button::n(state.registers[*reg]).unwrap();

                if state.buttons[button] {
                    state.pc += 2.into()
                }
            }
            AndReg(r1, r2) => state.registers[*r1] &= state.registers[*r2],
            OrReg(r1, r2) => state.registers[*r1] |= state.registers[*r2],
            XorReg(r1, r2) => state.registers[*r1] ^= state.registers[*r2],
            LShiftReg(r1, r2) => {
                let msb = state.registers[*r1] >> 7;
                state.registers[*r2] = state.registers[*r1] << 1;
                state.registers[Register::VF] = msb;
            }
            RShiftReg(r1, r2) => {
                let lsb = state.registers[*r1] & 1;
                state.registers[*r2] = state.registers[*r1] >> 1;
                state.registers[Register::VF] = lsb;
            }
            SetReg(r1, r2) => state.registers[*r1] = state.registers[*r2],
            AddReg(r1, r2) => {
                let (val, carry) = state.registers[*r1].overflowing_add(state.registers[*r2]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            }
            SubReg(r1, r2) => {
                let (val, carry) = state.registers[*r1].overflowing_sub(state.registers[*r2]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if !carry { 1 } else { 0 };
            }
            RevSubReg(r1, r2) => {
                let (val, carry) = state.registers[*r2].overflowing_sub(state.registers[*r1]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if !carry { 1 } else { 0 };
            }
            IndexedJump(offset) => {
                state.pc = Address::from(state.registers[Register::V0] as u16) + *offset
            }
            WaitPress(reg) => {
                for (button, pressed) in &state.buttons {
                    if *pressed {
                        state.registers[*reg] = button as u8;
                        return;
                    }
                }

                state.pc -= 2.into();
            }
            AddAddr(reg) => state.i_reg += (state.registers[*reg] as u16).into(),
            ClearDisplay => state.bit_gfx = [0u8; 256],
            RcaCall(_) => panic!("RCA calls not supported!"),
        }
    }
}

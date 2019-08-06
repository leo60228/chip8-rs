use crate::types::*;

#[derive(Debug)]
pub enum Instruction {
    RcaCall(Address), // 0NNN
    ClearDisplay, // 00E0
    Return, // 00EE
    Goto(Address), // 1NNN
    Call(Address), // 2NNN
    SkipEqImm(Register, u8), // 3XNN
    SkipNeqImm(Register, u8), // 4XNN
    SkipEqReg(Register, Register), // 5XY0
    SetImm(Register, u8), // 6XNN
    AddImm(Register, u8), // 7XNN
    SetReg(Register, Register), // 8XY0
    OrReg(Register, Register), // 8XY1
    AndReg(Register, Register), // 8XY2
    XorReg(Register, Register), // 8XY3
    AddReg(Register, Register), // 8XY4
    SubReg(Register, Register), // 8XY5
    RShiftReg(Register, Register), // 8XY6
    /// .0 = .1 - .0
    RevSubReg(Register, Register), // 8XY7
    LShiftReg(Register, Register), // 8XYE
    SkipNeqReg(Register, Register), // 9XY0
    SetAddr(Address), // ANNN
    IndexedJump(Address), // BNNN
    Rand(Register, u8), // CXNN
    Draw(Register, Register, u8), // DXYN
    SkipPressed(Register), // EX9E
    SkipUnpressed(Register), // EXA1
    GetTimer(Register), // FX07
    WaitPress(Register), // FX0A
    SetTimer(Register), // FX15
    SetSoundTimer(Register), // FX18
    AddAddr(Register), // FX1E
    SpriteAddr(Register), // FX29
    BCD(Register), // FX33
    RegDump(Register), // FX55
    RegLoad(Register), // FX65
}

impl Instruction {
    pub fn eval(&self, state: &mut State) {
        use Instruction::*;

        state.pc += 2.into();

        match self {
            SetImm(reg, n) => state.registers[*reg] = *n,
            SetAddr(addr) => state.i_reg = *addr,
            Draw(_x, _y, _h) => {}, // TODO: graphics
            Call(addr) => {
                state.call_stack.push(state.pc);
                state.pc = *addr;
            },
            BCD(reg) => {
                let bcd = bcd::Bcd::<u32>(state.registers[*reg].into());
                let hundreds = bcd.digit(2);
                let tens = bcd.digit(1);
                let ones = bcd.digit(0);
                state.memory[state.i_reg.0 as usize + 0] = hundreds;
                state.memory[state.i_reg.0 as usize + 1] = tens;
                state.memory[state.i_reg.0 as usize + 2] = ones;
            },
            RegLoad(reg) => {
                for (i, (_, reg)) in state.registers.iter_mut().filter(|r| r.0 <= *reg).enumerate() {
                    *reg = state.memory[state.i_reg.0 as usize + i];
                }
            },
            RegDump(reg) => {
                for (i, (_, reg)) in state.registers.iter_mut().filter(|r| r.0 <= *reg).enumerate() {
                    state.memory[state.i_reg.0 as usize + i] = *reg;
                }
            },
            SpriteAddr(reg) => {
                assert!(state.registers[*reg] <= 0xF, "{:X} not in font", state.registers[*reg]);
                state.i_reg = (5u16 * (state.registers[*reg] as u16)).into();
            }
            AddImm(reg, n) => {
                let (val, carry) = state.registers[*reg].overflowing_add(*n);
                state.registers[*reg] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            },
            Return => state.pc = state.call_stack.pop().expect("Returned with empty stack!"),
            SetTimer(reg) => state.timer = state.registers[*reg],
            SetSoundTimer(reg) => state.sound_timer = state.registers[*reg],
            GetTimer(reg) => state.registers[*reg] = state.timer,
            SkipEqImm(reg, n) => if state.registers[*reg] == *n { state.pc += 2.into(); },
            SkipEqReg(r1, r2) => if state.registers[*r1] == state.registers[*r2] {
                state.pc += 2.into();
            },
            SkipNeqImm(reg, n) => if state.registers[*reg] != *n { state.pc += 2.into(); },
            SkipNeqReg(r1, r2) => if state.registers[*r1] != state.registers[*r2] {
                state.pc += 2.into();
            },
            Goto(addr) => state.pc = *addr,
            Rand(reg, mask) => state.registers[*reg] = rand::random::<u8>() & mask,
            SkipUnpressed(_reg) => state.pc += 2.into(), // TODO: input
            SkipPressed(_reg) => {}, // TODO: input
            AndReg(r1, r2) => state.registers[*r1] &= state.registers[*r2],
            OrReg(r1, r2) => state.registers[*r1] |= state.registers[*r2],
            XorReg(r1, r2) => state.registers[*r1] ^= state.registers[*r2],
            LShiftReg(r1, r2) => {
                let lsb = state.registers[*r2] & (!1);
                state.registers[*r1] = state.registers[*r2] << 1;
                state.registers[Register::VF] = lsb;
            },
            RShiftReg(r1, r2) => {
                let msb = state.registers[*r2] << 7;
                state.registers[*r1] = state.registers[*r2] >> 1;
                state.registers[Register::VF] = msb;
            },
            SetReg(r1, r2) => state.registers[*r1] = state.registers[*r2],
            AddReg(r1, r2) => {
                let (val, carry) = state.registers[*r1].overflowing_add(state.registers[*r2]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            },
            SubReg(r1, r2) => {
                let (val, carry) = state.registers[*r1].overflowing_sub(state.registers[*r2]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            },
            RevSubReg(r1, r2) => {
                let (val, carry) = state.registers[*r2].overflowing_sub(state.registers[*r1]);
                state.registers[*r1] = val;
                state.registers[Register::VF] = if carry { 1 } else { 0 };
            },
            IndexedJump(offset) => state.pc = Address::from(state.registers[Register::V0] as u16) + *offset,
            WaitPress(_reg) => state.pc -= 2.into(), // TODO: input
            AddAddr(reg) => state.i_reg += (state.registers[*reg] as u16).into(),
            ClearDisplay => {}, // TODO: graphics
            RcaCall(_) => panic!("RCA calls not supported!"),
        }
    }
}
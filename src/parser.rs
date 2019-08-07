use crate::types::*;
use nom::bits::bits;
use nom::bits::complete as bits;
use nom::branch::alt;
use nom::combinator::map;
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

pub fn addr_bits(input: Bits) -> IResult<Bits, Address> {
    map(bits::take(12usize), |n: u16| Address(n))(input)
}

pub fn register_bits(input: Bits) -> IResult<Bits, Register> {
    map(bits::take(4usize), |n: u8| match n {
        0x0 => Register::V0,
        0x1 => Register::V1,
        0x2 => Register::V2,
        0x3 => Register::V3,
        0x4 => Register::V4,
        0x5 => Register::V5,
        0x6 => Register::V6,
        0x7 => Register::V7,
        0x8 => Register::V8,
        0x9 => Register::V9,
        0xA => Register::VA,
        0xB => Register::VB,
        0xC => Register::VC,
        0xD => Register::VD,
        0xE => Register::VE,
        0xF => Register::VF,
        _ => unreachable!(),
    })(input)
}

pub fn instr_bits(input: Bits) -> IResult<Bits, Instruction> {
    use Instruction::*;

    alt((
        map(bits::tag(0x00E0, 16usize), |_| ClearDisplay),
        map(bits::tag(0x00EE, 16usize), |_| Return),
        map(preceded(bits::tag(0x0, 4usize), addr_bits), |addr| {
            RcaCall(addr)
        }),
        map(preceded(bits::tag(0x1, 4usize), addr_bits), |addr| {
            Goto(addr)
        }),
        map(preceded(bits::tag(0x2, 4usize), addr_bits), |addr| {
            Call(addr)
        }),
        map(
            preceded(
                bits::tag(0x3, 4usize),
                tuple((register_bits, bits::take(8usize))),
            ),
            |(reg, n): (Register, u8)| SkipEqImm(reg, n),
        ),
        map(
            preceded(
                bits::tag(0x4, 4usize),
                tuple((register_bits, bits::take(8usize))),
            ),
            |(reg, n): (Register, u8)| SkipNeqImm(reg, n),
        ),
        map(
            preceded(
                bits::tag(0x5, 4usize),
                tuple((register_bits, register_bits, bits::take(4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| SkipEqReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x6, 4usize),
                tuple((register_bits, bits::take(8usize))),
            ),
            |(reg, n): (Register, u8)| SetImm(reg, n),
        ),
        map(
            preceded(
                bits::tag(0x7, 4usize),
                tuple((register_bits, bits::take(8usize))),
            ),
            |(reg, n): (Register, u8)| AddImm(reg, n),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x0, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| SetReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x1, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| OrReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x2, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| AndReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x3, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| XorReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x4, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| AddReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x5, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| SubReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x6, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| RShiftReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x7, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| RevSubReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x8, 4usize),
                tuple((register_bits, register_bits, bits::tag(0xE, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| LShiftReg(r1, r2),
        ),
        map(
            preceded(
                bits::tag(0x9, 4usize),
                tuple((register_bits, register_bits, bits::tag(0x0, 4usize))),
            ),
            |(r1, r2, _): (_, _, u8)| SkipNeqReg(r1, r2),
        ),
        alt((
            map(preceded(bits::tag(0xA, 4usize), addr_bits), |addr| {
                SetAddr(addr)
            }),
            map(preceded(bits::tag(0xB, 4usize), addr_bits), |addr| {
                IndexedJump(addr)
            }),
            map(
                preceded(
                    bits::tag(0xC, 4usize),
                    tuple((register_bits, bits::take(8usize))),
                ),
                |(reg, n): (Register, u8)| Rand(reg, n),
            ),
            map(
                preceded(
                    bits::tag(0xD, 4usize),
                    tuple((register_bits, register_bits, bits::take(4usize))),
                ),
                |(r1, r2, h): (_, _, u8)| Draw(r1, r2, h),
            ),
            map(
                preceded(
                    bits::tag(0xE, 4usize),
                    terminated(register_bits, bits::tag(0x9E, 8usize)),
                ),
                |reg| SkipPressed(reg),
            ),
            map(
                preceded(
                    bits::tag(0xE, 4usize),
                    terminated(register_bits, bits::tag(0xA1, 8usize)),
                ),
                |reg| SkipUnpressed(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x07, 8usize)),
                ),
                |reg| GetTimer(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x0A, 8usize)),
                ),
                |reg| WaitPress(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x15, 8usize)),
                ),
                |reg| SetTimer(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x18, 8usize)),
                ),
                |reg| SetSoundTimer(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x1E, 8usize)),
                ),
                |reg| AddAddr(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x29, 8usize)),
                ),
                |reg| SpriteAddr(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x33, 8usize)),
                ),
                |reg| BCD(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x55, 8usize)),
                ),
                |reg| RegDump(reg),
            ),
            map(
                preceded(
                    bits::tag(0xF, 4usize),
                    terminated(register_bits, bits::tag(0x65, 8usize)),
                ),
                |reg| RegLoad(reg),
            ),
        )),
    ))(input)
}

pub fn instr(input: &[u8]) -> IResult<&[u8], Instruction> {
    bits(instr_bits)(input)
}

use std::fs::File;
use std::env;
use std::usize;
use std::io::prelude::*;
use std::time::{Instant, Duration};

fn main() {
    let mut state = chip8::types::State::default();

    for (i, path) in env::args().skip(1).enumerate() {
        let mut file = File::open(path).expect("Couldn't open!");

        let mut write_at = if i == 0 { 0x200 } else { 0x0 };

        while let read @ 1..=usize::MAX = file.read(&mut state.memory[write_at..]).expect("Couldn't read!") {
            write_at += read;
        }
    }

    let mut time = Instant::now();

    loop {
        let instr_start: u16 = state.pc.into();
        let instr_end: u16 = instr_start + 1;

        let current_instr = chip8::parser::instr(&state.memory[(instr_start as usize)..=(instr_end as usize)])
            .expect("invalid instruction at pc").1;
        println!("{:X?}", current_instr);

        current_instr.eval(&mut state);

        let now = Instant::now();
        if now - time > Duration::from_millis(1000 / 60) {
            time = now;
            if state.timer > 0 { state.timer -= 1; }
            println!("timer: {}", state.timer);
        }
    }
}
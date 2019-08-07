use std::fs::File;
use std::env;
use std::usize;
use std::io::prelude::*;
use std::time::{Instant, Duration};
use rodio::Sink;
use rodio::source::SineWave;
use minifb::Window;
use minifb::WindowOptions;
use bitvec::prelude::*;

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

    let device = rodio::default_output_device();

    let sink = device.as_ref().and_then(|device| {
        if device.supported_output_formats().ok().map(Iterator::count).unwrap_or(0) <= 0 {
            None
        } else {
            let sink = Sink::new(device);
            sink.set_volume(0.5);
            sink.pause();

            let sine = SineWave::new(440);
            sink.append(sine);

            Some(sink)
        }
    });

    if device.is_none() {
        eprintln!("Couldn't initialize audio!");
    }

    let mut window = Window::new("chip8-rs", 64, 32, WindowOptions {
        scale: minifb::Scale::X16,
        ..Default::default()
    }).expect("Couldn't initialize window!");

    loop {
        let instr_start: u16 = state.pc.into();
        let instr_end: u16 = instr_start + 1;

        let current_instr = chip8::parser::instr(&state.memory[(instr_start as usize)..=(instr_end as usize)])
            .expect("invalid instruction at pc").1;

        current_instr.eval(&mut state);

        let now = Instant::now();
        if now - time > Duration::from_millis(1000 / 60) {
            time = now;
            if state.timer > 0 { state.timer -= 1; }
            if state.sound_timer > 0 { state.sound_timer -= 1; }
        }

        if let Some(sink) = &sink {
            if state.sound_timer > 0 {
                sink.play();
            } else {
                sink.pause();
            }
        }

        for (i, e) in (&state.bit_gfx[..]).as_bitslice::<BigEndian>().iter().enumerate() {
            if e {
                state.pix_gfx[i] = 0xFFFFFFFF;
            } else {
                state.pix_gfx[i] = 0;
            }
        }

        window.update_with_buffer(&state.pix_gfx[..]).expect("Couldn't update window!");

        for button in &chip8::types::BUTTON_KEYS {
            if window.is_key_down(*button) {
                state.buttons[chip8::types::Button::from_key(*button).unwrap()] = true;
            } else {
                state.buttons[chip8::types::Button::from_key(*button).unwrap()] = false;
            }
        }
    }
}

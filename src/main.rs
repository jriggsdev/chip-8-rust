extern crate rand as rand_crate;

mod ui;

use std::collections::HashMap;
use macroquad::prelude::*;
use chip_8_emulator::{Chip8, EmulatorType, Chip8Key, KeyState};
use ui::audio::AudioPlayer;

/// The frequency of the tone to play for the Chip-8 sound
const TONE_FREQUENCY: f32 = 440.0;

/// The duration of the tone to play for the Chip-8 sound
const TONE_DURATION: f32 = 5.0;

/// The amplitude of the tone to play for the Chip-8 sound
const TONE_AMPLITUDE: f32 = 0.5;

#[macroquad::main("Chip-8 Emulator")]
async fn main() {
    let key_code_map : HashMap<KeyCode, Chip8Key> = HashMap::from([
        (KeyCode::Key1, Chip8Key::One),
        (KeyCode::Key2, Chip8Key::Two),
        (KeyCode::Key3, Chip8Key::Three),
        (KeyCode::Key4, Chip8Key::C),
        (KeyCode::Q, Chip8Key::Four),
        (KeyCode::W, Chip8Key::Five),
        (KeyCode::E, Chip8Key::Six),
        (KeyCode::R, Chip8Key::D),
        (KeyCode::A, Chip8Key::Seven),
        (KeyCode::S, Chip8Key::Eight),
        (KeyCode::D, Chip8Key::Nine),
        (KeyCode::F, Chip8Key::E),
        (KeyCode::Z, Chip8Key::A),
        (KeyCode::X, Chip8Key::Zero),
        (KeyCode::C, Chip8Key::B),
        (KeyCode::V, Chip8Key::F),
    ]);

    let mut audio_player = AudioPlayer::build(TONE_FREQUENCY, TONE_DURATION, TONE_AMPLITUDE).await;

    // TODO take Emulator type as a command line argument
    let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand_crate::rng());
    let program = include_bytes!("/home/josh/Downloads/bowling.ch8");

    chip8.load_program(program);

    let mut counter: u8 = 0;
    loop {
        chip8.execute_next_instruction();

        if counter % 12 == 0 {
            ui::renderer::render_frame(chip8.frame_buffer()).await;

            if chip8.is_playing_sound() {
                audio_player.play_tone();
            }

            if !chip8.is_playing_sound() {
                audio_player.stop_tone();
            }

            chip8.decrement_timers();
        }
        counter = counter.wrapping_add(1);

        for key in key_code_map.iter() {
            let is_key_down = is_key_down(*key.0);
            let key_state = chip8.key_state(*key.1);

            if is_key_down && key_state == KeyState::Up {
                chip8.key_down(*key.1);
            }
            if !is_key_down && key_state == KeyState::Down {
                chip8.key_up(*key.1);
            }
        }
    }
}

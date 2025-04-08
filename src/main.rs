extern crate rand as rand_crate;

use std::collections::HashMap;
use std::io::Cursor;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams};
use macroquad::prelude::*;
use chip_8_emulator::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT, EmulatorType, Chip8Key, KeyState};

const TONE_FREQUENCY: f32 = 440.0;
const TONE_DURATION: f32 = 5.0;
const TONE_AMPLITUDE: f32 = 0.5;

async fn render_frame(frame_buffer: &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
    clear_background(BLACK);

    let pixel_length = screen_width() / DISPLAY_WIDTH as f32;
    let pixel_height = screen_height() / DISPLAY_HEIGHT as f32;

    for (i, pixel) in frame_buffer.iter().enumerate() {
        let x = i % DISPLAY_WIDTH;
        let y = i / DISPLAY_WIDTH;
        let color = if *pixel == 0 { BLACK } else { GREEN };
        draw_rectangle(x as f32 * pixel_length, y as f32 * pixel_height, pixel_length, pixel_height, color);
    }

    next_frame().await;
}

/// Don't ask me how this works. I vibe coded it
fn get_sound_bytes() -> Vec<u8> {
    let spec = hound::WavSpec   {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Generate a sine wave
    let num_samples = (spec.sample_rate as f32 * TONE_DURATION) as usize;
    let mut samples = Vec::new();
    for i in 0..num_samples {
        let t = i as f32 / spec.sample_rate as f32; // Time (in seconds)
        let sample = (TONE_AMPLITUDE * (t * TONE_FREQUENCY * 2.0 * std::f32::consts::PI).sin())
            * i16::MAX as f32; // Scale sample to 16-bit range
        samples.push(sample as i16);
    }

    let mut wav_buffer = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut wav_buffer, spec).unwrap();

        for sample in samples {
            writer.write_sample(sample).unwrap();
        }

        writer.finalize().unwrap();
    }

    wav_buffer.into_inner()
}

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

    let sound_bytes = get_sound_bytes();
    let sound = load_sound_from_bytes(sound_bytes.as_slice()).await;
    let mut already_playing_sound = false;

    // TODO take Emulator type as a command line argument
    let mut chip8 = Chip8::new(EmulatorType::Chip48, rand_crate::rng());
    let program = include_bytes!("/home/josh/Downloads/test_opcode.ch8");
    chip8.load_program(program);

    let mut fb = [ 0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let mut counter: u8 = 0;
    loop {
        chip8.execute_next_instruction();
        fb.copy_from_slice(chip8.frame_buffer());

        if counter % 12 == 0 {
            render_frame(chip8.frame_buffer()).await;

            if let Ok(ref sound) = sound {
                if chip8.is_playing_sound() && !already_playing_sound {
                    play_sound(sound, PlaySoundParams { looped: false, volume: 0.2 });
                    already_playing_sound = true;
                }

                if !chip8.is_playing_sound() && already_playing_sound {
                    stop_sound(sound);
                    already_playing_sound = false;
                }
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

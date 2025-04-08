extern crate rand as rand_crate;

use std::collections::HashMap;
use macroquad::prelude::*;
use chip_8_emulator::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT, EmulatorType, Chip8Key, KeyState};

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

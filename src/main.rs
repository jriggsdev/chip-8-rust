use macroquad::prelude::*;
use chip_8_emulator::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT};

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
    let mut chip8 = Chip8::new();

    loop {
        chip8.execute_next_instruction();
        let fb = chip8.frame_buffer();
        render_frame(fb).await;
    }
}

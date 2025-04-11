use macroquad::color::{BLACK, GREEN};
use macroquad::prelude::{clear_background, draw_rectangle, next_frame, screen_height, screen_width};
use chip_8_emulator::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

// TODO make this more robust
pub async fn render_frame(frame_buffer: &[u8]) {
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

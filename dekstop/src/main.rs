use std::env;
use chip8_core::*;
use sdl2::event::Event;
use std::fs::File;
use std::io::Read;
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, keyboard::Keycode};
use std::time::{Duration, Instant};
const CYCLE_DURATION: Duration = Duration::from_nanos(1_000_000_000 / CLOCK_SPEED);



const SCREEN_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32 * SCREEN_SCALE;
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32 * SCREEN_SCALE;
const TICKS_PER_FRAME: usize = 10;


fn main() {
    // Get env arguments
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run game/path");
        return;
    }

    // Create a window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rust Chip 8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build().unwrap();
    // Canvas for drawing graphics
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut emu = Emulator::new();
    // Open ROM from arguments, read it and load into emulator
    let mut rom = File::open(&args[1]).expect("Failed to open the file");
    let mut game_buffer = Vec::new();
    rom.read_to_end(&mut game_buffer).unwrap();
    emu.load_data(&game_buffer);

    let timer_duration = Duration::from_nanos(1_000_000_000 / 60); // 60Hz
    let mut last_timer_update = Instant::now();

    let mut event_pump = sdl_context.event_pump().unwrap();
    // The entire program loop
    'programLoop: loop {
        // Poll events and match them
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'programLoop;
                },
                Event::KeyDown {keycode: Some(key), ..} => {
                    if let Some(btn) = key_to_button(key) {
                        emu.keypress(btn, true);
                    }
                },
                Event::KeyUp {keycode: Some(key), ..} => {
                    if let Some(btn) = key_to_button(key) {
                        emu.keypress(btn, false);
                    }
                },
                _ => ()
            }
        }

        let cycle_start = Instant::now();
        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }

        if cycle_start.duration_since(last_timer_update) >= timer_duration {
            emu.time_tick();
            last_timer_update = cycle_start;
        }
        // Continue emulation and draw results
        draw_display(&emu, &mut canvas);

        let elapsed = cycle_start.elapsed();
        if elapsed < CYCLE_DURATION {
            std::thread::sleep(CYCLE_DURATION - elapsed);
        }
    }
}


// Draw the entire frame from emulator to screen canvas
fn draw_display(emu: &Emulator, canvas: &mut Canvas<Window>) {
    // Clear display with black color
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Draw every white pixel on the screen
    let display = emu.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (number, pixel) in display.iter().enumerate() {
        if *pixel {
            // Convert 1D array into 2D (x, y)
            let x = (number % SCREEN_WIDTH) as u32;
            let y = (number / SCREEN_WIDTH) as u32;

            // Draw a rectangle at position (x, y) with size of (scale, scale)
            let rect = Rect::new(
                (x * SCREEN_SCALE) as i32,
                (y * SCREEN_SCALE) as i32,
                SCREEN_SCALE, SCREEN_SCALE
            );
            canvas.fill_rect(rect).unwrap()
        }
    }
    canvas.present();
}

// Convert key to Chip8 button
fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

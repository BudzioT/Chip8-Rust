use js_sys::Uint8Array;
use chip8_core::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};
use std::io::Read;
use std::fs::File;
use console_error_panic_hook::set_once;

#[wasm_bindgen(start)]
pub fn main() {
    set_once();
}

#[wasm_bindgen]
pub struct EmulatorWasm {
    emu: Emulator,
    ctx: CanvasRenderingContext2d,
}

// Wrappers and functions for frontend using wasm
#[wasm_bindgen]
impl EmulatorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<EmulatorWasm, JsValue> {
        let emu = Emulator::new();

        // Get document window, html canvas
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ()).unwrap();

        // Grab canvas context
        let ctx = canvas.get_context("2d").unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>().unwrap();

        Ok(EmulatorWasm{emu, ctx})
    }

    #[wasm_bindgen]
    pub fn draw_display(&mut self, scale: usize) {
        let display = self.emu.get_display();

        // Draw all pixels needed on canvas
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            if display[i] {
                let x = i % SCREEN_WIDTH;
                let y = i / SCREEN_WIDTH;

                self.ctx.fill_rect(
                    (x * scale) as f64, (y * scale) as f64,
                    scale as f64, scale as f64
                );
            }
        }
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, event: KeyboardEvent, pressed: bool) {
        let key = event.key();
        if let Some(k) = key_to_button(&key) {
            self.emu.keypress(k, pressed);
        }
    }

    #[wasm_bindgen]
    pub fn load_data(&mut self, data: Uint8Array) {
        self.emu.load_data(&data.to_vec());
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.emu.tick();
    }

    #[wasm_bindgen]
    pub fn time_tick(&mut self) {
        self.emu.time_tick();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.emu.reset();
    }
}

// Change key from JS to Chip8 button
fn key_to_button(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0x1),
        "2" => Some(0x2),
        "3" => Some(0x3),
        "4" => Some(0xC),
        "q" => Some(0x4),
        "w" => Some(0x5),
        "e" => Some(0x6),
        "r" => Some(0xD),
        "a" => Some(0x7),
        "s" => Some(0x8),
        "d" => Some(0x9),
        "f" => Some(0xE),
        "z" => Some(0xA),
        "x" => Some(0xB),
        "c" => Some(0),
        "v" => Some(0xF),
        _ => None,
    }
}

mod cartridge;
mod cpu;
mod emulator;
mod gpu;
mod interrupts;
mod joypad;
mod memory;
mod timer;

use emulator::Emulator;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::time::{Duration, Instant};

const SCALE: u32 = 4;
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let cart = cartridge::Cartridge::load(&args[1]).expect("Failed to load ROM");
    println!(
        "Loaded ROM: {} bytes, MBC: {:?}",
        cart.data.len(),
        cart.mbc_type
    );

    let mut emu = Emulator::new(cart);

    let sdl_context = sdl2::init().expect("Failed to init SDL2");
    let video = sdl_context.video().expect("Failed to init video");

    let window = video
        .window(
            "Game Boy Emulator",
            SCREEN_WIDTH * SCALE,
            SCREEN_HEIGHT * SCALE,
        )
        .position_centered()
        .build()
        .expect("Failed to create window");

    let mut canvas = window.into_canvas().build().expect("Failed to create canvas");
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
        .expect("Failed to create texture");

    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");

    let frame_duration = Duration::from_micros(16742); // ~59.73 Hz

    'running: loop {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(btn) = map_key(key) {
                        emu.key_pressed(btn);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(btn) = map_key(key) {
                        emu.key_released(btn);
                    }
                }

                _ => {}
            }
        }

        emu.update();

        // Blit screen data to texture
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..SCREEN_HEIGHT as usize {
                    for x in 0..SCREEN_WIDTH as usize {
                        let offset = y * pitch + x * 3;
                        buffer[offset] = emu.gpu.screen_data[x][y][0];
                        buffer[offset + 1] = emu.gpu.screen_data[x][y][1];
                        buffer[offset + 2] = emu.gpu.screen_data[x][y][2];
                    }
                }
            })
            .expect("Failed to update texture");

        canvas.clear();
        canvas.copy(&texture, None, None).expect("Failed to copy texture");
        canvas.present();

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

fn map_key(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Right => Some(0),
        Keycode::Left => Some(1),
        Keycode::Up => Some(2),
        Keycode::Down => Some(3),
        Keycode::Z => Some(4),     // A
        Keycode::X => Some(5),     // B
        Keycode::Space => Some(6), // Select
        Keycode::Return => Some(7), // Start
        _ => None,
    }
}

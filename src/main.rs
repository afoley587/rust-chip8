
use afoley_chip8::chip8::Chip8;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use std::time::Duration;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureAccess;
use bytemuck;
use clap::Parser;
use std::thread;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    rom: String,
}

fn map_sdl_keycode_to_chip8_key(keycode: sdl2::keyboard::Keycode) -> Option<usize> {
    match keycode {
        sdl2::keyboard::Keycode::Num1 => Some(0x1),
        sdl2::keyboard::Keycode::Num2 => Some(0x2),
        sdl2::keyboard::Keycode::Num3 => Some(0x3),
        sdl2::keyboard::Keycode::Num4 => Some(0xC),
        sdl2::keyboard::Keycode::Q => Some(0x4),
        sdl2::keyboard::Keycode::W => Some(0x5),
        sdl2::keyboard::Keycode::E => Some(0x6),
        sdl2::keyboard::Keycode::R => Some(0xD),
        sdl2::keyboard::Keycode::A => Some(0x7),
        sdl2::keyboard::Keycode::S => Some(0x8),
        sdl2::keyboard::Keycode::D => Some(0x9),
        sdl2::keyboard::Keycode::F => Some(0xE),
        sdl2::keyboard::Keycode::Z => Some(0xA),
        sdl2::keyboard::Keycode::X => Some(0x0),
        sdl2::keyboard::Keycode::C => Some(0xB),
        sdl2::keyboard::Keycode::V => Some(0xF),
        _ => None,
    }
}



fn main() -> Result<(), String> {
    let args = Args::parse();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Chip-8 Emulator", 640, 320)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB888, 64u32, 32u32)
        .map_err(|e| e.to_string())?;

    let mut chip8 = Chip8::load_rom(&args.rom);

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(chip8_key) = map_sdl_keycode_to_chip8_key(keycode) {
                            chip8.keyboard[chip8_key] = true;
                        }
                    }
                },
                sdl2::event::Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(chip8_key) = map_sdl_keycode_to_chip8_key(keycode) {
                            chip8.keyboard[chip8_key] = false;
                        }
                    }
                },
                _ => {}
            }
        }

        chip8.cycle();

        texture
            .update(None, bytemuck::cast_slice(&chip8.video), 64 * 4)
            .map_err(|e| e.to_string())?;

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, 640, 320)))?;
        canvas.present();
    }

    Ok(())
}


// fn main() {
//     let mut chip_8 = Chip8::load_rom("/Users/alexanderfoley/Projects/courses/fuckit-its-rust/01-new-to-rust/afoley-chip8/roms/TETRIS");
//     chip_8.cycle();
//     // println!("{:#?}", chip_8);
// }

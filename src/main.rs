extern crate minifb;

use minifb::{Key, Scale, WindowOptions, Window};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::process::exit;
use std::env;
use std::fs::File;
use std::io::Read;

mod chip8;
use chip8::Chip8;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const TIME_PER_FRAME: u64 = 1000; // not sure

fn main() {

    const Chip8Keys: [Key; 16] = [
        Key::Key1,
        Key::Key2,
        Key::Key3,
        Key::Key4,
        Key::A,
        Key::Z,
        Key::E,
        Key::R,
        Key::Q,
        Key::S,
        Key::D,
        Key::F,
        Key::W,
        Key::X,
        Key::C,
        Key::V,
    ];

    let args: Vec<String> = env::args().collect();
    println!("ARGS {:?}", args);
    if args.len() != 2 {
        println!("usage : chip ROM");
        exit(0x1);
    }

    let mut f = File::open(&args[1]).unwrap();
    let mut buffer: Vec<u8> = vec![0; 4092];
    f.read(&mut buffer);
    let mut chip8 = Chip8::init();
    chip8.load_rom(&buffer);

    let mut screen: Vec<u32>;
    let mut window = Window::new("Chip8 Emulator", WIDTH, HEIGHT, 
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
    }).unwrap();
        
    let mut timer = Duration::from_micros(0);
    while window.is_open() && !window.is_key_down(Key::Escape) {

        let now = Instant::now();

        if timer >= Duration::from_micros(16667) {
            chip8.tick_delay_timer();
            chip8.tick_sound_timer();
            timer = Duration::from_micros(0);
        }
  
        for (index, key) in Chip8Keys.iter().enumerate() {
            if window.is_key_down(*key) {
                chip8.keys[index] = 1;
            }

            if window.is_key_released(*key) {
                chip8.keys[index] = 0;
            }
        }
        chip8.emulate_cycle();
        screen = chip8.screen.iter()
            .flat_map(|array| array.iter())
            .cloned()
            .map(|pixel| { if pixel > 0 { return 0xFFFFFF; };return 0;})
            .collect();
        window.update_with_buffer(&screen).unwrap();
        sleep(Duration::from_micros(TIME_PER_FRAME));
        timer += now.elapsed();
    }



}

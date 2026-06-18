use emunes::cpu::Cpu;
use rand::RngExt;
use sdl2::{
    self, EventPump,
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
};

const RNG_ADDR: u16 = 0xfe; // random byte every step
const INPUT_ADDR: u16 = 0xff; // last key the player pressed (ASCII byte)

const SCREEN_START: u16 = 0x0200;
const SCREEN_END: u16 = 0x0600;
const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;
const SCALE: f32 = 10.0;
const FRAME_LEN: usize = (WIDTH * HEIGHT * 3) as usize; // RGB24

const GAME_PATH: &str = "games/snake.bin";

fn main() {
    let game_code =
        std::fs::read(GAME_PATH).unwrap_or_else(|e| panic!("failed to read {GAME_PATH}: {e}"));

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("Snake", WIDTH * SCALE as u32, HEIGHT * SCALE as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(SCALE, SCALE).unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH, HEIGHT)
        .unwrap();

    let mut cpu = Cpu::new();
    cpu.load(game_code);
    cpu.reset();

    let mut screen_state = [0u8; FRAME_LEN];
    let mut rng = rand::rng();

    cpu.run_with(move |cpu| {
        handle_user_input(cpu, &mut event_pump);
        cpu.mem_write(RNG_ADDR, rng.random_range(1..16));

        if read_screen_state(cpu, &mut screen_state) {
            texture
                .update(None, &screen_state, (WIDTH * 3) as usize)
                .unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        std::thread::sleep(std::time::Duration::new(0, 70_000));
    });
}

fn read_screen_state(cpu: &Cpu, frame: &mut [u8; FRAME_LEN]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for addr in SCREEN_START..SCREEN_END {
        let color_idx = cpu.mem_read(addr);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

fn handle_user_input(cpu: &mut Cpu, ep: &mut EventPump) {
    for event in ep.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                let dir = match key {
                    Keycode::W => Some(0x77),
                    Keycode::S => Some(0x73),
                    Keycode::A => Some(0x61),
                    Keycode::D => Some(0x64),
                    _ => None,
                };

                if let Some(dir) = dir {
                    cpu.mem_write(INPUT_ADDR, dir);
                }
            }
            _ => { /* do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

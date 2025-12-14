mod cpu;
mod display;
mod utility;

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const PATH: &str = "./roms/beep.ch8";
const SHOW_FPS: bool = true;

const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;
const SCALE: u32 = 10;

const TARGET_FPS: f64 = 60.0;
const FRAME_TIME: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);

struct App<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    cpu: cpu::Cpu,
    last_timer_update: Instant,
    last_frame_time: Instant,
    frame_count: u32,
    last_fps_update: Instant,
    current_fps: f64,
}

impl<'a> App<'a> {
    fn handle_keyboard(&mut self, key_event: KeyEvent) {
        // Map keyboard keys to CHIP-8 keypad (0-F)
        // CHIP-8 keypad layout:
        // 1 2 3 C
        // 4 5 6 D
        // 7 8 9 E
        // A 0 B F

        // Modern keyboard mapping:
        // 1 2 3 4
        // Q W E R
        // A S D F
        // Z X C V

        let chip8_key = match key_event.physical_key {
            PhysicalKey::Code(KeyCode::Digit1) => Some(0x1),
            PhysicalKey::Code(KeyCode::Digit2) => Some(0x2),
            PhysicalKey::Code(KeyCode::Digit3) => Some(0x3),
            PhysicalKey::Code(KeyCode::Digit4) => Some(0xC),

            PhysicalKey::Code(KeyCode::KeyQ) => Some(0x4),
            PhysicalKey::Code(KeyCode::KeyW) => Some(0x5),
            PhysicalKey::Code(KeyCode::KeyE) => Some(0x6),
            PhysicalKey::Code(KeyCode::KeyR) => Some(0xD),

            PhysicalKey::Code(KeyCode::KeyA) => Some(0x7),
            PhysicalKey::Code(KeyCode::KeyS) => Some(0x8),
            PhysicalKey::Code(KeyCode::KeyD) => Some(0x9),
            PhysicalKey::Code(KeyCode::KeyF) => Some(0xE),

            PhysicalKey::Code(KeyCode::KeyZ) => Some(0xA),
            PhysicalKey::Code(KeyCode::KeyX) => Some(0x0),
            PhysicalKey::Code(KeyCode::KeyC) => Some(0xB),
            PhysicalKey::Code(KeyCode::KeyV) => Some(0xF),

            _ => None,
        };

        if let Some(key) = chip8_key {
            match key_event.state {
                ElementState::Pressed => {
                    self.cpu.key_press(key);
                }
                ElementState::Released => {
                    self.cpu.key_release(key);
                }
            }
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("CHIP-8 Emulator")
            .with_inner_size(winit::dpi::LogicalSize::new(
                DISPLAY_WIDTH * SCALE,
                DISPLAY_HEIGHT * SCALE,
            ));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        let pixels = Pixels::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.last_timer_update = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_keyboard(key_event);
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    render_display(&self.cpu, pixels.frame_mut());
                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time);

        if elapsed < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - elapsed);
        }
        self.last_frame_time = Instant::now();

        for _ in 0..12 {
            self.cpu.cycle();
        }

        let now = Instant::now();
        if now.duration_since(self.last_timer_update) >= Duration::from_micros(16667) {
            self.cpu.decrement_timers();
            self.last_timer_update = now;
        }

        self.cpu.end_frame();

        self.frame_count += 1;
        if SHOW_FPS && now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            self.current_fps = self.frame_count as f64;
            self.frame_count = 0;
            self.last_fps_update = now;
            println!("FPS: {}", self.current_fps);
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let mut cpu = cpu::build_cpu();
    let rom_path = PATH;
    cpu.load_rom(rom_path);

    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        window: None,
        pixels: None,
        cpu,
        last_timer_update: Instant::now(),
        last_frame_time: Instant::now(),
        frame_count: 0,
        last_fps_update: Instant::now(),
        current_fps: 0.0,
    };

    event_loop.run_app(&mut app).unwrap();
}

fn render_display(cpu: &cpu::Cpu, frame: &mut [u8]) {
    let display = cpu.get_display();

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % 64;
        let y = i / 64;

        let is_on = display[y][x] != 0;

        let color = if is_on {
            [239, 100, 97, 255]
        } else {
            [7, 79, 87, 255]
        };

        pixel.copy_from_slice(&color);
    }
}

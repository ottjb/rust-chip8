mod cpu;
mod display;

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const ROM_PATH: &str = "../roms/chip8.ch8";
const SHOW_FPS: bool = true;

const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;
const SCALE: u32 = 10;

const TARGET_FPS: f64 = 60.0;
const FRAME_TIME: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);
const TIMER_INTERVAL: Duration = Duration::from_micros(16667);
const CYCLES_PER_FRAME: usize = 12;

const COLOR_ON: [u8; 4] = [0, 255, 159, 255];
const COLOR_OFF: [u8; 4] = [10, 14, 39, 255];

struct App<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    cpu: cpu::Cpu,
    last_timer_update: Instant,
    last_frame_time: Instant,
    frame_count: u32,
    last_fps_update: Instant,
}

impl<'a> App<'a> {
    fn new(cpu: cpu::Cpu) -> Self {
        let now = Instant::now();
        Self {
            window: None,
            pixels: None,
            cpu,
            last_timer_update: now,
            last_frame_time: now,
            frame_count: 0,
            last_fps_update: now,
        }
    }
    fn handle_keyboard(&mut self, key_event: KeyEvent) {
        let chip8_key = Self::map_key_to_chip8(key_event.physical_key);

        if let Some(key) = chip8_key {
            match key_event.state {
                ElementState::Pressed => self.cpu.key_press(key),
                ElementState::Released => self.cpu.key_release(key),
            }
        }
    }

    fn map_key_to_chip8(physical_key: PhysicalKey) -> Option<u8> {
        // CHIP-8 keypad layout:     Modern keyboard mapping:
        // 1 2 3 C                   1 2 3 4
        // 4 5 6 D                   Q W E R
        // 7 8 9 E                   A S D F
        // A 0 B F                   Z X C V
        match physical_key {
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
        }
    }

    fn update_timers(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_timer_update) >= TIMER_INTERVAL {
            self.cpu.decrement_timers();
            self.last_timer_update = now;
        }
    }

    fn update_fps(&mut self) {
        if !SHOW_FPS {
            return;
        }

        self.frame_count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            println!("FPS: {}", self.frame_count);
            self.frame_count = 0;
            self.last_fps_update = now;
        }
    }

    fn render(&mut self) {
        if let Some(pixels) = &mut self.pixels {
            render_display(&self.cpu, pixels.frame_mut());
            if pixels.render().is_err() {
                eprintln!("Failed to render frame");
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

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        let pixels = Pixels::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, surface_texture)
            .expect("Failed to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.last_timer_update = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard(event),
            WindowEvent::RedrawRequested => self.render(),
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

        for _ in 0..CYCLES_PER_FRAME {
            self.cpu.cycle();
        }

        self.update_timers();
        self.cpu.end_frame();
        self.update_fps();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let mut cpu = cpu::build_cpu();
    cpu.load_rom(ROM_PATH);

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut app = App::new(cpu);

    event_loop.run_app(&mut app).expect("Event loop error");
}

fn render_display(cpu: &cpu::Cpu, frame: &mut [u8]) {
    let display = cpu.get_display();

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % 64;
        let y = i / 64;

        let is_on = display[y][x] != 0;

        let color = if is_on { COLOR_ON } else { COLOR_OFF };

        pixel.copy_from_slice(&color);
    }
}

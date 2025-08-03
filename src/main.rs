use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use rand::Rng;

mod ppu;
use crate::ppu::*;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const MICROS_PER_FRAME: u64 = 1_000_000 / 60;

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = {
        let size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("rgb")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let winsize = window.inner_size();
        let surface = SurfaceTexture::new(winsize.width, winsize.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface).unwrap()
    };

    let mut fps_counter = 0;
    let mut fps_time = Instant::now();
    let mut last_frame_time = Instant::now();

    let mut ppu = PPU::new();
    ppu.bgp = 0b11100100;

    let mut vram: Vec<u8> = vec![];
    for _ in 0..0x1800 {
        vram.push(rand::rng().random());
    }
    for _ in 0x1800..0x1fff {
        vram.push(0);
    }

    event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                if let Err(_) = pixels.render() {
                    eprintln!("pixels render error");
                    elwt.exit();
                }
                fps_counter += 1;
                if fps_time.elapsed().as_secs() == 1 {
                    let new_title = format!("rgb [{} fps]", fps_counter);
                    window.set_title(&new_title);
                    fps_counter = 0;
                    fps_time = Instant::now();
                }
            },
            Event::AboutToWait => {
                // Limit framerate to 60 FPS.
                if last_frame_time.elapsed() >= Duration::from_micros(MICROS_PER_FRAME) {
                    last_frame_time = Instant::now();
                    for _ in 0..HEIGHT {
                        ppu.draw_scanline(pixels.frame_mut(), &vram);
                    }
                    window.request_redraw();
                    ppu.scx = ppu.scx.wrapping_add(1);
                    ppu.scy = ppu.scy.wrapping_add(1);
                }
            },
            _ => ()
        }
    })
}

use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

mod ppu;
mod system;
use crate::system::*;

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

    let mut system = System::new();

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
                    system.run_frame(pixels.frame_mut());
                    window.request_redraw();
                }
            },
            _ => ()
        }
    })
}

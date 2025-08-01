use std::time::Instant;

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

extern crate rand;
use rand::Rng;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

fn main() {
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

    let mut frame_counter = 0;
    let mut frame_time = Instant::now();
    let _ = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
                return;
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                if let Err(_) = pixels.render() {
                    eprintln!("pixels render error");
                    elwt.exit();
                    return;
                }
                frame_counter += 1;
                if frame_time.elapsed().as_secs() == 1 {
                    let new_title = format!("rgb [{} fps]", frame_counter);
                    window.set_title(&new_title);
                    frame_counter = 0;
                    frame_time = Instant::now();
                }
            },
            Event::AboutToWait => {
                for pix in pixels.frame_mut().chunks_exact_mut(4) {
                    let r: u8 = rand::rng().random();
                    let g: u8 = rand::rng().random();
                    let b: u8 = rand::rng().random();
                    pix.copy_from_slice(&[r, g, b, 0xff]);
                }
                window.request_redraw();
            },
            _ => ()
        }
    });
}

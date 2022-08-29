use crate::context::Context;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

#[macro_use]
extern crate common;

#[macro_use]
extern crate egregoria;
extern crate core;

#[allow(unused_imports)]
#[macro_use]
extern crate inline_tweak;

#[macro_use]
mod uiworld;

mod audio;
mod context;
mod game_loop;
mod gui;
mod init;
mod input;
mod inputmap;
mod network;
mod rendering;

async fn run(el: EventLoop<()>, window: Window) {
    let mut ctx = Context::new(el, window).await;
    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}

fn main() {
    profiling::register_thread!("Main Thread");

    init::init();

    let el = EventLoop::new();

    #[cfg(target_arch = "wasm32")]
    {
        let window = WindowBuilder::new()
            .with_transparent(true)
            .with_title("K4 Kahlberg")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: 1422,
                height: 700,
            })
            .build(&el)
            .unwrap();

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("Failed to initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("Failed to append canvas to body");
        wasm_bindgen_futures::spawn_local(run(el, window));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        common::logger::MyLog::init();

        let size = match el.primary_monitor() {
            Some(monitor) => monitor.size(),
            None => el.available_monitors().next().unwrap().size(),
        };

        let wb = WindowBuilder::new();

        let window;
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            window = wb.with_drag_and_drop(false);
        }
        #[cfg(not(target_os = "windows"))]
        {
            window = wb;
        }
        let window = window
            .with_inner_size(winit::dpi::PhysicalSize::new(
                size.width as f32 * 0.8,
                size.height as f32 * 0.8,
            ))
            .with_title(format!("Egregoria {}", include_str!("../../VERSION")))
            .build(&el)
            .expect("Failed to create window");
        futures::executor::block_on(run(el, window))
    }
}

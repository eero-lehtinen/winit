#![allow(clippy::single_match, clippy::disallowed_methods)]

#[cfg(not(wasm_platform))]
use simple_logger::SimpleLogger;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[path = "util/fill.rs"]
mod fill;

fn main() -> Result<(), impl std::error::Error> {
    #[cfg(not(wasm_platform))]
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    #[cfg(wasm_platform)]
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("asdf");

    let event_loop = EventLoop::new().unwrap();
    let builder = WindowBuilder::new().with_title("A fantastic window!");
    #[cfg(wasm_platform)]
    let builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        builder.with_append(true)
    };
    let window = builder.build(&event_loop).unwrap();

    let mut cursor_key = 0;
    let mut cursor_visible = true;

    let cursor_image_bytes = include_bytes!("../cross.png").to_vec();
    let cursor_image_bytes2 = include_bytes!("../cross2.png").to_vec();

    window.register_custom_cursor_icon(0, cursor_image_bytes, 7, 7);
    window.register_custom_cursor_icon(1, cursor_image_bytes2, 7, 7);

    event_loop.run(move |event, _elwt| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(code),
                        ..
                    },
                ..
            } => match code {
                KeyCode::KeyA => {
                    log::debug!("Setting cursor to {:?}", cursor_key);
                    window.set_custom_cursor_icon(cursor_key);
                    cursor_key = (cursor_key + 1) % 2;
                }
                KeyCode::KeyS => {
                    log::debug!("Setting cursor icon to default");
                    window.set_cursor_icon(Default::default());
                }
                KeyCode::KeyD => {
                    cursor_visible = !cursor_visible;
                    log::debug!("Setting cursor visibility to {:?}", cursor_visible);
                    window.set_cursor_visible(cursor_visible);
                }

                _ => {}
            },
            WindowEvent::RedrawRequested => {
                #[cfg(not(wasm_platform))]
                fill::fill_window(&window);
            }
            WindowEvent::CloseRequested => {
                #[cfg(not(wasm_platform))]
                _elwt.exit();
            }
            _ => (),
        },
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    })
}

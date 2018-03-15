extern crate x11_rs as x11;

use x11::{Display, Event, Window, GC};
use std::thread;
use std::time::Duration;

fn draw(_window: &Window, _gc: &GC) {}

fn main() {
    let display = Display::open().unwrap();
    let window = Window::create(&display, 640, 480).unwrap();
    let gc = GC::create(&window).unwrap();

    window.set_title("xshm example");
    window.show();

    loop {
        let ev = window.check_event();
        match ev {
            Some(Event::Key(code)) => {
                println!("key pressed: {}", code);
                return;
            }
            Some(Event::Delete) => {
                println!("Window is closed!");
                return;
            }
            _ => draw(&window, &gc),
        }
        thread::sleep(Duration::from_millis(50));
    }
}

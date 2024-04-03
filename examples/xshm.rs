extern crate fastrand;
extern crate x11_rs as x11;

use std::thread;
use std::time::Duration;
use x11::shm::ShmImage;
use x11::{Display, Event, Window, GC};

fn main() {
    let display = Display::open().unwrap();
    let window = Window::create(&display, 640, 480).unwrap();
    let gc = GC::create(&window).unwrap();

    window.set_title("xshm example");
    window.show();

    let mut img = ShmImage::create(&display, 640, 480).unwrap();

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
            _ => {
                let x = fastrand::choice(0..img.width() - 1).unwrap();
                let y = fastrand::choice(0..img.height() - 1).unwrap();
                let c = fastrand::choice(0..0x00FFFFFF).unwrap();
                img.put_pixel(x, y, c);
                img.put_image(&window, &gc, 0, 0);
                display.sync();
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

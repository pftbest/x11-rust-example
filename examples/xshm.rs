extern crate rand;
extern crate x11_rs as x11;

use x11::{Display, Event, Window, GC};
use x11::shm::ShmImage;
use std::thread;
use std::time::Duration;
use rand::Rng;

fn main() {
    let display = Display::open().unwrap();
    let window = Window::create(&display, 640, 480).unwrap();
    let gc = GC::create(&window).unwrap();

    window.set_title("xshm example");
    window.show();

    let mut img = ShmImage::create(&display, 640, 480).unwrap();
    let mut rng = rand::thread_rng();

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
                let x = rng.gen_range(0, img.width() - 1);
                let y = rng.gen_range(0, img.height() - 1);
                let c = rng.gen_range(0, 0x00FFFFFF);
                img.put_pixel(x, y, c);
                img.put_image(&window, &gc, 0, 0);
                display.sync();
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

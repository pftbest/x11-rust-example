#[macro_use]
extern crate cstr_macro;
extern crate libc;
#[macro_use]
extern crate quick_error;
extern crate x11_sys as xlib;

mod errors;
pub mod shm;
pub use errors::*;

use std::ptr::null_mut;
use std::os::raw::*;
use std::mem;
use std::ffi::CString;

pub struct Display {
    raw: *mut xlib::Display,
}

impl Display {
    pub fn open() -> Result<Self, X11Error> {
        let display = unsafe { xlib::XOpenDisplay(null_mut()) };
        if display.is_null() {
            Err("open display")?;
        }
        Ok(Display { raw: display })
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.raw) };
    }
}

pub struct Window<'a> {
    display: &'a Display,
    window_id: xlib::Window,
    wm_protocols: xlib::Atom,
    wm_delete_window: xlib::Atom,
}

impl<'a> Window<'a> {
    pub fn create(display: &'a Display, width: u32, height: u32) -> Result<Self, X11Error> {
        let screen_num = unsafe { xlib::XDefaultScreen(display.raw) };
        let root_wnd_id = unsafe { xlib::XRootWindow(display.raw, screen_num) };

        let window_id = unsafe {
            let mut attributes: xlib::XSetWindowAttributes = mem::zeroed();
            xlib::XCreateWindow(
                display.raw,
                root_wnd_id,
                0,
                0,
                width,
                height,
                0,
                24,
                xlib::InputOutput,
                null_mut(),
                c_ulong::from(xlib::CWOverrideRedirect | xlib::CWBackPixel | xlib::CWBorderPixel),
                &mut attributes,
            )
        };
        if window_id == 0 {
            Err("create window")?;
        }

        let wm_protocols;
        let wm_delete_window;
        unsafe {
            wm_protocols =
                xlib::XInternAtom(display.raw, cstr!("WM_PROTOCOLS"), xlib::False as c_int);
            wm_delete_window =
                xlib::XInternAtom(display.raw, cstr!("WM_DELETE_WINDOW"), xlib::False as c_int);
            let mut protocols = [wm_delete_window];
            xlib::XSetWMProtocols(
                display.raw,
                window_id,
                protocols.as_mut_ptr(),
                protocols.len() as c_int,
            );
            xlib::XSelectInput(
                display.raw,
                window_id,
                c_long::from(
                    xlib::ExposureMask | xlib::KeyPressMask | xlib::ButtonPressMask
                        | xlib::StructureNotifyMask,
                ),
            );
        }

        Ok(Window {
            display,
            window_id,
            wm_protocols,
            wm_delete_window,
        })
    }

    pub fn set_title(&self, title: &str) {
        let title_str = CString::new(title).unwrap();
        unsafe { xlib::XStoreName(self.display.raw, self.window_id, title_str.as_ptr()) };
    }

    pub fn show(&self) {
        unsafe { xlib::XMapWindow(self.display.raw, self.window_id) };
        self.sync();
    }

    pub fn sync(&self) {
        unsafe { xlib::XSync(self.display.raw, xlib::False as c_int) };
    }

    pub fn check_event(&self) -> Option<Event> {
        unsafe {
            let mut event: xlib::XEvent = mem::zeroed();

            if xlib::XCheckTypedWindowEvent(
                self.display.raw,
                self.window_id,
                xlib::ClientMessage as c_int,
                &mut event,
            ) != 0
            {
                if event.xclient.message_type as xlib::Atom == self.wm_protocols
                    && event.xclient.data.l[0] as xlib::Atom == self.wm_delete_window
                {
                    return Some(Event::Delete);
                }
            }

            if xlib::XCheckWindowEvent(
                self.display.raw,
                self.window_id,
                c_long::from(xlib::KeyPressMask | xlib::ExposureMask),
                &mut event,
            ) != 0
            {
                match event.type_ as u32 {
                    xlib::KeyPress => {
                        return Some(Event::Key(event.xkey.keycode));
                    }
                    xlib::Expose => {
                        return Some(Event::Expose);
                    }
                    _ => {}
                }
            }
        }

        None
    }
}

impl<'a> Drop for Window<'a> {
    fn drop(&mut self) {
        unsafe { xlib::XDestroyWindow(self.display.raw, self.window_id) };
    }
}

pub struct GC<'a> {
    window: &'a Window<'a>,
    gc: xlib::GC,
}

impl<'a> GC<'a> {
    pub fn create(window: &'a Window) -> Result<Self, X11Error> {
        let gc = unsafe { xlib::XCreateGC(window.display.raw, window.window_id, 0, null_mut()) };
        if gc.is_null() {
            Err("create gc")?;
        }
        Ok(GC { window, gc })
    }
}

impl<'a> Drop for GC<'a> {
    fn drop(&mut self) {
        unsafe { xlib::XFreeGC(self.window.display.raw, self.gc) };
    }
}

pub enum Event {
    Key(u32),
    Delete,
    Expose,
}

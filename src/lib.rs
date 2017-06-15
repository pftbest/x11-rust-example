extern crate x11_sys as xlib;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate cstr_macro;

mod errors;
pub use errors::*;

use std::marker::PhantomData;
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
        if display == null_mut() {
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
}

impl<'a> Window<'a> {
    pub fn create(display: &'a Display, width: u32, height: u32) -> Result<Self, X11Error> {
        let screen_num = unsafe { xlib::XDefaultScreen(display.raw) };
        let root_wnd_id = unsafe { xlib::XRootWindow(display.raw, screen_num) };

        let window_id = unsafe {
            let mut attributes: xlib::XSetWindowAttributes = mem::zeroed();
            xlib::XCreateWindow(display.raw,
                                root_wnd_id,
                                0,
                                0,
                                width,
                                height,
                                0,
                                24,
                                xlib::InputOutput,
                                null_mut(),
                                (xlib::CWOverrideRedirect | xlib::CWBackPixel |
                                 xlib::CWBorderPixel) as c_ulong,
                                &mut attributes)
        };
        if window_id == 0 {
            Err("create window")?;
        }

        unsafe {
            xlib::XInternAtom(display.raw, cstr!("WM_PROTOCOLS"), xlib::False as c_int);
            let wm_delete_window =
                xlib::XInternAtom(display.raw, cstr!("WM_DELETE_WINDOW"), xlib::False as c_int);
            let mut protocols = [wm_delete_window];
            xlib::XSetWMProtocols(display.raw,
                                  window_id,
                                  protocols.as_mut_ptr(),
                                  protocols.len() as c_int);
            xlib::XSelectInput(display.raw,
                               window_id,
                               (xlib::ExposureMask | xlib::KeyPressMask | xlib::ButtonPressMask |
                                xlib::StructureNotifyMask) as
                               c_long);
        }

        Ok(Window {
               display: display,
               window_id: window_id,
           })
    }

    pub fn set_title(&self, title: &str) {
        let title_str = CString::new(title).unwrap();
        unsafe { xlib::XStoreName(self.display.raw, self.window_id, title_str.as_ptr()) };
    }

    pub fn show(&self) {
        unsafe { xlib::XMapWindow(self.display.raw, self.window_id) };
        unsafe { xlib::XSync(self.display.raw, xlib::False as c_int) };
    }

    pub fn check_event(&self) -> Option<Event> {
        let mut event: xlib::XEvent = unsafe { mem::zeroed() };
        let result = unsafe {
            xlib::XCheckWindowEvent(self.display.raw,
                                    self.window_id,
                                    xlib::KeyPressMask as c_long,
                                    &mut event)
        };
        if result == 0 {
            return None;
        }

        let ev_type = *unsafe { event.type_.as_ref() };
        match ev_type as u32 {
            xlib::KeyPress => {
                let key_event = *unsafe { event.xkey.as_ref() };
                Some(Event::Key(key_event.keycode as u32))
            }
            _ => None,
        }
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
        if gc == null_mut() {
            Err("create gc")?;
        }
        Ok(GC {
               window: window,
               gc: gc,
           })
    }
}

impl<'a> Drop for GC<'a> {
    fn drop(&mut self) {
        unsafe { xlib::XFreeGC(self.window.display.raw, self.gc) };
    }
}

pub enum Event {
    Key(u32),
}

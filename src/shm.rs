use errors::X11Error;
use libc::{shmat, shmctl, shmdt, shmget, IPC_CREAT, IPC_PRIVATE, IPC_RMID};
use std::ptr::{null, null_mut};
use xlib;
use {Display, Window, GC};

pub struct ShmImage<'a> {
    display: &'a Display,
    img: *mut xlib::_XImage,
    shm: Box<SharedMemory>,
    width: u32,
    height: u32,
}

impl<'a> ShmImage<'a> {
    pub fn create(display: &'a Display, width: u32, height: u32) -> Result<Self, X11Error> {
        let screen_num = unsafe { xlib::XDefaultScreen(display.raw) };
        let visual = unsafe { xlib::XDefaultVisual(display.raw, screen_num) };

        let size = width * height * 4;
        let mut shm = Box::new(SharedMemory::allocate(size as _)?);
        let img = unsafe {
            xlib::XShmCreateImage(
                display.raw,
                visual,
                24,
                xlib::ZPixmap as _,
                null_mut(),
                shm.as_segment_info(),
                width,
                height,
            )
        };

        if img.is_null() {
            Err("create shm image")?;
        }

        unsafe {
            assert_eq!(size, ((*img).bytes_per_line * (*img).height) as u32);
            (*img).data = shm.address() as *mut _;
            if xlib::XShmAttach(display.raw, shm.as_segment_info()) == 0 {
                XDestroyImage(img);
                Err("attach shm image")?;
            }
        }

        Ok(ShmImage {
            display,
            img,
            shm,
            width,
            height,
        })
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, c: u32) {
        unsafe {
            if let Some(put_fn) = (*self.img).f.put_pixel {
                put_fn(self.img, x as _, y as _, c as _);
            }
        }
    }

    pub fn put_image(&self, window: &Window, gc: &GC, x: i32, y: i32) {
        unsafe {
            xlib::XShmPutImage(
                self.display.raw,
                window.window_id,
                gc.gc,
                self.img,
                0,
                0,
                x,
                y,
                self.width,
                self.height,
                xlib::False as _,
            );
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[allow(non_snake_case)]
unsafe fn XDestroyImage(img: *mut xlib::_XImage) {
    if let Some(destroy_image) = (*img).f.destroy_image {
        destroy_image(img);
    }
}

impl<'a> Drop for ShmImage<'a> {
    fn drop(&mut self) {
        unsafe {
            xlib::XShmDetach(self.display.raw, self.shm.as_segment_info());
            XDestroyImage(self.img);
            self.display.sync();
        }
    }
}

struct SharedMemory(xlib::XShmSegmentInfo);

impl SharedMemory {
    pub fn allocate(size: usize) -> Result<Self, X11Error> {
        let shm_id = unsafe { shmget(IPC_PRIVATE, size, IPC_CREAT | 0o777) };
        if shm_id < 0 {
            Err("create shared memory")?;
        }

        let shm_addr = unsafe { shmat(shm_id, null(), 0) };
        if shm_addr as isize == -1 {
            unsafe { shmctl(shm_id, IPC_RMID, null_mut()) };
            Err("attach to shared memory")?;
        }

        Ok(SharedMemory(xlib::XShmSegmentInfo {
            shmseg: 0,
            shmid: shm_id,
            shmaddr: shm_addr as *mut _,
            readOnly: 0,
        }))
    }

    pub fn as_segment_info(&mut self) -> *mut xlib::XShmSegmentInfo {
        &mut self.0
    }

    pub fn address(&mut self) -> *mut u8 {
        self.0.shmaddr as *mut _
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        unsafe {
            shmdt(self.0.shmaddr as *mut _);
            shmctl(self.0.shmid, IPC_RMID, null_mut());
        }
    }
}

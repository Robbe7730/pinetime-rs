// This is ONLY fine because rtic handles the locking, only one thread can have
// an instance, so they should be safe to Send
unsafe impl Send for MCUBootFooter {}

#[derive(Debug)]
pub struct MCUBootFooter {
    start: *mut u8,
}

impl MCUBootFooter {
    pub fn get() -> Self {
        let flash_length = 475104;
        let trailer_length: usize = 40;

        let start = (0x8020 + flash_length - trailer_length) as *mut u8;

        let slice;
        unsafe {
            slice = core::slice::from_raw_parts_mut(
                start,
                trailer_length,
            )
        }

        if slice[24..40] != [
            0x77, 0xc2, 0x95, 0xf3,
            0x60, 0xd2, 0xef, 0x7f,
            0x35, 0x52, 0x50, 0x0f,
            0x2c, 0xb6, 0x79, 0x80
        ] {
            panic!("Invalid magic for MCUBoot footer");
        }

        MCUBootFooter {
            start
        }
    }

    // Marking this function as unsafe to discourage its usage
    pub unsafe fn mark_valid(&mut self) {
        *self.start.offset(0x10) = 0x01;
    }

    pub fn is_valid(&self) -> bool {
        unsafe {
            return *self.start.offset(0x10) == 0x01;
        }
    }
}

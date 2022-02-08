// This is ONLY fine because rtic handles the locking, only one thread can have
// an instance, so they should be safe to Send
unsafe impl Send for MCUBootHeader {}

#[derive(Debug)]
pub struct MCUBootHeaderVersion {
    pub major: u8,
    pub minor: u8,
    pub revision: u16,
    pub build_num: u32,
}

#[derive(Debug)]
pub struct MCUBootHeader {
    pub version: MCUBootHeaderVersion,
}

impl MCUBootHeader {
    pub fn get() -> Self {
        let header_length = 0x20;
        let start = 0x8000 as *mut u8;

        let slice;
        unsafe {
            slice = core::slice::from_raw_parts_mut(
                start,
                header_length,
            )
        }

        if slice[0..4] != [
            0x3d, 0xb8, 0xf3, 0x96
        ] {
            panic!("Invalid magic for MCUBoot header");
        }

        let version = unsafe {
            MCUBootHeaderVersion {
                major: *start.offset(20),
                minor: *start.offset(21),
                revision: u16::from_le_bytes([
                    *start.offset(22),
                    *start.offset(23),
                ]),
                build_num: u32::from_le_bytes([
                    *start.offset(24),
                    *start.offset(25),
                    *start.offset(26),
                    *start.offset(27),
                ]),
            }
        };

        MCUBootHeader {
            version
        }
    }
}

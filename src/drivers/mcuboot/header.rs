use crate::drivers::flash::InternalFlash;

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
    pub fn get(internal_flash: &mut InternalFlash) -> Self {
        let mut data = [0; 32];
        internal_flash.read(0x00000000, &mut data).unwrap();
        if data[0..4] != [
            0x3d, 0xb8, 0xf3, 0x96
        ] {
            panic!("Invalid magic for MCUBoot header");
        }

        let version = MCUBootHeaderVersion {
            major: data[20],
            minor: data[21],
            revision: u16::from_le_bytes([
                data[22],
                data[23],
            ]),
            build_num: u32::from_le_bytes([
                data[24],
                data[25],
                data[26],
                data[27],
            ]),
        };

        MCUBootHeader {
            version
        }
    }
}

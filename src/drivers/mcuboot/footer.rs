use crate::drivers::flash::InternalFlash;

const FOOTER_START: u32 = 475096;
const FOOTER_MAGIC: [u8; 16] = [
    0x77, 0xc2, 0x95, 0xf3,
    0x60, 0xd2, 0xef, 0x7f,
    0x35, 0x52, 0x50, 0x0f,
    0x2c, 0xb6, 0x79, 0x80
];

#[derive(Debug)]
pub struct MCUBootFooter {
    pub is_valid: bool,
}

impl MCUBootFooter {
    pub fn get(internal_flash: &mut InternalFlash) -> Self {
        let mut footer = [0; 40];
        internal_flash.read(FOOTER_START, &mut footer).unwrap();
        if footer[24..40] != FOOTER_MAGIC {
            panic!("Invalid magic for MCUBoot footer"); 
        }

        MCUBootFooter {
            is_valid: footer[16] == 1,
        }
    }

    pub fn write(&self, internal_flash: &mut InternalFlash) {
        let mut footer = [0xff; 40];

        for i in 0..16 {
            footer[i + 24] = FOOTER_MAGIC[i];
        }

        if self.is_valid {
            footer[16] = 0x01;
        }

        // The reason we don't have to erase here is because 0xff can be set to
        // 0x01, the only two possible values.
        internal_flash.write(FOOTER_START, &footer).unwrap();
    }
}

use nrf52832_hal::nvmc::{Nvmc, NvmcError};
use nrf52832_hal::pac::NVMC;

use embedded_storage::nor_flash::{ReadNorFlash, NorFlash};

pub struct InternalFlash {
    nvmc: Nvmc<NVMC>,
}

const FLASH_SIZE: usize = 464 * 1024; // 464KiB


extern "C" {
    #[link_name = "flash_start"]
    static mut FOOTER: [u8; FLASH_SIZE];
}

impl InternalFlash {
    pub fn new(nvmc: NVMC) -> InternalFlash {
        InternalFlash {
            nvmc: Nvmc::new(nvmc, unsafe { &mut FOOTER })
        }
    }

    // Note: all offsets are relative to 0x8000, not to 0x0
    pub fn read(&mut self, offset: u32, buffer: &mut [u8]) -> Result<(), NvmcError> {
        self.nvmc.read(offset, buffer)
    }

    pub fn write(&mut self, offset: u32, buffer: &[u8]) -> Result<(), NvmcError> {
        self.nvmc.write(offset, buffer)
    }

    pub fn erase(&mut self, start: u32, end: u32) -> Result<(), NvmcError> {
        self.nvmc.erase(start, end)
    }
}

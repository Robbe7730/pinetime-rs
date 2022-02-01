use nrf52832_hal::spim::Spim;
use nrf52832_hal::pac::SPIM0;

pub struct FlashMemory {
    // Spi can be 'static because it is accessible as long as the device is
    // powered on.
    spi: &'static Spim<SPIM0>,
}

impl FlashMemory {
    pub fn new(spi: &'static Spim<SPIM0>) -> FlashMemory {
        FlashMemory {
            spi,
        }
    }
}

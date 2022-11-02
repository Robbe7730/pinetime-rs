mod phy;

use nrf52832_hal::pac::{RADIO, FICR};

use self::phy::PhyRadio;

pub struct Bluetooth {
    phy: PhyRadio,
}

impl Bluetooth {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        packet_buffer: &'static mut [u8; 258]
    ) -> Self {
        let phy = PhyRadio::new(radio, ficr, packet_buffer);

        Bluetooth {
            phy,
        }
    }

    pub fn on_radio_interrupt(&mut self) {
        self.phy.on_radio_interrupt();
    }
}

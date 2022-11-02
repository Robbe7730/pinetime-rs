use nrf52832_hal::pac::{RADIO, FICR, radio::txpower::TXPOWER_A};

use alloc::vec::Vec;

#[derive(Debug)]
pub struct InvalidStateError {}

#[derive(Debug, PartialEq)]
pub enum PhyState {
    Disabled,
    RxRU,
    RxIdle,
    Rx,
    TxRU,
    TxIdle,
    Tx,
    TxDisable,
    RxDisable,
}

impl From<u32> for PhyState {
    fn from(number: u32) -> Self {
        match number {
            0 => PhyState::Disabled,
            1 => PhyState::RxRU,
            2 => PhyState::RxIdle,
            3 => PhyState::Rx,
            4 => PhyState::RxDisable,
            9 => PhyState::TxRU,
            10 => PhyState::TxIdle,
            11 => PhyState::Tx,
            12 => PhyState::TxDisable,
            x => panic!("Invalid RADIO state: {}", x)
        }
    }
}

#[derive(Debug)]
pub struct PhyRadioPacket {
    payload: Vec<u8>,
}

#[derive(Debug)]
pub struct PhyRadio {
    radio: RADIO,
    ficr: FICR,
    packet_buffer: &'static mut [u8; 258],
    packet_queue: Vec<PhyRadioPacket>,
}

impl PhyRadio {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        packet_buffer: &'static mut  [u8; 258]
    ) -> Self {
        let ret = Self {
            radio,
            ficr,
            packet_buffer,
            packet_queue: Vec::new(),
        };

        // Set MODE to 1 Mbit/s BLE
        ret.radio.mode.write(|w| w.mode().ble_2mbit());

        // PCNF0 is set to al 0 on reset, which disables S0, LENGTH and S1
        // and sets preable to 8 Mbit/s
        ret.radio.pcnf0.reset();

        // Set PCNF1
        unsafe {
            ret.radio.pcnf1.write(|w| w
                // Set max packet length to 255
                .maxlen().bits(255)
                // Set base address length to 3 bytes (total 4 bytes)
                .balen().bits(3)
                // Enable whitening
                // "Whitening shall be applied on the PDU and CRC ... No other
                // parts of the packets are whitened"
                .whiteen().bit(true)
            )
        }

        // Set RX/TX address(es)
        // 0 -> Advertising access address
        // TODO: 1 -> Random address
        // According to rubble, BASE0 uses the upper 3 bytes instead of the lower 3
        unsafe {
            let advertising_access_address = 0x8E89BED6;
            ret.radio.base0.write(|w| w
                .base0().bits(advertising_access_address << 8)
            );
            ret.radio.prefix0.write(|w| w
                .ap0().bits((advertising_access_address >> 24) as u8)
            );
        }

        // Set TX Address to 0 (advertising, set above)
        unsafe {
            ret.radio.txaddress.write(|w| w.txaddress().bits(0));
        }

        // Enable CRC
        ret.radio.crccnf.write(|w| w
            .len().bits(3)        // "The CRC is 24 bits in length and"
            .skipaddr().bit(true) // "the value is calculated over all the PDU bits"
        );

        // Set CRC polynomial
        unsafe {
            // x^24 + x^10 + x^9 + x^6 + x^4 + x^3 + x^1 + x^0.
            ret.radio.crcpoly.write(|w| w
                .crcpoly().bits(
                    // (1 << 24) | Only use 24 least significant bits
                    (1 << 10) |
                    (1 <<  9) |
                    (1 <<  6) |
                    (1 <<  4) |
                    (1 <<  3) |
                    (1 <<  1) |
                    (1 <<  0) // Hard-wired to be 1, but it's better to set it anyway
                )
            )
        }

        // Set TIFS to 150µs
        unsafe {
            ret.radio.tifs.write(|w| w
                .tifs().bits(150u8)
            )
        }

        // Set packet pointer
        unsafe {
            let packetptr = ret.packet_buffer.as_mut_ptr() as u32;
            ret.radio.packetptr.write(|w| w
                .packetptr().bits(packetptr)
            );
        }

        ret.enable_interrupts();

        ret
    }

    fn set_frequency(&self, frequency_mhz: usize) {
        if frequency_mhz > 2500 || frequency_mhz < 2400 {
            panic!("Invalid frequency {} MHz", frequency_mhz)
        }

        unsafe {
            self.radio.frequency.write(|w| w
                .frequency().bits((frequency_mhz - 2400) as u8)
                .map().bit(false)
            )
        }
    }

    fn set_tx_power(&self, value: TXPOWER_A) {
        self.radio.txpower.write(|w| w
            .txpower().variant(value)
        )
    }

    fn set_powered(&self, value: bool) {
        self.radio.power.write(|w| w
            .power().bit(value)
        )
    }

    fn set_crc_initial(&self, value: u32) {
        if value > 0x00ffffff {
            panic!("Invalid initial CRC value: {:x}", value);
        }
        unsafe {
            // All 24 bit values are allowed
            self.radio.crcinit.write(|w| w
                .crcinit().bits(value)
            )
        }
    }

    fn tx_enable(&self) -> Result<(), InvalidStateError> {
        if self.get_state() != PhyState::Disabled {
            return Err(InvalidStateError {});
        }

        // Writing 1 to a task register fires the task
        unsafe {
            self.radio.tasks_txen.write(|w| w.bits(1));
        }

        Ok(())
    }

    pub fn get_state(&self) -> PhyState {
        return self.radio.state.read().bits().into();
    }

    pub fn set_interrupts(&self, value: bool) {
        self.radio.intenset.write(|w| w
            .ready().bit(value)
            // .end().bit(value)
            // .disabled().bit(value)
            // .address().bit(value)
            // .payload().bit(value)
        )
    }

    pub fn enable_interrupts(&self) {
        self.set_interrupts(true);
    }

    pub fn disable_interrupts(&self) {
        self.set_interrupts(false);
    }

    pub fn on_radio_interrupt(&mut self) {
        self.disable_interrupts();
        if self.radio.events_ready.read().bits() != 0 {
            // Acknowledge READY
            self.radio.events_ready.reset();

            if self.get_state() == PhyState::TxIdle {
                //self.transmit_next_packet();
            }
        }
        self.enable_interrupts();
    }
}

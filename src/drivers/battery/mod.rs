use nrf52832_hal::gpio::{Pin, Input, Floating};
use nrf52832_hal::prelude::InputPin;
use nrf52832_hal::saadc::Saadc;
use nrf52832_hal::prelude::_embedded_hal_adc_OneShot;
use nrf52832_hal::gpio::p0::P0_31;

#[derive(Debug, Clone, Copy)]
pub enum BatteryState {
    Discharging(f32),
    Charging(f32),
    Unknown
}

pub struct Battery {
    // High = Discharging, Low = Charging
    charging_state_pin: Pin<Input<Floating>>,

    // Voltage
    voltage_pin: P0_31<Input<Floating>>,

    // SAADC (Analog-to-Digital Converter)
    saadc: Saadc,

    state: BatteryState,
}

impl Battery {
    pub fn new(
        charging_state_pin: Pin<Input<Floating>>,
        voltage_pin: P0_31<Input<Floating>>,
        saadc: Saadc,
    ) -> Battery {
        Battery {
            charging_state_pin,
            voltage_pin,
            saadc,
            state: BatteryState::Unknown
        }
    }

    pub fn get_state(&mut self) -> BatteryState {
        let value_adc = self.saadc.read(&mut self.voltage_pin).unwrap() as u16;

        let value: f32 = (value_adc as f32) * (6.6 / f32::from(u16::MAX));

        self.state = match self.charging_state_pin.is_high() {
            Ok(true)  => BatteryState::Discharging(value),
            Ok(false) => BatteryState::Charging(value),
            Err(_)    => BatteryState::Unknown
        };
        return self.state;
    }
}

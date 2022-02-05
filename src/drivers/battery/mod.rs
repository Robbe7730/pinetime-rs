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

    pub fn get_voltage(&mut self) -> f32 {
        let voltage_adc = self.saadc.read(&mut self.voltage_pin).unwrap() as u16;

        // voltage_adc is 14 bit, so divide by 2 ** 14, times reference voltage
        // (3.3V), times 2
        (voltage_adc as f32) / 16384.0  * 3.3 * 2.0
    }

    pub fn get_state(&mut self) -> BatteryState {
        let voltage_full = 4.2;
        let voltage_empty = 3.2;

        let voltage = self.get_voltage();

        // This should probably be calculated using a curve, but this works for now
        let percentage: f32;
        if voltage < voltage_empty {
            percentage = 0.0;
        } else if voltage > voltage_full {
            percentage = 100.0;
        } else {
            percentage = ((voltage - voltage_empty) * 100.0) / (voltage_full - voltage_empty);
        }

        self.state = match self.charging_state_pin.is_high() {
            Ok(true)  => BatteryState::Discharging(percentage),
            Ok(false) => BatteryState::Charging(percentage),
            Err(_)    => BatteryState::Unknown
        };
        return self.state;
    }
}

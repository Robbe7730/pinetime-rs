use embedded_graphics::pixelcolor::Rgb565;
use nrf52832_hal::pac::{SPIM0, RTC1};

pub type PixelType = Rgb565;
pub type ConnectedSpim = SPIM0;
pub type ConnectedRtc = RTC1;

pub mod tasks_impl;

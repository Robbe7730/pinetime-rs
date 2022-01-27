#![no_main]
#![no_std]

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

// Need to be used to init vectors
use nrf52832_hal;

use core::panic::PanicInfo;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("Hello, world!");

    todo!("Rest of the code")
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("----- PANIC -----");
    rprintln!("{:#?}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}

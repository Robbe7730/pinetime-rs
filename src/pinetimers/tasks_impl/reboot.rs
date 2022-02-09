use nrf52832_hal::pac::SCB;

pub fn reboot(_ctx: crate::tasks::reboot::Context) {
    SCB::sys_reset();
}

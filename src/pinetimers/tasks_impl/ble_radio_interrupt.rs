use rtic::Mutex;

pub fn ble_radio_interrupt(mut ctx: crate::tasks::ble_radio_interrupt::Context) {
    ctx.shared.bluetooth.lock(|bluetooth| {
        bluetooth.on_radio_interrupt()
    });
}

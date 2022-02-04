use rtic::Mutex;

pub fn ble_radio(mut ctx: crate::tasks::ble_radio::Context) {
    ctx.shared.bluetooth.lock(|bluetooth| {
        bluetooth.on_radio()
    });
}

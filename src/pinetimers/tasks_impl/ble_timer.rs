use rtic::Mutex;

pub fn ble_timer(mut ctx: crate::tasks::ble_timer::Context) {
    ctx.shared.bluetooth.lock(|bluetooth| {
        bluetooth.on_timer();
    });
}

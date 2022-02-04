use rtic::Mutex;

pub fn ble_worker(mut ctx: crate::tasks::ble_worker::Context) {
    ctx.shared.bluetooth.lock(|bluetooth| {
        bluetooth.work()
    })
}

use rtic::Mutex;

pub fn ble_worker(mut ctx: crate::tasks::ble_worker::Context) {
    ctx.shared.ble_r.lock(|ble_r| {
        while ble_r.has_work() {
            ble_r.process_one().unwrap();
        }
    })
}

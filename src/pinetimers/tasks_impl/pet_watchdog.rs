use rtic::Mutex;

use fugit::ExtU32;

pub fn pet_watchdog(mut ctx: crate::tasks::pet_watchdog::Context) {
    crate::tasks::pet_watchdog::spawn_after(5.secs()).unwrap();
    ctx.shared.watchdog_handles.lock(|watchdog_handles| {
        // Good doggy
        for watchdog_handle in watchdog_handles {
            watchdog_handle.pet();
        }
    });
}

use rtic::Mutex;

use fugit::ExtU32;

pub fn periodic_update_device_state(mut ctx: crate::tasks::periodic_update_device_state::Context) {
    crate::tasks::periodic_update_device_state::spawn_after(1.secs()).unwrap();

    ctx.shared.clock.lock(|clock| {
        clock.tick();
    });

    crate::tasks::redraw_screen::spawn().unwrap();
}

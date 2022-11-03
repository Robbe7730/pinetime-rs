use rtic::mutex_prelude::TupleExt05;

pub fn init_screen(ctx: crate::tasks::init_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.clock,
        ctx.shared.mcuboot,
        ctx.shared.bluetooth
    ).lock(|display, current_screen, clock, mcuboot, bluetooth| {
        current_screen.draw_init(display, clock, mcuboot, bluetooth);
    });
}

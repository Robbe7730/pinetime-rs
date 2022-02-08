use rtic::mutex_prelude::TupleExt04;

pub fn init_screen(ctx: crate::tasks::init_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.clock,
        ctx.shared.mcuboot
    ).lock(|display, current_screen, clock, mcuboot| {
        current_screen.draw_init(display, clock, mcuboot);
    });
}

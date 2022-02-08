use rtic::mutex_prelude::TupleExt04;

pub fn redraw_screen(ctx: crate::tasks::redraw_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.clock,
        ctx.shared.mcuboot,
    ).lock(|display, current_screen, clock, mcuboot| {
        current_screen.draw_update(display, clock, mcuboot);
    });
}

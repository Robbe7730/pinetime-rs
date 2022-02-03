use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics_core::draw_target::DrawTarget;
use rtic::Mutex;

pub fn display_init(mut ctx: crate::tasks::display_init::Context) {
    ctx.shared.display.lock(|display| {
        display.init();
        display.clear(RgbColor::BLACK).unwrap();
    });
    crate::tasks::periodic_update_device_state::spawn().unwrap();
}

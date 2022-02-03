use alloc::boxed::Box;

use rtic::Mutex;

use crate::drivers::display::Display;
use crate::ui::screen::Screen;
use crate::pinetimers::{PixelType, ConnectedSpim};

pub fn transition(mut ctx: crate::tasks::transition::Context, new_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>) {
    ctx.shared.current_screen.lock(|current_screen| {
        *current_screen = new_screen;
    });
    crate::tasks::init_screen::spawn().unwrap();
    crate::tasks::redraw_screen::spawn().unwrap();
}

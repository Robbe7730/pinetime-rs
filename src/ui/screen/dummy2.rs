use crate::ui::screen::{Screen, ScreenDummy1};
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::{Display, DisplaySupported};

use embedded_graphics::prelude::{Point, Drawable};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::{Text, Baseline};

use core::marker::PhantomData;
use core::fmt::Debug;

use alloc::sync::Arc;
use alloc::boxed::Box;

#[derive(Debug)]
pub struct ScreenDummy2<COLOR> {
    event_handler: Arc<ScreenDummy2EventHandler>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenDummy2EventHandler {}

impl TouchPanelEventHandler for ScreenDummy2EventHandler {
    fn on_slide_left(&self, _point: TouchPoint) {
        crate::pinetimers::transition::spawn(Box::new(ScreenDummy1::new())).unwrap();
    }
}

impl<COLOR : RgbColor + Send + Debug> Screen<COLOR> for ScreenDummy2<COLOR>
where
    Display<COLOR>: DisplaySupported<COLOR>,
{
    fn new() -> ScreenDummy2<COLOR> {
        ScreenDummy2 {
            event_handler: Arc::new(ScreenDummy2EventHandler {}),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> { 
        return self.event_handler.clone();
    }

    fn draw(&self, display: &mut Display<COLOR>) {
        let mut character_style = MonoTextStyle::new(&FONT_10X20, COLOR::BLACK);
        character_style.set_background_color(Some(COLOR::WHITE));
        Text::with_baseline("Screen 2", Point::new(0, 0), character_style, Baseline::Top)
            .draw(display)
            .unwrap();
    }
}

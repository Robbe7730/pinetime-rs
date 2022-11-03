use crate::drivers::bluetooth::Bluetooth;
use crate::ui::screen::{Screen, ScreenMain};
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::DisplaySupported;
use crate::drivers::clock::Clock;
use crate::drivers::mcuboot::MCUBoot;

use crate::pinetimers::ConnectedRtc;

use embedded_graphics::pixelcolor::{RgbColor, PixelColor};
use embedded_graphics::prelude::{Drawable, Point, DrawTarget};
use embedded_graphics::image::{Image, ImageRawLE, ImageDrawable};

use core::marker::PhantomData;
use core::fmt::Debug;

use alloc::sync::Arc;
use alloc::boxed::Box;

use tinybmp::Bmp;

#[derive(Debug)]
pub struct ScreenPoes<COLOR> {
    event_handler: Arc<ScreenPoesEventHandler>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenPoesEventHandler {}

impl TouchPanelEventHandler for ScreenPoesEventHandler {
    fn on_event(&self, _point: TouchPoint) {
        crate::tasks::transition::spawn(Box::new(ScreenMain::new())).unwrap();
    }
}

impl<'a, DISPLAY, COLOR> Screen<DISPLAY> for ScreenPoes<DISPLAY>
where
    DISPLAY: DisplaySupported<COLOR> + DrawTarget<Color = COLOR> + Send + Debug,
    <DISPLAY as DrawTarget>::Error: Debug,
    COLOR: From<<COLOR as PixelColor>::Raw> + RgbColor,
    ImageRawLE<'a, COLOR>: ImageDrawable<Color = COLOR>,
{
    fn new() -> ScreenPoes<DISPLAY> {
        ScreenPoes {
            event_handler: Arc::new(ScreenPoesEventHandler {}),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> { 
        return self.event_handler.clone();
    }

    fn draw_update(&mut self, _display: &mut DISPLAY, _devicestate: &Clock<ConnectedRtc>, _: &MCUBoot, _: &Bluetooth) {}

    fn draw_init(&mut self, display: &mut DISPLAY, _devicestate: &Clock<ConnectedRtc>, _: &MCUBoot, _: &Bluetooth) {
        let bmp_data = include_bytes!("../../../poes565.bmp");
        let image = Bmp::<COLOR>::from_slice(bmp_data).unwrap();
        Image::new(&image, Point::new(0,0))
            .draw(display)
            .unwrap();
    }
}

use embedded_graphics_core::pixelcolor::PixelColor;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size, Dimensions};
use embedded_graphics_core::primitives::Rectangle;
use embedded_graphics_core::prelude::PointsIter;
use embedded_graphics_core::Pixel;

use nrf52832_hal::prelude::OutputPin;

use crate::display::{Display, DisplaySupported};
use crate::display::commands::DisplayCommand;

use core::convert::Infallible;

impl<T> OriginDimensions for Display<T> {
    fn size(&self) -> Size {
        Size::new(240, 240)
    }
}

impl<PIXEL : PixelColor> DrawTarget for Display<PIXEL>
where
    Display<PIXEL>: DisplaySupported<PIXEL>
{
    type Color = PIXEL;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>> {
        pixels.into_iter().for_each(|p| {
            if self.bounding_box().contains(p.0) {
                let (col, row, color) = (
                    p.0.x.try_into().unwrap(),
                    p.0.y.try_into().unwrap(),
                    p.1
                );
                self.select_area(row, col, row+1, col+1);
                self.transmit_color(color);
            }
        });

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());
        self.select_area(
            drawable_area.rows().start.try_into().unwrap(),
            drawable_area.columns().start.try_into().unwrap(),
            u16::try_from(drawable_area.rows().end).unwrap() - 1,
            u16::try_from(drawable_area.columns().end).unwrap() - 1,
        );

        self.pin_chip_select.set_low().unwrap();

        self.send_no_cs(DisplayCommand::StartRamWrite);
        area.points()
            .zip(colors)
            .for_each(|(p, c)| {
                if self.bounding_box().contains(p) {
                    self.transmit_color(c);
                }
            });

        self.pin_chip_select.set_high().unwrap();

        Ok(())
    }
}

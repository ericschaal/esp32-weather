use std::time::Duration;
use anyhow::{Result};
use epd_waveshare::{
    color::Color::{Black as White, White as Black},
    epd7in5_v2::{Epd7in5, WIDTH, HEIGHT},
    graphics::{VarDisplay},
    prelude::*
};

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use esp_idf_hal::
{delay, gpio, peripheral, spi
};
use esp_idf_hal::gpio::{PinDriver};
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::spi::{Dma, SpiDriverConfig, SpiConfig};
use crate::display::{DisplayManager, DisplayManagerPins};

const SCREEN_BUFFER_SIZE: usize =  WIDTH as usize / 8 * HEIGHT as usize;
const DEFAULT_COLOR: Color = White;

impl<'a> DisplayManager<'a> {
    pub fn new(
        spi: impl peripheral::Peripheral<P = impl spi::SpiAnyPins> + 'static,
        pins: DisplayManagerPins,
        buffer: &'a mut [u8]) -> Result<Self> {

        let mut driver = spi::SpiDeviceDriver::new_single(
            spi,
            pins.sclk,
            pins.sdo,
            Option::<gpio::AnyIOPin>::None,
            Option::<gpio::AnyOutputPin>::None,
            &SpiDriverConfig::new().dma(Dma::Disabled),
            &SpiConfig::new().baudrate(12.MHz().into()),
        )?;

        let epd = Epd7in5::new(
            &mut driver,
            PinDriver::output(pins.cs)?,
            PinDriver::input(pins.busy)?,
            PinDriver::output(pins.dc)?,
            PinDriver::output(pins.rst)?,
            &mut delay::Ets,
            Some(100),
        )?;

        let display = VarDisplay::new(
            WIDTH,
            HEIGHT,
            buffer,
            false
        ).unwrap();

        Ok(Self {
            display,
            driver,
            epd,
        })
    }

    pub fn hello_world(&mut self) -> Result<()> {
        let style = MonoTextStyleBuilder::new()
            .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
            .text_color(White)
            .background_color(Black)
            .build();
        let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

        let _ = Text::with_text_style("Hello World!", Point::new(90, 10), style, text_style)
            .draw(&mut self.display);

        // Display updated frame
        self.epd.update_frame(&mut self.driver, self.display.buffer(), &mut delay::Ets)?;
        self.epd.display_frame(&mut self.driver, &mut delay::Ets)?;

        Ok(())
    }
    #[inline]
    pub fn new_buffer() -> Vec<u8> {
        vec![DEFAULT_COLOR.get_byte_value(); SCREEN_BUFFER_SIZE]
    }
}



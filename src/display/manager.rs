use anyhow::{Result};
use epd_waveshare::{
    color::Color::{Black as White, White as Black},
    epd7in5_v2::{Epd7in5, WIDTH, HEIGHT},
    graphics::{VarDisplay},
    prelude::*
};

use embedded_graphics::{
    prelude::*,
    image::{ImageRaw, Image},
    pixelcolor::{BinaryColor},
    primitives:: {Rectangle},
};
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use u8g2_fonts::{
    FontRenderer,
    fonts,
    types::{HorizontalAlignment, VerticalPosition, FontColor},
};

use esp_idf_hal::
{delay, gpio, peripheral, spi
};
use esp_idf_hal::gpio::{PinDriver};
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::spi::{Dma, SpiDriverConfig, SpiConfig};

use crate::display::{DisplayManager, DisplayManagerPins, DisplayRect};
use crate::icons::i196x196::WI_DAY_SNOW_THUNDERSTORM_196X196;


const SCREEN_BUFFER_SIZE: usize =  WIDTH as usize / 8 * HEIGHT as usize;
const DEFAULT_COLOR: Color = White;

const MARGIN: u32 = 8;

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

        let viewport = Rectangle::new(Point::new(MARGIN as i32, MARGIN as i32), Size::new(WIDTH - MARGIN, HEIGHT - MARGIN));
        let current_weather = Rectangle::new(viewport.top_left, Size::new(2 * 196,196));
        let weather_icon = Rectangle::new(current_weather.top_left, Size::new(196,196));
        let current_temp = Rectangle::new(weather_icon.anchor_point(AnchorPoint::TopRight), Size::new(196,196));


        let rect = DisplayRect {
            viewport,
            current_weather,
            weather_icon,
            current_temp,
        };

        Ok(Self {
            display,
            driver,
            epd,
            rect
        })
    }

    pub fn build_frame(&mut self) -> Result<()> {
        self.current_weather_icon()?;
        self.current_temperature_txt()?;
        self.draw_rect()?;

        self.update_frame()?;
        self.display_frame()?;

        Ok(())
    }

    pub fn draw_rect(&mut self) -> Result<()> {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Black)
            .fill_color(White)
            .stroke_width(1)
            .build();
        self.rect.viewport.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        Ok(())
    }

    pub fn current_weather_icon(&mut self) -> Result<()> {
        let raw_image = ImageRaw::<BinaryColor>::new(WI_DAY_SNOW_THUNDERSTORM_196X196 , 196);
        Image::new(&raw_image, self.rect.current_weather.top_left)
            .draw(&mut self.display.color_converted())?;
        Ok(())
    }

    pub fn current_temperature_txt(&mut self) -> Result<()> {
        let font = FontRenderer::new::<fonts::u8g2_font_logisoso92_tn>();
        let text = "32";

        let _ = font.render_aligned(
            text,
            self.rect.current_temp.top_left,
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(Black),
            &mut self.display.color_converted(),
        ).unwrap();

        Ok(())
    }

    fn update_frame(&mut self) -> Result<()> {
        self.epd.update_frame(&mut self.driver, self.display.buffer(), &mut delay::Ets)?;
        Ok(())
    }

    fn display_frame(&mut self) -> Result<()> {
        self.epd.display_frame(&mut self.driver, &mut delay::Ets)?;
        Ok(())
    }

    #[inline]
    pub fn new_buffer() -> Vec<u8> {
        vec![DEFAULT_COLOR.get_byte_value(); SCREEN_BUFFER_SIZE]
    }
}



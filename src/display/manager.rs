use std::fmt::Debug;
use anyhow::{Result};
use epd_waveshare::{
    color::Color::{Black as White, White as Black},
    epd7in5_v2::{Epd7in5, WIDTH, HEIGHT},
    graphics::{VarDisplay},
    prelude::*
};

use embedded_graphics::{
    prelude::*,
    image::{Image},
    geometry::{AnchorPoint},
    primitives:: {Rectangle, Circle, PrimitiveStyleBuilder},
};
use tinyqoi::Qoi;
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

use crate::config::CONFIG;
use crate::display::{DisplayManager, DisplayManagerPins, DisplayRect};
use crate::icons::WeatherIconSet;
use crate::owm::icons::get_icon_for_current_weather;
use crate::owm::model::WeatherData;

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

        let viewport_size = Size::new(WIDTH - MARGIN, HEIGHT - MARGIN);
        let current_icon_size = Size::new(196, 196);
        let current_temp_size = Size::new(196, current_icon_size.height);
        let current_weather_size = Size::new(current_temp_size.width+ current_icon_size.width, current_icon_size.height);

        let temp_unit_size = Size::new(32, current_temp_size.height);
        let temp_feels_like_size = Size::new(current_temp_size.width, 32);

        let date_location_size = Size::new(viewport_size.width - current_weather_size.width, 64);
        let forecast_size = Size::new(viewport_size.width - current_weather_size.width, current_weather_size.height - date_location_size.height);

        let viewport = Rectangle::new(Point::new(MARGIN as i32, MARGIN as i32), viewport_size);
        let current_weather = Rectangle::new(viewport.top_left, current_weather_size);
        let weather_icon = Rectangle::new(current_weather.top_left, current_icon_size);

        let current_temp = Rectangle::new(weather_icon.anchor_point(AnchorPoint::TopRight), current_temp_size);

        let current_temp_unit = Rectangle::new(current_temp.anchor_point(AnchorPoint::TopRight), temp_unit_size)
            .translate(Point::new(-(temp_unit_size.width as i32), 0));

        let feels_like = Rectangle::new(current_temp.anchor_point(AnchorPoint::BottomLeft), temp_feels_like_size)
            .translate(Point::new(0, -(temp_feels_like_size.height as i32)));

        let date_location = Rectangle::new(current_temp.anchor_point(AnchorPoint::TopRight), date_location_size);
        let forecast = Rectangle::new(date_location.anchor_point(AnchorPoint::BottomLeft), forecast_size);

        let rect = DisplayRect {
            viewport,
            current_weather,
            weather_icon,
            current_temp,
            feels_like,
            current_temp_unit,
            date_location,
            forecast
        };

        Ok(Self {
            display,
            driver,
            epd,
            rect
        })
    }

    pub fn draw_weather_report(&mut self, data: WeatherData) -> Result<()> {
        let app_config = CONFIG;
        let location_name = app_config.location_name;
        let current = data.current.unwrap();
        let dt = current.dt;
        // let feels_like = current.feels_like;
        // let temp = current.temp;

        // let large_icon_set = WeatherIconSet::new()?;
        // let _ = WeatherIconSet::new_small()?;
        // let icon = get_icon_for_current_weather(&large_icon_set, &current);

        // self.current_weather_icon(&icon)?;
        // self.current_temperature(temp)?;
        // self.current_feels_like(feels_like)?;
        // self.current_temp_unit()?;
        self.debug_draw_rect()?;
        self.date_and_location(dt, location_name)?;

        self.update_frame()?;
        self.display_frame()?;

        Ok(())
    }

    fn current_weather_icon(&mut self, icon: &Qoi) -> Result<()> {
        Image::new(icon, self.rect.current_weather.top_left)
            .draw(&mut self.display.color_converted())?;
        Ok(())
    }

    fn current_temp_unit(&mut self) -> Result<()> {
        let unit_style = PrimitiveStyleBuilder::new()
            .stroke_width(4)
            .stroke_color(Black)
            .build();

        let circle_diameter: u32 = 12;
        let circle_center = Point::new(circle_diameter as i32 / 2, circle_diameter as i32 / 2);
        let offset = Point::new(0, 46);
        Circle::new(
            self.rect.current_temp_unit.center() - offset - circle_center,
            circle_diameter
        ).into_styled(unit_style)
            .draw(&mut self.display.color_converted())?;

        Ok(())
    }

    fn current_feels_like(&mut self, feels_like: f32) -> Result<()> {
        let font = FontRenderer::new::<fonts::u8g2_font_profont22_tf>();

        font.render_aligned(
            format_args!("Feels Like {}°", feels_like.round() as i32),
            self.rect.feels_like.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(Black),
            &mut self.display.color_converted(),
        ).unwrap();

        Ok(())
    }

    fn current_temperature(&mut self, temp: f32) -> Result<()> {
        let large_font = FontRenderer::new::<fonts::u8g2_font_logisoso92_tn>();

        large_font.render_aligned(
            format_args!("{}", temp.round() as i32),
            self.rect.current_temp.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(Black),
            &mut self.display.color_converted(),
        ).unwrap();

        Ok(())
    }

    fn date_and_location(&mut self, current_time: u64, location_name: &str) -> Result<()> {
        let large = FontRenderer::new::<fonts::u8g2_font_profont29_tf>();
        let font = FontRenderer::new::<fonts::u8g2_font_profont22_tf>();

        let offset_dt = time::OffsetDateTime::from_unix_timestamp(current_time as i64)?;
        let format = time::format_description::parse("[weekday], [day] [month repr:long] [year]")?;
        let formatted = offset_dt.format(&format)?;

        large.render_aligned(
            location_name,
            self.rect.date_location.anchor_point(AnchorPoint::TopRight),
            VerticalPosition::Top,
            HorizontalAlignment::Right,
            FontColor::Transparent(Black),
            &mut self.display.color_converted(),
        ).unwrap();

        font.render_aligned(
            formatted.as_str(),
            self.rect.date_location.anchor_point(AnchorPoint::TopRight) + Point::new(0, 29),
            VerticalPosition::Top,
            HorizontalAlignment::Right,
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

    fn debug_draw_rect(&mut self) -> Result<()> {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Black)
            .stroke_width(1)
            .build();
        self.rect.viewport.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.current_weather.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.weather_icon.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.current_temp.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.feels_like.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.current_temp_unit.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.date_location.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.forecast.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        Ok(())
    }

    #[inline]
    pub fn new_buffer() -> Vec<u8> {
        vec![DEFAULT_COLOR.get_byte_value(); SCREEN_BUFFER_SIZE]
    }
}



use std::{cmp, thread};
use std::time::Duration;
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
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::{MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::StyledDrawable;
use u8g2_fonts::{
    FontRenderer,
    fonts,
    types::{HorizontalAlignment, VerticalPosition, FontColor},
};
use esp_idf_hal::
{
    delay,
    gpio,
    gpio::{PinDriver},
    peripheral,
    prelude::*,
    spi,
    spi::{Dma, SpiDriverConfig, SpiConfig}
};
use itertools::Itertools;

use crate::chart::axis::{Axis, Placement, Scale};
use crate::chart::bar::{BarChart};
use crate::chart::line::LineChart;

use crate::config::CONFIG;
use crate::display::{DisplayManager, DisplayManagerPins, DisplayRect};
use crate::icons::WeatherIconSet;
use crate::owm::icons::{get_icon_for_current_weather, get_icon_for_daily_forecast};
use crate::owm::model::{CurrentWeather, DailyForecast, HourlyForecast, WeatherData};

const SCREEN_BUFFER_SIZE: usize =  WIDTH as usize / 8 * HEIGHT as usize;
const DEFAULT_COLOR: Color = White;

const MARGIN: u32 = 8;

// Seems like icons have a bunch of padding on the horizontal axis
// This is a dirty attempt to gain some screen space
// Padding is actually closer to 32, but this is enough to make things fit
const IMG_ICON_PADDING: u32 = 16;

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
            &SpiConfig::new().baudrate(8.MHz().into()),
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
        let current_icon_size = Size::new(196 - IMG_ICON_PADDING, 196);
        let current_temp_size = Size::new(196, current_icon_size.height);

        let forecast_separator_size = Size::new(IMG_ICON_PADDING, current_icon_size.height);

        let current_weather_size = Size::new(current_temp_size.width + current_icon_size.width + forecast_separator_size.width, current_icon_size.height);

        let temp_unit_size = Size::new(32, current_temp_size.height);
        let temp_feels_like_size = Size::new(current_temp_size.width, 32);

        let date_location_size = Size::new(viewport_size.width - current_weather_size.width, 64);
        let forecast_size = Size::new(viewport_size.width - current_weather_size.width, current_weather_size.height - date_location_size.height);

        let metrics_size = Size::new(294, viewport_size.height - current_weather_size.height);
        let chart_size = Size::new(viewport_size.width - metrics_size.width, viewport_size.height - current_weather_size.height);

        let viewport = Rectangle::new(Point::new(MARGIN as i32, MARGIN as i32), viewport_size);
        let current_weather = Rectangle::new(viewport.top_left, current_weather_size);
        let weather_icon = Rectangle::new(current_weather.top_left, current_icon_size);

        let current_temp = Rectangle::new(weather_icon.anchor_point(AnchorPoint::TopRight), current_temp_size);
        let forecast_separator = Rectangle::new(current_temp.anchor_point(AnchorPoint::TopRight), forecast_separator_size);

        let current_temp_unit = Rectangle::new(current_temp.anchor_point(AnchorPoint::TopRight), temp_unit_size)
            .translate(Point::new(-(temp_unit_size.width as i32), 0));

        let feels_like = Rectangle::new(current_temp.anchor_point(AnchorPoint::BottomLeft), temp_feels_like_size)
            .translate(Point::new(0, -(temp_feels_like_size.height as i32)));

        let date_location = Rectangle::new(forecast_separator.anchor_point(AnchorPoint::TopRight), date_location_size);

        let forecast = Rectangle::new(date_location.anchor_point(AnchorPoint::BottomLeft), forecast_size);
        let forecasts = [0; 5].iter().enumerate().map(|(i, _)| {
            let size = Size::new(forecast_size.width / 5, forecast_size.height);
            let offset_x = size.width * i as u32;
           Rectangle::new(forecast.anchor_point(AnchorPoint::TopLeft) + Point::new(offset_x as i32, 0), size)
        }).collect::<Vec<_>>();

        let metrics = Rectangle::new(current_weather.anchor_point(AnchorPoint::BottomLeft), metrics_size);
        let chart = Rectangle::new(metrics.anchor_point(AnchorPoint::TopRight), chart_size);

        let rect = DisplayRect {
            viewport,
            current_weather,
            weather_icon,
            current_temp,
            feels_like,
            current_temp_unit,
            date_location,
            forecast,
            forecasts,
            metrics,
            chart,
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
        let daily = data.daily.unwrap();
        let hourly = data.hourly.unwrap();
        let dt = current.dt;

        let large_icon_set = WeatherIconSet::new()?;
        let small_icon_set = WeatherIconSet::new_small()?;

        self.current_weather_icon(&large_icon_set, &current)?;
        self.current_temperature(&current)?;
        self.current_feels_like(&current)?;
        self.current_temp_unit()?;
        self.date_and_location(dt, location_name)?;
        self.daily_forecast(&small_icon_set, &daily)?;

        self.debug_draw_rect()?;

        thread::sleep(Duration::from_millis(1000));
        self.chart(data.timezone_offset, &hourly)?;

        self.update_frame()?;
        self.display_frame()?;

        Ok(())
    }

    fn current_weather_icon(&mut self, icons: &WeatherIconSet, current: &CurrentWeather) -> Result<()> {
        let icon = get_icon_for_current_weather(icons, current);
        Image::new(icon, self.rect.current_weather
            .translate(Point::new(-(IMG_ICON_PADDING as i32) / 2, 0)).top_left)
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
        let offset = Point::new(0, 46); // 46 is half the font size
        Circle::new(
            self.rect.current_temp_unit.center() - offset - circle_center,
            circle_diameter
        ).into_styled(unit_style)
            .draw(&mut self.display.color_converted())?;

        Ok(())
    }

    fn current_feels_like(&mut self, current: &CurrentWeather) -> Result<()> {
        let font = FontRenderer::new::<fonts::u8g2_font_profont22_tf>();

        font.render_aligned(
            format_args!("Feels Like {}°", current.feels_like.round() as i32),
            self.rect.feels_like.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(Black),
            &mut self.display.color_converted(),
        ).unwrap();

        Ok(())
    }

    fn current_temperature(&mut self, current: &CurrentWeather) -> Result<()> {
        let large_font = FontRenderer::new::<fonts::u8g2_font_logisoso92_tn>();

        large_font.render_aligned(
            format_args!("{}", current.temp.round() as i32),
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

    fn daily_forecast(&mut self, icons: &WeatherIconSet, forecast: &Vec<DailyForecast>) -> Result<()> {
        for (index, rec) in self.rect.forecasts.iter().enumerate() {
            let daily = &forecast[index];
            let icon = get_icon_for_daily_forecast(icons, daily);
            let img_center_offset = Point::new((icons.WIDTH / 2) as i32, (icons.HEIGHT / 2) as i32);

            let _ = Image::new(icon, rec.bounding_box().center() - img_center_offset)
                .draw(&mut self.display.color_converted())?;

            let txt_offset = Point::new(0, (icons.HEIGHT / 2 + MARGIN) as i32);

            // Draw day of week
            let font = FontRenderer::new::<fonts::u8g2_font_profont22_tf>();
            let font_small = FontRenderer::new::<fonts::u8g2_font_profont17_tf>();
            let offset_dt = time::OffsetDateTime::from_unix_timestamp(daily.dt as i64)?;
            let format = time::format_description::parse("[weekday repr:short]")?;
            let day_formatted = offset_dt.format(&format)?;


            font.render_aligned(
                day_formatted.as_str(),
                rec.bounding_box().center() - txt_offset,
                VerticalPosition::Bottom,
                HorizontalAlignment::Center,
                FontColor::Transparent(Black),
                &mut self.display.color_converted(),
            ).unwrap();

            font_small.render_aligned(
                format_args!("{}°|{}°", daily.temp.min.round(), daily.temp.max.round()),
                rec.bounding_box().center() + txt_offset,
                VerticalPosition::Top,
                HorizontalAlignment::Center,
                FontColor::Transparent(Black),
                &mut self.display.color_converted(),
            ).unwrap();


        }
        Ok(())
    }

    fn chart(&mut self, timezone_offset: i64, forecast: &Vec<HourlyForecast>) -> Result<()> {
        let app_config = CONFIG;

        let temp = forecast.iter().map(|hourly| {
           Point { x: (hourly.dt) as i32, y: hourly.temp.round() as i32 }
        })
            .take(app_config.hours_to_draw)
            .collect::<Vec<_>>();

        // From 0 to 100%
        let precip = forecast.iter().map(|hourly| {
            Point { x: (hourly.dt) as i32, y: hourly.pop as i32 * 100 }
        })
            .take(app_config.hours_to_draw)
            .collect::<Vec<_>>();


        let (x_min, x_max) = temp.iter().map(|p| p.x).minmax().into_option().unwrap();
        let (temp_min, temp_max) = temp.iter().map(|p| p.y).minmax().into_option().unwrap();

        // Try to have a 15° range
        let temp_range = {
            let new_min = cmp::min(temp_min, temp_max - 15);
            let new_max = cmp::max(temp_max,temp_min + 15);

            new_min..new_max
        };

        // Layout rectangles
        let margin: u32 = 4;
        let axis_rec_dim = 12;
        let curve_size = Size::new(
            self.rect.chart.size.width - 2 * margin - 2 * axis_rec_dim,
            self.rect.chart.size.height - 2 * margin -  axis_rec_dim
        );

        let left_axis_rec = Rectangle::new(
            self.rect.chart.top_left + Point::new(margin as i32, margin as i32),
            Size::new(axis_rec_dim, curve_size.height)
        );


        let curve_rec = Rectangle::new(
            left_axis_rec.anchor_point(AnchorPoint::TopRight),
            Size::new(curve_size.width, curve_size.height)
        );

        let bottom_axis_rec = Rectangle::new(
            curve_rec.anchor_point(AnchorPoint::BottomLeft),
            Size::new(curve_size.width, axis_rec_dim)
        );

        let right_axis_rec = Rectangle::new(
            curve_rec.anchor_point(AnchorPoint::TopRight),
            Size::new(axis_rec_dim, curve_size.height)
        );

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Black)
            .stroke_width(1)
            .build();

        // curve_rec.into_styled(style).draw(&mut self.display.color_converted())?;
        // left_axis_rec.into_styled(style).draw(&mut self.display.color_converted())?;
        // right_axis_rec.into_styled(style).draw(&mut self.display.color_converted())?;
        // curve_rec.into_styled(style).draw(&mut self.display.color_converted())?;
        // bottom_axis_rec.into_styled(style).draw(&mut self.display.color_converted())?;

        // Temperature
        LineChart::new(temp.as_slice(), None, Some(temp_range.clone()))
            .into_drawable_curve(
                &curve_rec.top_left,
                &curve_rec.anchor_point(AnchorPoint::BottomRight)
            ).set_color(BinaryColor::Off)
            .set_thickness(2)
            .draw(&mut self.display.color_converted())?;

        // Precipitation
        BarChart::new(precip.as_slice(), None, Some(0..100))
            .into_drawable_curve(
                &curve_rec.top_left,
                &curve_rec.anchor_point(AnchorPoint::BottomRight)
            ).set_color(BinaryColor::Off)
            .set_thickness(1)
            .set_fill(true)
            .draw(&mut self.display.color_converted())?;

        // X Axis
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_8X13)
            .text_color(BinaryColor::Off)
            .build();

        Axis::new(x_min..x_max)
            .set_scale(Scale::Fixed(3600*2))
            .into_drawable_axis(
                Placement::X {
                    x1: bottom_axis_rec.anchor_point(AnchorPoint::TopLeft).x,
                    x2: bottom_axis_rec.anchor_point(AnchorPoint::TopRight).x,
                    y: bottom_axis_rec.bounding_box().center().y,
                }
            )
            .set_text_style(text_style)
            .set_color(BinaryColor::Off)
            .set_text_render(&|x| {
                let offset_dt = time::OffsetDateTime::from_unix_timestamp(x as i64 + timezone_offset).unwrap();
                let format = time::format_description::parse("[hour repr:24]").unwrap();
                let day_formatted = offset_dt.format(&format).unwrap();

                day_formatted
            }).draw(&mut self.display.color_converted())?;

        // Y Axis (Temperature)
        Axis::new(temp_range.clone())
            .set_scale(Scale::RangeFraction(5))
            .into_drawable_axis(Placement::Y {
                y1: left_axis_rec.anchor_point(AnchorPoint::TopRight).y,
                y2: left_axis_rec.anchor_point(AnchorPoint::BottomRight).y,
                x: left_axis_rec.bounding_box().center().x,
            })
            .set_text_style(text_style)
            .set_thickness(0)
            .set_color(BinaryColor::Off)
            .draw(&mut self.display.color_converted())?;

        // Y Axis Precipitation
        Axis::new(0..101)
            .set_scale(Scale::RangeFraction(5))
            .into_drawable_axis(Placement::Y {
                y1: right_axis_rec.anchor_point(AnchorPoint::TopRight).y,
                y2: right_axis_rec.anchor_point(AnchorPoint::BottomRight).y,
                x: right_axis_rec.bounding_box().center().x,
            })
            .set_text_style(text_style)
            .set_thickness(0)
            .set_color(BinaryColor::Off)
            .draw(&mut self.display.color_converted())?;

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
        // self.rect.viewport.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.current_weather.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.weather_icon.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.current_temp.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.feels_like.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.current_temp_unit.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.date_location.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        // self.rect.forecast.into_styled(style)
        //     .draw(&mut self.display.color_converted())?;
        //
        // for rec in self.rect.forecasts.iter() {
        //     rec.into_styled(style)
        //         .draw(&mut self.display.color_converted())?;
        // }

        self.rect.metrics.into_styled(style)
            .draw(&mut self.display.color_converted())?;
        self.rect.chart.into_styled(style)
            .draw(&mut self.display.color_converted())?;

        Ok(())
    }

    #[inline]
    pub fn new_buffer() -> Vec<u8> {
        vec![DEFAULT_COLOR.get_byte_value(); SCREEN_BUFFER_SIZE]
    }
}



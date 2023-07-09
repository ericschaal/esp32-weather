use crate::http::get;
use crate::config::CONFIG;
use anyhow::{Result};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherData {
    lat: f32,
    lon: f32,
    timezone: String,
    timezone_offset: f32,
    current: Current,
    minutely: Vec<Minutely>,
    hourly: Vec<Hourly>,
    daily: Vec<Daily>,
    alerts: Option<Vec<Alerts>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Current {
    dt: f32,
    sunrise: f32,
    sunset: f32,
    temp: f32,
    feels_like: f32,
    pressure: f32,
    humidity: f32,
    dew_point: f32,
    uvi: f32,
    clouds: f32,
    visibility: f32,
    wind_speed: f32,
    wind_deg: f32,
    wind_gust: Option<f32>,
    weather: Vec<Weather>,
    rain: Option<Rain>,
    snow: Option<Snow>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rain {
    #[serde(rename = "1h")]
    one_hour: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Snow {
    #[serde(rename = "1h")]
    one_hour: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    id: f32,
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Minutely {
    dt: f32,
    precipitation: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Hourly {
    dt: f32,
    temp: f32,
    feels_like: f32,
    pressure: f32,
    humidity: f32,
    dew_point: f32,
    uvi: f32,
    clouds: f32,
    visibility: f32,
    wind_speed: f32,
    wind_deg: f32,
    wind_gust: Option<f32>,
    weather: Vec<Weather>,
    pop: f32,
    rain: Option<Rain>,
    snow: Option<Snow>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Daily {
    dt: f32,
    sunrise: f32,
    sunset: f32,
    moonrise: f32,
    moonset: f32,
    moon_phase: f32,
    summary: Option<String>,
    temp: Temp,
    feels_like: FeelsLike,
    pressure: f32,
    humidity: f32,
    dew_point: f32,
    wind_speed: f32,
    wind_deg: f32,
    wind_gust: Option<f32>,
    weather: Vec<Weather>,
    clouds: f32,
    pop: f32,
    rain: Option<f32>,
    snow: Option<f32>,
    uvi: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Temp {
    day: f32,
    min: f32,
    max: f32,
    night: f32,
    eve: f32,
    morn: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct FeelsLike {
    day: f32,
    night: f32,
    eve: f32,
    morn: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Alerts {
    sender_name: String,
    event: String,
    start: f32,
    end: f32,
    description: String,
    tags: Vec<String>,
}
pub fn fetch_owm_report(
    lat: f32,
    lon: f32
) -> Result<WeatherData> {
    let url = format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}", lat, lon, CONFIG.owm_api_key);
    let result = get(url)?;
    let data: WeatherData = serde_json::from_str(&result)?;
    Ok(data)
}
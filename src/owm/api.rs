use crate::config::CONFIG;
use anyhow::{Result};
use crate::http_client::get;
use crate::owm::model::WeatherData;


pub fn fetch_owm_report(
    lat: f32,
    lon: f32
) -> Result<WeatherData> {
    let url = format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&exclude=minutely&units=metric", lat, lon, CONFIG.owm_api_key);
    let result = get(url)?;
    let data: WeatherData = serde_json::from_str(&result)?;
    Ok(data)
}
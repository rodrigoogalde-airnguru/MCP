use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Parámetros para obtener pronóstico
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ForecastRequest {
    pub latitude: f64,
    pub longitude: f64,
}

/// Parámetros para obtener alertas por estado
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AlertRequest {
    pub state: String,
}

/// Respuesta de alertas de la API NWS
#[derive(Debug, Deserialize, Serialize)]
pub struct AlertsResponse {
    pub features: Vec<AlertFeature>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertFeature {
    pub properties: AlertProperties,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertProperties {
    pub event: String,
    #[serde(rename = "areaDesc")]
    pub area_desc: String,
    pub severity: String,
    pub description: String,
    pub instruction: Option<String>,
}

/// Respuesta de puntos (resuelve lat/lon a forecast URL)
#[derive(Debug, Deserialize, Serialize)]
pub struct PointsResponse {
    pub properties: PointsProperties,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PointsProperties {
    pub forecast: String,
}

/// Respuesta de pronóstico
#[derive(Debug, Deserialize, Serialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForecastProperties {
    pub periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForecastPeriod {
    pub name: String,
    pub temperature: i32,
    #[serde(rename = "temperatureUnit")]
    temperature_unit: String,
    #[serde(rename = "windSpeed")]
    pub wind_speed: String,
    #[serde(rename = "windDirection")]
    pub wind_direction: String,
    #[serde(rename = "shortForecast")]
    pub short_forecast: String,
    #[serde(rename = "detailedForecast")]
    detailed_forecast: String,
}
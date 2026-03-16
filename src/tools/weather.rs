use anyhow::Result;
use crate::utils::http::HttpClient;
use crate::models::weather::ForecastResponse;

/// Obtiene el pronóstico del clima para una ubicación
pub async fn get_forecast(lat: f64, lon: f64) -> Result<ForecastResponse> {
    let url = format!(
        "{}/points/{},{}/forecast",
        crate::utils::NWS_API_BASE,
        lat,
        lon
    );
    HttpClient::get_nws(&url).await
}

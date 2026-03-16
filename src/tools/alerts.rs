use anyhow::Result;
use crate::utils::http::HttpClient;

use crate::models::weather::AlertsResponse;

/// Obtiene alertas de clima activas
pub async fn get_alerts() -> Result<AlertsResponse> {
    let url = format!("{}/alerts/active", crate::utils::NWS_API_BASE);
    HttpClient::get_nws(&url).await
}

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, tool_handler, tool_router};
use rmcp::service::ServiceExt;
use rmcp::transport::stdio;

use crate::models::weather::{AlertRequest, ForecastRequest};
use crate::utils::formatters::{format_alert, format_period};
use crate::utils::NWS_API_BASE;

/// Servidor MCP que expone tools de clima a Claude
#[derive(Clone)]
pub struct Weather {
    tool_router: ToolRouter<Weather>,
}

impl Weather {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Inicia el servidor MCP
    pub async fn run(self) -> anyhow::Result<()> {
        self.serve(stdio()).await?.waiting().await?;
        Ok(())
    }
}

#[tool_router]
impl Weather {
    /// Obtiene alertas de clima activas para un estado de EE.UU.
    #[rmcp::tool(
        description = "Get active weather alerts for a US state. State should be a 2-letter state code (e.g., TX, CA, NY)."
    )]
    async fn get_alerts(
        &self,
        Parameters(params): Parameters<AlertRequest>,
    ) -> String {
        let _url = format!(
            "{}/alerts/active/area/{}",
            NWS_API_BASE,
            params.state.to_uppercase()
        );

        match crate::tools::alerts::get_alerts().await {
            Ok(data) => {
                if data.features.is_empty() {
                    format!("No active weather alerts for {}.", params.state)
                } else {
                    data.features
                        .iter()
                        .map(format_alert)
                        .collect::<Vec<_>>()
                        .join("\n---\n")
                }
            }
            Err(_) => format!("Unable to fetch weather alerts for {}.", params.state),
        }
    }

    /// Obtiene pronóstico de clima para una ubicación específica
    #[rmcp::tool(
        description = "Get weather forecast for a specific latitude and longitude. Returns the next 5 forecast periods."
    )]
    async fn get_forecast(
        &self,
        Parameters(params): Parameters<ForecastRequest>,
    ) -> String {
        match crate::tools::weather::get_forecast(params.latitude, params.longitude).await {
            Ok(forecast_data) => {
                let periods = &forecast_data.properties.periods;
                if periods.is_empty() {
                    "No forecast data available.".to_string()
                } else {
                    periods
                        .iter()
                        .take(5)
                        .map(format_period)
                        .collect::<Vec<String>>()
                        .join("\n---\n")
                }
            }
            Err(_) => "Unable to fetch forecast data for this location.".to_string(),
        }
    }
}

#[tool_handler]
impl ServerHandler for Weather {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A weather MCP server that provides weather alerts and forecasts for US locations using the National Weather Service API.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

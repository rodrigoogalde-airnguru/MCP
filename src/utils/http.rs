use anyhow::Result;
use serde::de::DeserializeOwned;
use reqwest::header;

/// Configuración de headers para solicitudes HTTP
#[derive(Debug, Clone, Default)]
pub struct RequestConfig {
    /// User-Agent header. Si es None, usa el default
    pub user_agent: Option<String>,
    /// Accept header. Si es None, usa "application/json"
    pub accept: Option<String>,
    /// Headers adicionales personalizados
    pub custom_headers: Vec<(String, String)>,
    /// Si true, retorna error en status 4xx/5xx
    pub check_status: bool,
}

impl RequestConfig {
    /// Crea configuración con defaults
    pub fn new() -> Self {
        Self {
            user_agent: None,
            accept: None,
            custom_headers: Vec::new(),
            check_status: false,
        }
    }

    /// Configuración para API NWS (GeoJSON)
    pub fn nws() -> Self {
        Self {
            user_agent: Some(crate::utils::USER_AGENT.to_string()),
            accept: Some("application/geo+json".to_string()),
            custom_headers: Vec::new(),
            check_status: true,
        }
    }

    /// Configuración estándar (default)
    pub fn standard() -> Self {
        Self {
            user_agent: Some(crate::utils::USER_AGENT.to_string()),
            accept: Some("application/json".to_string()),
            custom_headers: Vec::new(),
            check_status: false,
        }
    }

    /// Agregar header personalizado
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.custom_headers.push((key, value));
        self
    }

    /// Habilitar/deshabilitar validación de status
    pub fn with_status_check(mut self, check: bool) -> Self {
        self.check_status = check;
        self
    }
}

/// Cliente HTTP para llamadas a APIs externas
pub struct HttpClient;

impl HttpClient {
    /// Realiza una solicitud GET con configuración
    /// Si config es None, usa configuración estándar
    pub async fn get<T: DeserializeOwned>(
        url: &str,
        config: Option<RequestConfig>,
    ) -> Result<T> {
        let config = config.unwrap_or_else(RequestConfig::standard);
        let client = reqwest::Client::new();

        let mut request = client.get(url);

        // Agregar User-Agent
        if let Some(ua) = &config.user_agent {
            request = request.header(header::USER_AGENT, ua);
        }

        // Agregar Accept
        if let Some(accept) = &config.accept {
            request = request.header(header::ACCEPT, accept);
        }

        // Agregar headers personalizados
        for (key, value) in &config.custom_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        let mut response = request.send().await?;

        // Validar status si está habilitado
        if config.check_status {
            response = response.error_for_status()?;
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    /// Alias para solicitud GET estándar sin configuración
    pub async fn get_simple<T: DeserializeOwned>(url: &str) -> Result<T> {
        Self::get(url, None).await
    }

    /// Alias para solicitud GET con configuración NWS
    pub async fn get_nws<T: DeserializeOwned>(url: &str) -> Result<T> {
        Self::get(url, Some(RequestConfig::nws())).await
    }
}

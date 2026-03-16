# Weather MCP Server

Un servidor **Model Context Protocol (MCP)** en Rust que expone herramientas de clima a Claude. Este proyecto es una guía educativa para aprender Rust y MCP.

## 📋 Índice

- [Visión General](#visión-general)
- [Arquitectura](#arquitectura)
- [Estructura de Carpetas](#estructura-de-carpetas)
- [Conceptos Clave](#conceptos-clave)
- [Flujo de Datos](#flujo-de-datos)
- [Cómo Funciona Cada Parte](#cómo-funciona-cada-parte)
- [Definiciones Importantes](#definiciones-importantes)
- [Próximos Pasos](#próximos-pasos)

---

## 🎯 Visión General

Este servidor MCP permite a Claude:

1. **Obtener alertas de clima** — por estado (TX, CA, NY, etc)
2. **Obtener pronósticos** — para una ubicación específica (lat/lon)

Ambas herramientas obtienen datos de la **National Weather Service API (NWS)** y los devuelven formateados en texto.

```
Claude ←→ MCP Protocol ←→ Weather Server ←→ NWS API
         (stdin/stdout)    (Rust)       (https://api.weather.gov)
```

---

## 🏗️ Arquitectura

### Componentes Principales

```
Weather Server (pub struct Weather)
    ├─ get_alerts(state: String) → String
    │   └─ tools::alerts::get_alerts() → AlertsResponse
    │       └─ HttpClient::get_nws() → JSON parsing
    │
    └─ get_forecast(lat: f64, lon: f64) → String
        └─ tools::weather::get_forecast() → ForecastResponse
            └─ HttpClient::get_nws() → JSON parsing
```

### Patrón de Diseño

```
MCP Tool (Claude invoca)
    ↓
Weather::{get_alerts, get_forecast} (Server methods)
    ↓
tools::{alerts, weather} (Helper functions)
    ↓
utils::http::HttpClient (HTTP client configurable)
    ↓
NWS API (datos externo)
    ↓
utils::formatters (convierte a texto legible)
    ↓
Claude (recibe respuesta)
```

---

## 📁 Estructura de Carpetas

```
src/
├── main.rs                 ← Punto de entrada. Arranca el servidor
├── server.rs               ← Struct Weather con #[tool_router] y #[tool]
├── tools/
│   ├── mod.rs             ← Declara: pub mod weather, alerts
│   ├── weather.rs         ← fn get_forecast(lat, lon) helper
│   └── alerts.rs          ← fn get_alerts() helper
├── models/
│   ├── mod.rs             ← Declara: pub mod weather
│   └── weather.rs         ← Structs para deserializar respuestas JSON
├── utils/
│   ├── mod.rs             ← NWS_API_BASE, USER_AGENT constants
│   ├── http.rs            ← HttpClient con RequestConfig flexible
│   └── formatters.rs      ← format_alert(), format_period()
```

### Propósito de Cada Archivo

| Archivo | Responsabilidad |
|---------|-----------------|
| `main.rs` | Inicia el runtime tokio y crea Weather::new().run() |
| `server.rs` | Define las tools MCP que Claude puede invocar |
| `tools/weather.rs` | Lógica para obtener datos de NWS |
| `tools/alerts.rs` | Lógica para obtener alertas de NWS |
| `models/weather.rs` | Structs que mapean respuestas JSON de NWS |
| `utils/http.rs` | Cliente HTTP reutilizable con config flexible |
| `utils/formatters.rs` | Convierte datos estructurados a strings legibles |

---

## 🔑 Conceptos Clave

### 1. **MCP (Model Context Protocol)**

Es un protocolo de comunicación entre Claude y servidores de herramientas.

```
Claude dice: "Necesito una herramienta llamada 'get_alerts'"
              ↓
        MCP server responde: "Tengo eso, espera los parámetros"
              ↓
Claude envía: { "state": "TX" }
              ↓
        Server ejecuta: Weather::get_alerts("TX")
              ↓
Claude recibe: "Event: Severe Thunderstorm...\nSeverity: Severe..."
```

### 2. **Macros en Rust**

Código que genera código automáticamente. En este proyecto usamos:

- `#[tool_router]` — genera enrutamiento automático de tools
- `#[tool]` — marca un método como una tool MCP
- `#[derive(...)]` — genera implementaciones automáticas

```rust
#[tool_router]  // Genera: tool_router(), enrutamiento automático
impl Weather {
    #[tool(description = "...")]  // Claude ve esto como una herramienta
    async fn get_alerts(&self, params: Parameters<AlertRequest>) -> String {
        // Lógica...
    }
}
```

### 3. **pub vs private**

Rust es **privado por defecto**. Debes ser explícito:

```rust
pub struct Weather;        // Visible desde afuera
struct WeatherInternal;    // Solo visible en este módulo

pub fn get_forecast() {}   // Visible
fn internal_helper() {}    // Privado
```

### 4. **async/await**

Código no-bloqueante para operaciones que toman tiempo (HTTP, I/O):

```rust
// Sin async: bloquea el thread
let response = HttpClient::get(url);  // ← espera aquí

// Con async: cede el thread a otro trabajo
let response = HttpClient::get(url).await?;  // ← cede control
```

### 5. **Result<T, E>**

Manejo de errores explícito en Rust:

```rust
fn get_forecast() -> Result<String> {
    let data = HttpClient::get(url).await?;  // ? = propaga error
    Ok(format_period(&data))                 // Ok() envuelve resultado
}
```

El `?` es azúcar sintáctico que dice: "Si hay error, retórnalo. Si es Ok, desenvuelve el valor."

### 6. **Traits**

Interfaces que definen comportamientos. Ejemplos en este proyecto:

- `Deserialize` — permite que serde convierta JSON → struct
- `Serialize` — permite que serde convierta struct → JSON
- `JsonSchema` — permite que schemars genere esquemas JSON

```rust
#[derive(Deserialize, JsonSchema)]  // Implementa ambos traits
pub struct AlertRequest {
    pub state: String,
}
```

---

## 🔄 Flujo de Datos

### Flujo: Get Alerts

```
1. Claude invoca: "Alertas para TX"
                       ↓
2. MCP enruta a: Weather::get_alerts(AlertRequest { state: "TX" })
                       ↓
3. Weather llama: tools::alerts::get_alerts()
                       ↓
4. tools::alerts hace: HttpClient::get_nws("https://api.weather.gov/alerts/active")
                       ↓
5. HttpClient:
   - Crea client de reqwest
   - Agrega headers: User-Agent, Accept: application/geo+json
   - Envía GET request
   - Verifica status HTTP (error_for_status)
   - Parsea JSON → AlertsResponse
                       ↓
6. Weather recibe: AlertsResponse { features: [...] }
                       ↓
7. Weather itera y formatea: format_alert(&feature) → String
                       ↓
8. Claude recibe:
   "Event: Tornado Warning
    Area: Dallas County, TX
    Severity: Extreme
    ..."
```

### Flujo: Get Forecast

```
1. Claude invoca: "Pronóstico para 32.78°N, -96.80°W"
                       ↓
2. MCP enruta a: Weather::get_forecast(ForecastRequest { latitude: 32.78, longitude: -96.80 })
                       ↓
3. Weather llama: tools::weather::get_forecast(32.78, -96.80)
                       ↓
4. tools::weather hace dos requests:

   a) GET /points/32.78,-96.80
      → PointsResponse { forecast: "https://api.weather.gov/gridpoints/FWD/..." }

   b) GET https://api.weather.gov/gridpoints/FWD/...
      → ForecastResponse { periods: [Period, Period, ...] }
                       ↓
5. Weather recibe: ForecastResponse
                       ↓
6. Weather itera (primeros 5 períodos) y formatea:
   format_period(&period) → String
                       ↓
7. Claude recibe:
   "Tonight:
    Temperature: 72°F
    Wind: 10 mph NW
    Forecast: Clear skies...
    ---
    Monday:
    Temperature: 85°F
    ..."
```

---

## 🧩 Cómo Funciona Cada Parte

### main.rs — Punto de Entrada

```rust
#[tokio::main]  // Arranca el runtime async de tokio
async fn main() -> Result<()> {
    let server = Weather::new();
    server.run().await?;
    Ok(())
}
```

**Qué pasa:**
1. `#[tokio::main]` transforma `main()` en código que inicia el runtime
2. Se crea una instancia de `Weather`
3. Se llama `run()` que debería escuchar en stdin/stdout
4. El `?` propaga errores hacia arriba

### server.rs — Las Tools

```rust
#[tool_router]
impl Weather {
    pub fn new() -> Self { ... }

    #[tool(description = "Get active weather alerts...")]
    async fn get_alerts(&self, Parameters(params): Parameters<AlertRequest>) -> String {
        // 1. Llamar al helper
        let data = crate::tools::alerts::get_alerts().await?;

        // 2. Formatear respuesta
        data.features
            .iter()
            .map(format_alert)
            .collect::<Vec<_>>()
            .join("\n---\n")
    }
}
```

**Desglose:**
- `#[tool_router]` — genera enrutamiento de tools
- `#[tool(...)]` — Claude ve esto como una herramienta disponible
- `Parameters(params)` — desempaqueta los argumentos JSON
- `await?` — espera respuesta HTTP y propaga errores

### utils/http.rs — Cliente HTTP Configurable

```rust
pub struct RequestConfig {
    pub user_agent: Option<String>,
    pub accept: Option<String>,
    pub custom_headers: Vec<(String, String)>,
    pub check_status: bool,
}

impl HttpClient {
    pub async fn get<T: DeserializeOwned>(
        url: &str,
        config: Option<RequestConfig>,
    ) -> Result<T> {
        let config = config.unwrap_or_else(RequestConfig::standard);
        // 1. Crear cliente
        // 2. Agregar headers
        // 3. Enviar request
        // 4. Parsear JSON
    }

    pub async fn get_nws<T: DeserializeOwned>(url: &str) -> Result<T> {
        Self::get(url, Some(RequestConfig::nws())).await
    }
}
```

**Ventajas:**
- ✅ Flexible: puedes usar `get(url, None)` o `get(url, Some(config))`
- ✅ Reutilizable: un solo lugar para toda la lógica HTTP
- ✅ Configurable: presets para NWS, JSON estándar, o personalizado

### models/weather.rs — Deserialización JSON

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct AlertsResponse {
    pub features: Vec<AlertFeature>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AlertRequest {
    pub state: String,
}
```

**Cómo funciona:**
- `Deserialize` — serde convierte JSON → struct
- `Serialize` — serde convierte struct → JSON
- `JsonSchema` — schemars genera el schema para Claude
- `Debug` — permite `println!("{:?}", value)`

---

## 📚 Definiciones Importantes

### pub (Público)

Indica que algo es accesible desde otros módulos.

```rust
pub struct Weather;           // Otros módulos pueden usar esto
struct InternalData;          // Solo visible dentro de este módulo

pub fn public_method() {}      // Accessible desde otros módulos
fn private_method() {}         // Solo visible aquí
```

### impl (Implementación)

Bloque donde defines métodos para una struct.

```rust
struct Weather;

impl Weather {
    pub fn new() -> Self { Weather }
    pub fn run(&self) { ... }
}

// Uso:
let server = Weather::new();
server.run();
```

### async/await

Permite ejecutar código sin bloquear el thread.

```rust
async fn get_data() -> Result<String> {
    let response = HttpClient::get(url).await?;
    Ok(response)
}

// Uso:
let data = get_data().await?;  // Espera el resultado
```

### Result<T, E>

Tipo que representa éxito (Ok) o error (Err).

```rust
fn might_fail() -> Result<String> {
    Ok("success".to_string())      // Envuelve en Ok
    // o
    Err(anyhow::anyhow!("failed"))  // Envuelve en Err
}

// Uso:
match might_fail() {
    Ok(val) => println!("{}", val),
    Err(e) => println!("Error: {}", e),
}

// O con ?:
let result = might_fail()?;  // Propaga error si falla
```

### Macro (#[...])

Código que genera código en tiempo de compilación.

```rust
#[derive(Debug)]               // Genera implementación de Debug
pub struct MyStruct;

#[tool_router]                 // Genera enrutamiento de tools
impl Weather { ... }

#[tokio::main]                 // Genera runtime inicialización
async fn main() { ... }
```

### Option<T>

Valor que puede ser `Some(valor)` o `None`.

```rust
let maybe_value: Option<String> = Some("hello");
let nothing: Option<String> = None;

if let Some(value) = maybe_value {
    println!("{}", value);
}

// O con unwrap_or:
let value = maybe_value.unwrap_or("default".to_string());
```

### serde

Librería para serialización/deserialización de datos.

```rust
#[derive(Deserialize)]
pub struct AlertRequest {
    pub state: String,
}

// serde convierte: { "state": "TX" } → AlertRequest { state: "TX" }
```

### schemars

Genera JSON Schema desde structs Rust.

```rust
#[derive(JsonSchema)]
pub struct ForecastRequest {
    pub latitude: f64,
    pub longitude: f64,
}

// schemars genera:
// {
//   "type": "object",
//   "properties": {
//     "latitude": { "type": "number" },
//     "longitude": { "type": "number" }
//   }
// }
```

---

## ⚙️ Detalles Técnicos

### ¿Por qué async?

Las operaciones HTTP son **lentas** (network latency). Sin async:

```rust
// ❌ Sin async: bloquea el thread
let forecast = HttpClient::get(url);  // Espera 200ms aquí
let alerts = HttpClient::get(url2);   // No comienza hasta que termine forecast
// Total: 400ms
```

Con async:

```rust
// ✅ Con async: cede el thread
let forecast = HttpClient::get(url).await;    // Comienza
let alerts = HttpClient::get(url2).await;     // Comienza en paralelo
// Total: ~200ms (ambas simultáneas)
```

### ¿Por qué modules/carpetas?

Mantener el código organizado y modular:

```
tools/ - contiene la lógica de negocio (get_forecast, get_alerts)
models/ - contiene estructuras de datos
utils/ - contiene utilidades reutilizables
```

Si todo estuviera en un archivo, sería caótico.

### Headers HTTP en NWS

NWS requiere:
- `User-Agent` — identifica tu app
- `Accept: application/geo+json` — formato que devuelve

Sin estos headers, la API puede rechazar la solicitud.

---

## 🚀 Próximos Pasos

### 1. Implementar run() Completamente

Actualmente `run()` solo imprime un mensaje. Necesita:

```rust
pub async fn run(&self) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Escuchar JSON en stdin
    // Enrutar a tool_router
    // Escribir JSON en stdout
}
```

Esto requiere entender cómo rmcp espera que funcione el loop MCP.

### 2. Testing

Agregar tests para cada función:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_format_alert() {
        let alert = AlertFeature { ... };
        let formatted = format_alert(&alert);
        assert!(formatted.contains("Event:"));
    }
}
```

### 3. Logging

Usar `tracing` (ya en Cargo.toml) para debugging:

```rust
use tracing::{info, error};

info!("Fetching forecast for {}, {}", lat, lon);
error!("Failed to parse response: {}", err);
```

### 4. Error Handling

Mejorar los `Err(_)` genéricos:

```rust
// ❌ Malo: pierde información
Err(_) => "Unable to fetch alerts",

// ✅ Mejor: logs el error real
Err(e) => {
    error!("Alert fetch failed: {}", e);
    "Unable to fetch alerts"
}
```

### 5. Documentación

Agregar doc comments:

```rust
/// Obtiene el pronóstico para una ubicación.
///
/// # Arguments
/// * `lat` - Latitud (-90 a 90)
/// * `lon` - Longitud (-180 a 180)
///
/// # Returns
/// Pronóstico formateado como string
pub async fn get_forecast(lat: f64, lon: f64) -> Result<String> { ... }
```

---

## 🔗 Referencias

- [MCP Documentation](https://modelcontextprotocol.io/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [async Rust](https://doc.rust-lang.org/nightly/async-book/)
- [NWS API](https://www.weather.gov/documentation/services-web-api)
- [rmcp Crate](https://crates.io/crates/rmcp)
- [serde](https://serde.rs/)
- [schemars](https://docs.rs/schemars/)

---

## 💡 Tips para Aprender

### 1. Lee el código en orden

1. `main.rs` — entiende cómo arranca
2. `server.rs` — entiende las tools
3. `tools/*.rs` — entiende la lógica
4. `models/*.rs` — entiende las estructuras
5. `utils/*.rs` — entiende los helpers

### 2. Compila frecuentemente

```bash
cargo check          # Rápido, sin generar binario
cargo build          # Compila a binary
cargo run            # Compila y ejecuta
```

### 3. Lee los errores

Los errores de Rust son detallados y útiles. Léelos completamente.

### 4. Experimenta

Cambia cosas, ve qué rompe, arréglalo. Es la mejor forma de aprender.

---

## 📝 Notas Finales

- Este proyecto es un **esqueleto educativo**, no producción-ready
- Falta implementar la comunicación MCP real (stdin/stdout loop)
- Los conceptos mostrados aquí se aplican a cualquier servidor MCP en Rust
- Rust tiene una curva de aprendizaje pronunciada, pero una vez entiendes el ownership, todo hace sentido

¡Feliz aprendizaje! 🚀

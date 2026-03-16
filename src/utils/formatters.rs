use crate::models::weather::{AlertFeature, ForecastPeriod};

/// Formatea una alerta de clima para mostrar de forma legible
pub fn format_alert(feature: &AlertFeature) -> String {
    let props = &feature.properties;
    format!(
        "Event: {}\nArea: {}\nSeverity: {}\nDescription: {}\nInstructions: {}",
        props.event,
        props.area_desc,
        props.severity,
        props.description,
        props
            .instruction
            .as_deref()
            .unwrap_or("No specific instructions provided")
    )
}

/// Formatea un período de pronóstico para mostrar de forma legible
pub fn format_period(period: &ForecastPeriod) -> String {
    format!(
        "{}:\nTemperature: {}°\nWind: {} {}\nForecast: {}",
        period.name,
        period.temperature,
        period.wind_speed,
        period.wind_direction,
        period.short_forecast
    )
}

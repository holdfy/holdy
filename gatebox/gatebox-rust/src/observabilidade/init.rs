// Converted from gateboxgo/observabilidade/init.go
use anyhow::Result;
use tracing::info;

use crate::observabilidade::metrics;

/// InitObservability inicializa métricas Prometheus (equivalente ao OpenTelemetry do Go).
pub fn init_observability(service_name: String, _service_version: String) -> Result<()> {
    metrics::register_metrics(&service_name)?;
    info!(
        "Observabilidade inicializada com sucesso (service={})",
        service_name
    );
    Ok(())
}

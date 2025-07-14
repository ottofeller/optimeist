use crate::environment::LambdaEnvironment;
use lambda_extension::{LambdaTelemetry, LambdaTelemetryRecord};
use serde::Serialize;
use tracing::{error, info};

// API_URL contains the backend URL without a trailing slash
const BASE_API_URL: &str = env!("METRICS_API_URL");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    /// Request identifier
    request_id: String,
    /// Duration in milliseconds
    duration_ms: f64,
    /// Billed duration in milliseconds
    billed_duration_ms: u64,
    /// Memory allocated in megabytes
    #[serde(rename = "memorySizeMB")]
    memory_size_mb: u64,
    /// Maximum memory used for the invoke in megabytes
    #[serde(rename = "maxMemoryUsedMB")]
    max_memory_used_mb: u64,
    /// Init duration in case of a cold start
    #[serde(default = "Option::default")]
    init_duration_ms: Option<f64>,
    #[serde(default = "Option::default", skip_serializing_if = "Option::is_none")]
    restore_duration_ms: Option<f64>,
    /// Timestamp in microseconds when the log was created
    timestamp_us: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestData {
    metrics: Vec<Metrics>,
    meta: LambdaEnvironment,
}

pub async fn telemetry_handler(
    environment: LambdaEnvironment,
    logs: Vec<LambdaTelemetry>,
) -> eyre::Result<()> {
    let client = reqwest::Client::new();

    let mut batch: Vec<Metrics> = vec![];
    info!("Processing {} logs", logs.len());

    for log in logs {
        if let LambdaTelemetryRecord::PlatformReport {
            request_id,
            metrics,
            ..
        } = log.record
        {
            batch.push(Metrics {
                request_id,
                duration_ms: metrics.duration_ms,
                billed_duration_ms: metrics.billed_duration_ms,
                memory_size_mb: metrics.memory_size_mb,
                max_memory_used_mb: metrics.max_memory_used_mb,
                init_duration_ms: metrics.init_duration_ms,
                restore_duration_ms: metrics.restore_duration_ms,
                timestamp_us: chrono::Utc::now().timestamp_micros().to_string(),
            })
        }
    }

    let api_url = format!("{BASE_API_URL}/collect");
    info!("Sending metrics ({}) to {}", batch.len(), api_url);

    // TODO Split the batch into 50-100 records per chunk
    let result = client
        .post(api_url)
        .timeout(std::time::Duration::from_secs(10))
        .header(
            "Authorization",
            format!("Bearer {}", environment.access_token),
        )
        .json(&RequestData {
            metrics: batch,
            meta: environment.clone(),
        })
        .send()
        .await;

    match result {
        Ok(_) => info!("Metrics sent successfully"),
        Err(e) => error!("Failed to send metrics: {}", e),
    }

    Ok(())
}

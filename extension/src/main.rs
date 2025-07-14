mod environment;
mod events;
mod telemetry;

use crate::environment::LambdaEnvironment;
use crate::events::{events_handler, Updater};
use crate::telemetry::telemetry_handler;
use aws_config::{BehaviorVersion, Region};
use eyre::Result;
use lambda_extension::{service_fn, Error, Extension, LambdaEvent, SharedService};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install CryptoProvider");

    let region = env::var("AWS_REGION").unwrap_or("us-east-1".to_string());

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region))
        .load()
        .await;

    let telemetry_environment = LambdaEnvironment::new(&config).await?;
    let events_environment = telemetry_environment.clone();

    let telemetry_processor = SharedService::new(service_fn(move |logs| {
        telemetry_handler(telemetry_environment.clone(), logs)
    }));

    // Create a shared state to run the updater task and manage its shutdown
    let updater = Updater::new(&config, events_environment);

    let events_processor =
        service_fn(move |event: LambdaEvent| events_handler(updater.clone(), event));

    Extension::new()
        .with_telemetry_processor(telemetry_processor)
        .with_events_processor(events_processor)
        .run()
        .await?;

    Ok(())
}

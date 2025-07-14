use crate::environment::LambdaEnvironment;
use aws_config::SdkConfig;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_ssm::Client as SsmClient;
use eyre::eyre;
use lambda_extension::{LambdaEvent, NextEvent};
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{error, info};

// API_URL contains the backend URL without a trailing slash
const BASE_API_URL: &str = env!("METRICS_API_URL");

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaConfig {
    #[serde(rename = "memorySizeMB")]
    pub memory_size_mb: Option<i32>,
}

/// Handles the Shutdown Lambda event and gracefully shut down the updater task
pub(crate) async fn events_handler(updater: Updater, event: LambdaEvent) -> eyre::Result<()> {
    if let NextEvent::Shutdown(_) = event.next {
        info!("Extension is shutting down");
        updater.shutdown().await?;
    }
    Ok(())
}

/// Periodically updates the Lambda function's memory size based on the provided API
/// Util the shutdown signal is received.
async fn updater_task(
    // Shutdown signal
    mut shutdown_rx: Receiver<()>,
    // Lambda environment variables
    environment: LambdaEnvironment,

    // AWS clients required for update the Lambda function
    clients: AwsClients,
) {
    let client = Client::new();
    let lambda_environment = environment.clone();

    // Use the current memory size from env as a fallback
    let fallback_memory_size = lambda_environment.memory_size_mb;

    // Polling API every 5 minutes to get a new memory size
    let mut interval = interval(Duration::from_secs(60 * 5));

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => break,
            _ = interval.tick() => {
                let api_url = format!("{BASE_API_URL}/config");
                info!("Requesting new RAM size from the provider: {}", api_url);

                let ram_size = match client.get(api_url)
                    .header("Authorization", format!("Bearer {}", lambda_environment.access_token))
                    .timeout(Duration::from_secs(10))
                    .query(&[
                        ("name", lambda_environment.name.clone()),
                        ("region", lambda_environment.region.clone()),
                        ("version", lambda_environment.version.clone()),
                        ("strategy", lambda_environment.strategy.to_string()),
                        ("arn", lambda_environment.arn.clone()),
                    ])
                    .send()
                    .await {
                        Ok(response) => {
                            match response.json::<LambdaConfig>().await {
                                Ok(config) => config.memory_size_mb.unwrap_or(fallback_memory_size),
                                Err(e) => {
                                    error!("Failed to parse response: {:?}", e);
                                    fallback_memory_size
                                }
                            }
                        },

                        Err(e) => {
                            error!("Failed to get a new RAM size: {:?}", e);
                            fallback_memory_size
                        }
                    };

                if ram_size == fallback_memory_size {
                    info!("No new RAM config is available: {}", fallback_memory_size);
                    continue;
                }

                info!("Received a new RAM size: {}", ram_size);

                let lambda = update_lambda_config(&clients.lambda_client, &lambda_environment.name, ram_size);
                let ssm = update_ssm_parameter(&clients.ssm_client, lambda_environment.memory_parameter_name.as_deref(), ram_size);

                let (lambda_result, ssm_result) = tokio::join!(
                    lambda,
                    ssm,
                );

                match (lambda_result, ssm_result) {
                    (Ok(_), Ok(_)) => info!("Lambda and SSM parameters updated successfully"),
                    (Err(e1), Ok(_)) => error!("Failed to update Lambda: {:?}", e1),
                    (Ok(_), Err(e2)) => error!("Failed to update SSM: {:?}", e2),
                    (Err(e1), Err(e2)) => error!("Failed to update Lambda and SSM: {:?} and {:?}", e1, e2),
                }
            }
        }
    }

    info!("Updater completed successfully");
}

/// Updates the SSM parameter with the new RAM size
async fn update_ssm_parameter(
    client: &SsmClient,
    ssm_name: Option<&str>,
    ram_size: i32,
) -> eyre::Result<()> {
    if let Some(ssm_name) = ssm_name {
        info!("Updating SSM parameter: {}", ssm_name);

        client
            .put_parameter()
            .name(ssm_name)
            .value(ram_size.to_string())
            .overwrite(true)
            .send()
            .await
            .map_err(|e| eyre!("Failed to update SSM parameter: {:?}", e))?;
    }

    Ok(())
}

/// Updates the Lambda function configuration with the new RAM size
async fn update_lambda_config(
    client: &LambdaClient,
    function_name: &str,
    ram_size: i32,
) -> eyre::Result<()> {
    info!("Updating Lambda function: {}", function_name);

    client
        .update_function_configuration()
        .function_name(function_name)
        .memory_size(ram_size)
        .send()
        .await
        .map_err(|e| eyre!("Failed to update Lambda configuration: {:?}", e))?;

    Ok(())
}

/// Manages an updater task and provides a way to complete it
#[derive(Debug)]
pub struct Updater {
    /// Arc to hold the inner state to prevent it from being moved while being borrowed
    inner: Arc<Mutex<Option<InnerState>>>,
}

#[derive(Debug)]
struct InnerState {
    /// Handle to the updater task
    handle: JoinHandle<()>,

    /// Channel for sending a shutdown signal to the updater task
    tx: Sender<()>,
}

#[derive(Debug, Clone)]
pub struct AwsClients {
    ssm_client: SsmClient,
    lambda_client: LambdaClient,
}

impl AwsClients {
    pub fn new(aws_config: &SdkConfig) -> Self {
        let ssm_client = SsmClient::new(aws_config);
        let lambda_client = LambdaClient::new(aws_config);

        AwsClients {
            ssm_client,
            lambda_client,
        }
    }
}

impl Updater {
    pub fn new(aws_config: &SdkConfig, environment: LambdaEnvironment) -> Self {
        // Create a channel for gracefully shutting down a task to update the RAM size
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let clients = AwsClients::new(aws_config);

        // Spawn a task to update the RAM size periodically
        let updater_handle = tokio::spawn(updater_task(shutdown_rx, environment, clients));

        Updater {
            inner: Arc::new(Mutex::new(Some(InnerState {
                handle: updater_handle,
                tx: shutdown_tx,
            }))),
        }
    }

    /// Gracefully shut down the updater task
    async fn shutdown(self) -> eyre::Result<()> {
        info!("Sending a shutdown signal to the updater task");

        // Extract the JoinHandle from the Mutex to avoid holding the lock across await
        let handle = {
            let mut inner_guard = self
                .inner
                .lock()
                .map_err(|e| eyre!("Failed to lock state: {:?}", e))?;

            if let Some(inner) = inner_guard.take() {
                // Send shutdown signal
                let _ = inner.tx.send(());
                inner.handle
            } else {
                // Nothing to do if there's no inner state
                return Ok(());
            }
        };

        // Now await on the handle without holding the lock
        if let Err(e) = handle.await {
            error!("Background task failed: {:?}", e);
        }

        Ok(())
    }

    pub(crate) fn clone(&self) -> Self {
        Updater {
            inner: Arc::clone(&self.inner),
        }
    }
}

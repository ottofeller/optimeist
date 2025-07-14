use crate::models::Lambda;
use aws_config::SdkConfig;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_lambda::types::{Architecture, FunctionConfiguration};
use color_eyre::eyre::OptionExt;
use futures::future::join_all;

pub async fn fetch_lambda_functions(aws_config: &SdkConfig) -> color_eyre::Result<Vec<Lambda>> {
    let client = LambdaClient::new(aws_config);
    let mut lambdas = Vec::new();
    let mut next_marker: Option<String> = None;

    loop {
        let mut request = client.list_functions();

        if let Some(marker) = &next_marker {
            request = request.marker(marker);
        }

        let list = request.send().await?;

        if let Some(functions) = list.functions {
            let mut handles = vec![];

            for function in functions {
                let client = client.clone();

                let handle =
                    tokio::spawn(async move { fetch_lambda_config(&client, &function).await });

                handles.push(handle);
            }

            let result = join_all(handles).await;

            for lambda in result.into_iter().flatten().flatten() {
                lambdas.push(lambda);
            }
        }

        next_marker = list.next_marker;
        if next_marker.is_none() {
            break;
        }
    }

    Ok(lambdas)
}

async fn fetch_lambda_config(
    client: &LambdaClient,
    function: &FunctionConfiguration,
) -> color_eyre::Result<Lambda> {
    let name = function
        .function_name
        .clone()
        .ok_or_eyre("No name provided")?;

    let arn = function
        .function_arn
        .clone()
        .ok_or_eyre("No arn provided")?;

    let function_config = client.get_function().function_name(&name).send().await?;
    let config = function_config.configuration;

    let architecture = config
        .as_ref()
        .and_then(|config| config.architectures.clone())
        .and_then(|archs| archs.first().cloned())
        .unwrap_or(Architecture::X8664);

    let layers = config
        .as_ref()
        .and_then(|config| config.layers.clone())
        .unwrap_or(vec![]);

    let role = config
        .as_ref()
        .and_then(|config| config.role.clone())
        .unwrap_or_default();

    let variables = config
        .as_ref()
        .and_then(|config| config.environment.clone())
        .and_then(|env| env.variables.clone())
        .unwrap_or_default();

    let is_installed = layers
        .iter()
        .any(|item| item.arn.clone().unwrap_or_default().contains("optimeist"));

    Ok(Lambda {
        // Select the lambdas for which installation has already been performed
        is_selected: is_installed,
        is_installed,
        arn,
        architecture,
        layers,
        name,
        role,
        variables,
    })
}

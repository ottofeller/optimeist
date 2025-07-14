use crate::aws::policy::generate_policy_for_lambda;
use crate::models::{LAYERS, Lambda};
use aws_config::{Region, SdkConfig};
use aws_sdk_iam::Client as IamClient;
use aws_sdk_lambda::types::{Architecture, Environment};
use color_eyre::eyre::OptionExt;

pub async fn install_extension(
    aws_config: &SdkConfig,
    lambda: Lambda,
    secret_arn: &str,
) -> color_eyre::Result<()> {
    use aws_sdk_lambda::Client as LambdaClient;
    let client = LambdaClient::new(aws_config);
    let iam_client = IamClient::new(aws_config);

    let region = aws_config
        .region()
        .unwrap_or(&Region::new("us-east-1"))
        .to_string();

    let layer = LAYERS
        .get(&region)
        .ok_or_eyre(format!("Layer isn't found for a region: {region}"))?;

    let layer_anr = match lambda.architecture {
        Architecture::Arm64 => layer.arm64.clone(),
        Architecture::X8664 => layer.x86_64.clone(),
        _ => layer.x86_64.clone(),
    };

    let role_name = lambda
        .role
        .split('/')
        .next_back()
        .ok_or_else(|| color_eyre::eyre::eyre!("Invalid role ARN format"))?;

    let policy_document = generate_policy_for_lambda(&lambda, secret_arn);
    let inline_policy_name = format!("OptimeistPolicy-{}", lambda.name);

    let mut variables = lambda.variables.clone();

    variables.insert(
        "OPTIMEIST_ACCESS_TOKEN_SECRET_ARN".to_string(),
        secret_arn.to_string(),
    );

    iam_client
        .put_role_policy()
        .role_name(role_name)
        .policy_name(&inline_policy_name)
        .policy_document(policy_document)
        .send()
        .await?;

    client
        .update_function_configuration()
        .function_name(&lambda.name)
        // There is no need to check that layer already attached
        // All previously attached layers will be preserved
        .layers(layer_anr)
        .environment(
            Environment::builder()
                .set_variables(Some(variables))
                .build(),
        )
        .send()
        .await?;

    Ok(())
}

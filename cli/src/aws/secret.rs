use aws_config::SdkConfig;
use aws_sdk_secretsmanager::Client as SecretsClient;
use color_eyre::eyre::Context;

const SECRET_NAME: &str = "optimeist-api-key";

pub async fn get_or_create_secret(config: &SdkConfig) -> color_eyre::Result<String> {
    let client = SecretsClient::new(config);

    match client
        .get_secret_value()
        .secret_id(SECRET_NAME)
        .send()
        .await
    {
        Ok(response) => Ok(response.arn.unwrap_or_default()),
        Err(_) => {
            let api_key =
                std::env::var("OPTIMEIST_API_KEY").wrap_err("OPTIMEIST_API_KEY is required")?;

            Ok(client
                .create_secret()
                .name(SECRET_NAME)
                .secret_string(&api_key)
                .send()
                .await?
                .arn
                .unwrap_or_default())
        }
    }
}

use aws_config::SdkConfig;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_secretsmanager::Client as SecretsClient;
use eyre::{Context, OptionExt, Result};
use serde::Serialize;
use std::env;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// Contains information about the AWS Lambda function environment
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaEnvironment {
    pub arn: String,
    pub region: String,
    pub version: String,
    pub name: String,
    pub memory_size_mb: i32,
    pub strategy: Strategy,

    #[serde(skip)]
    pub memory_parameter_name: Option<String>,

    #[serde(skip)]
    pub access_token: String,
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Strategy {
    Cost,
    Speed,
    #[default]
    Balanced,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Strategy::Cost => write!(f, "COST"),
            Strategy::Speed => write!(f, "SPEED"),
            // FIXME Add weight to BALANCED strategy like BALANCED#0.5
            Strategy::Balanced => write!(f, "BALANCED"),
        }
    }
}

impl FromStr for Strategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "COST" => Ok(Strategy::Cost),
            "SPEED" => Ok(Strategy::Speed),
            // FIXME Add weight to BALANCED strategy like BALANCED#0.5
            "BALANCED" => Ok(Strategy::Balanced),
            _ => Ok(Strategy::Balanced), // Using Balanced as default
        }
    }
}

// This will allow conversion from any type that can be converted to &str
impl<T: AsRef<str>> From<T> for Strategy {
    fn from(s: T) -> Self {
        Strategy::from_str(s.as_ref()).unwrap_or_default()
    }
}

impl LambdaEnvironment {
    pub async fn new(config: &SdkConfig) -> Result<Self> {
        let access_token_secret_arn = env::var("OPTIMEIST_ACCESS_TOKEN_SECRET_ARN")
            .wrap_err("Failed to get OPTIMEIST_ACCESS_TOKEN_SECRET_ARN env variable")?;

        let secret_client = SecretsClient::new(config);

        let response = secret_client
            .get_secret_value()
            .secret_id(access_token_secret_arn.clone())
            .send()
            .await
            .wrap_err("Failed to get access token secret value")?;

        let access_token = response
            .secret_string
            .ok_or_eyre("Failed to get a secret string")?;

        let function_name = env::var("AWS_LAMBDA_FUNCTION_NAME")?;
        let lambda_client = LambdaClient::new(config);

        let function_response = lambda_client
            .get_function()
            .function_name(function_name.clone())
            .send()
            .await
            .wrap_err("Failed to get function details")?;

        Ok(Self {
            arn: function_response
                .configuration
                .and_then(|config| config.function_arn)
                .ok_or_eyre("Failed to get function ARN")?,

            access_token,
            strategy: Strategy::from(
                env::var("OPTIMEIST_DECISION_ALGORITHM_TYPE").unwrap_or("balanced".to_string()),
            ),
            region: env::var("AWS_REGION")?,
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION")?,
            name: function_name,
            memory_size_mb: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")?.parse()?,
            memory_parameter_name: env::var("OPTIMEIST_MEMORY_PARAMETER_NAME").ok(),
        })
    }
}

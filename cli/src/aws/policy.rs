use crate::models::Lambda;
use serde_json::json;

pub fn generate_policy_for_lambda(lambda: &Lambda, secret_arn: &str) -> String {
    let policy = json!({
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Action": [
                    "lambda:GetFunction",
                    "lambda:UpdateFunctionConfiguration"
                ],
                "Resource": lambda.arn
            },
            {
                "Effect": "Allow",
                "Action": [
                    "secretsmanager:DescribeSecret",
                    "secretsmanager:GetSecretValue"
                ],
                "Resource": secret_arn
            }
        ]
    });

    serde_json::to_string_pretty(&policy).unwrap_or_default()
}

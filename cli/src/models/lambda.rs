use aws_sdk_lambda::types::{Architecture, Layer};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Lambda {
    pub name: String,
    pub arn: String,
    pub is_selected: bool,
    pub is_installed: bool,
    pub architecture: Architecture,
    pub layers: Vec<Layer>,
    pub role: String,
    pub variables: HashMap<String, String>,
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arn == other.arn
    }
}

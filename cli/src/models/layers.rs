use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct LayerInfo {
    pub arm64: String,
    pub x86_64: String,
}

type RegionLayers = HashMap<String, LayerInfo>;

const LAYERS_JSON: &str = include_str!("../../../src/layers.json");

pub static LAYERS: Lazy<RegionLayers> =
    Lazy::new(|| serde_json::from_str(LAYERS_JSON).expect("Failed to parse layers.json"));

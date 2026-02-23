use crate::blender_model_editor::MODELS_CONFIG_PATH;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsConfig {
    pub models: Vec<ModelDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDefinition {
    pub id: String,
    pub name: String,
    pub script: String,
    pub default_glb: String,
    pub params: Vec<ParamDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParamDefinition {
    pub key: String,
    pub label: String,
    pub kind: String,
    pub default: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub help: Option<String>,
}

pub fn load_models_config() -> Result<ModelsConfig, String> {
    let text = fs::read_to_string(MODELS_CONFIG_PATH)
        .map_err(|err| format!("failed to read {MODELS_CONFIG_PATH}: {err}"))?;
    let config = ron::de::from_str::<ModelsConfig>(&text)
        .map_err(|err| format!("failed to parse {MODELS_CONFIG_PATH}: {err}"))?;
    if config.models.is_empty() {
        return Err("config has no models".to_string());
    }
    Ok(config)
}

pub fn default_models_config() -> ModelsConfig {
    ModelsConfig {
        models: vec![ModelDefinition {
            id: "table".to_string(),
            name: "Stylized Table".to_string(),
            script: "assets/blender_ai/table.py".to_string(),
            default_glb: "assets/models/table.glb".to_string(),
            params: vec![
                ParamDefinition {
                    key: "top-width".to_string(),
                    label: "Top Width".to_string(),
                    kind: "float".to_string(),
                    default: "1.2".to_string(),
                    min: Some(0.6),
                    max: Some(3.0),
                    step: Some(0.01),
                    help: None,
                },
                ParamDefinition {
                    key: "top-depth".to_string(),
                    label: "Top Depth".to_string(),
                    kind: "float".to_string(),
                    default: "1.2".to_string(),
                    min: Some(0.6),
                    max: Some(3.0),
                    step: Some(0.01),
                    help: None,
                },
                ParamDefinition {
                    key: "top-thickness".to_string(),
                    label: "Top Thickness".to_string(),
                    kind: "float".to_string(),
                    default: "0.08".to_string(),
                    min: Some(0.02),
                    max: Some(0.30),
                    step: Some(0.005),
                    help: None,
                },
                ParamDefinition {
                    key: "table-height".to_string(),
                    label: "Table Height".to_string(),
                    kind: "float".to_string(),
                    default: "0.75".to_string(),
                    min: Some(0.3),
                    max: Some(2.5),
                    step: Some(0.01),
                    help: None,
                },
                ParamDefinition {
                    key: "leg-thickness".to_string(),
                    label: "Leg Thickness".to_string(),
                    kind: "float".to_string(),
                    default: "0.10".to_string(),
                    min: Some(0.04),
                    max: Some(0.5),
                    step: Some(0.005),
                    help: None,
                },
                ParamDefinition {
                    key: "inset".to_string(),
                    label: "Leg Inset".to_string(),
                    kind: "float".to_string(),
                    default: "0.08".to_string(),
                    min: Some(0.0),
                    max: Some(0.4),
                    step: Some(0.005),
                    help: None,
                },
                ParamDefinition {
                    key: "top-taper".to_string(),
                    label: "Top Taper".to_string(),
                    kind: "float".to_string(),
                    default: "0.90".to_string(),
                    min: Some(0.6),
                    max: Some(1.0),
                    step: Some(0.01),
                    help: None,
                },
                ParamDefinition {
                    key: "leg-taper".to_string(),
                    label: "Leg Taper".to_string(),
                    kind: "float".to_string(),
                    default: "0.82".to_string(),
                    min: Some(0.6),
                    max: Some(1.0),
                    step: Some(0.01),
                    help: None,
                },
                ParamDefinition {
                    key: "leg-splay-deg".to_string(),
                    label: "Leg Splay (deg)".to_string(),
                    kind: "float".to_string(),
                    default: "5.0".to_string(),
                    min: Some(0.0),
                    max: Some(20.0),
                    step: Some(0.1),
                    help: None,
                },
                ParamDefinition {
                    key: "top-warp".to_string(),
                    label: "Top Warp".to_string(),
                    kind: "float".to_string(),
                    default: "0.008".to_string(),
                    min: Some(0.0),
                    max: Some(0.03),
                    step: Some(0.001),
                    help: None,
                },
            ],
        }],
    }
}

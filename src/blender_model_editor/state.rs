use crate::blender_model_editor::model::{ModelsConfig, default_models_config, load_models_config};
use crate::blender_model_editor::{
    LEGACY_PRESETS_PATH, LEGACY_PRESETS_PATH_OLD, LIVE_REPORT_PATH, PRESETS_PATH,
};
use bevy::prelude::Resource;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SavedEditorState {
    pub models: HashMap<String, SavedModelParams>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SavedModelParams {
    pub values: HashMap<String, String>,
    pub export_path: Option<String>,
}

#[derive(Resource)]
pub struct EditorState {
    pub config: ModelsConfig,
    pub selected_model_idx: usize,
    pub values: HashMap<String, String>,
    pub dirty: bool,
    pub request_center_view: bool,
    pub show_grid: bool,
    pub report_path: PathBuf,
    pub presets_path: PathBuf,
    pub presets: SavedEditorState,
    pub export_path: String,
    pub status: String,
    pub last_command: String,
    pub last_stdout: String,
    pub last_stderr: String,
    pub report_text: String,
}

impl EditorState {
    pub fn current_model(&self) -> &crate::blender_model_editor::model::ModelDefinition {
        &self.config.models[self.selected_model_idx]
    }

    pub fn current_model_id(&self) -> String {
        self.current_model().id.clone()
    }

    pub fn reset_values_from_defaults(&mut self) -> bool {
        self.values.clear();
        let params = self.current_model().params.clone();
        for param in &params {
            self.values.insert(param.key.clone(), param.default.clone());
        }
        self.export_path = self.current_model().default_glb.clone();
        let loaded = self.apply_saved_for_current_model();
        self.dirty = true;
        loaded
    }

    pub fn apply_saved_for_current_model(&mut self) -> bool {
        let model_id = self.current_model_id();
        let Some(saved) = self.presets.models.get(&model_id).cloned() else {
            return false;
        };

        let valid_keys: HashSet<String> = self
            .current_model()
            .params
            .iter()
            .map(|p| p.key.clone())
            .collect();

        for (key, value) in saved.values {
            if valid_keys.contains(&key) {
                self.values.insert(key, value);
            }
        }

        if let Some(path) = saved.export_path {
            self.export_path = path;
        }

        true
    }

    pub fn save_current_model_preset(&mut self) -> Result<(), String> {
        let model_id = self.current_model_id();
        let preset = SavedModelParams {
            values: self.values.clone(),
            export_path: Some(self.export_path.clone()),
        };
        self.presets.models.insert(model_id.clone(), preset.clone());
        self.save_presets_to_disk()?;
        self.save_snapshot_to_history(&model_id, &preset)
    }

    pub fn reload_presets_from_disk(&mut self) -> Result<(), String> {
        self.presets = load_saved_presets(&self.presets_path)?;
        Ok(())
    }

    pub fn save_presets_to_disk(&self) -> Result<(), String> {
        if let Some(parent) = self.presets_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create preset dir: {err}"))?;
        }

        let content = ron::ser::to_string_pretty(&self.presets, PrettyConfig::new())
            .map_err(|err| format!("failed to serialize presets: {err}"))?;
        fs::write(&self.presets_path, content)
            .map_err(|err| format!("failed to write {}: {err}", self.presets_path.display()))
    }

    fn save_snapshot_to_history(
        &self,
        model_id: &str,
        preset: &SavedModelParams,
    ) -> Result<(), String> {
        let Some(presets_root) = self.presets_path.parent() else {
            return Err("presets path has no parent directory".to_string());
        };
        let history_dir = presets_root.join("history").join(model_id);
        fs::create_dir_all(&history_dir).map_err(|err| {
            format!(
                "failed to create history dir {}: {err}",
                history_dir.display()
            )
        })?;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error while creating snapshot timestamp: {err}"))?
            .as_millis();
        let snapshot_path = history_dir.join(format!("{timestamp_ms}.ron"));
        let content = ron::ser::to_string_pretty(preset, PrettyConfig::new())
            .map_err(|err| format!("failed to serialize snapshot: {err}"))?;
        fs::write(&snapshot_path, content)
            .map_err(|err| format!("failed to write {}: {err}", snapshot_path.display()))
    }

    pub fn model_default(&self, key: &str) -> Option<&str> {
        self.current_model()
            .params
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.default.as_str())
    }

    pub fn get_f32(&self, key: &str, hard_fallback: f32) -> f32 {
        self.values
            .get(key)
            .and_then(|raw| raw.parse::<f32>().ok())
            .or_else(|| {
                self.model_default(key)
                    .and_then(|raw| raw.parse::<f32>().ok())
            })
            .unwrap_or(hard_fallback)
    }
}

pub fn load_saved_presets(path: &Path) -> Result<SavedEditorState, String> {
    if !path.exists() {
        return Ok(SavedEditorState::default());
    }
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    ron::de::from_str::<SavedEditorState>(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub fn load_initial_state() -> EditorState {
    let config = load_models_config().unwrap_or_else(|err| {
        eprintln!("Falling back to built-in model config: {err}");
        default_models_config()
    });

    let report_path = PathBuf::from(LIVE_REPORT_PATH);
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let presets_path = PathBuf::from(PRESETS_PATH);
    let legacy_presets_path = PathBuf::from(LEGACY_PRESETS_PATH);
    let legacy_presets_path_old = PathBuf::from(LEGACY_PRESETS_PATH_OLD);
    let (presets, loaded_from_legacy) = if presets_path.exists() {
        (
            load_saved_presets(&presets_path).unwrap_or_else(|err| {
                eprintln!("Ignoring saved presets due to parse/read error: {err}");
                SavedEditorState::default()
            }),
            false,
        )
    } else if legacy_presets_path.exists() {
        (
            load_saved_presets(&legacy_presets_path).unwrap_or_else(|err| {
                eprintln!("Ignoring legacy saved presets due to parse/read error: {err}");
                SavedEditorState::default()
            }),
            true,
        )
    } else if legacy_presets_path_old.exists() {
        (
            load_saved_presets(&legacy_presets_path_old).unwrap_or_else(|err| {
                eprintln!("Ignoring legacy saved presets due to parse/read error: {err}");
                SavedEditorState::default()
            }),
            true,
        )
    } else {
        (SavedEditorState::default(), false)
    };

    let mut state = EditorState {
        export_path: String::new(),
        config,
        selected_model_idx: 0,
        values: HashMap::new(),
        dirty: false,
        request_center_view: true,
        show_grid: true,
        report_path,
        presets_path,
        presets,
        status: "Ready".to_string(),
        last_command: String::new(),
        last_stdout: String::new(),
        last_stderr: String::new(),
        report_text: String::new(),
    };

    let loaded = state.reset_values_from_defaults();
    if loaded {
        state.status = if loaded_from_legacy {
            "Loaded saved parameters (migrated from _artifacts)".to_string()
        } else {
            "Loaded saved parameters".to_string()
        };
    }

    if loaded_from_legacy {
        if let Err(err) = state.save_presets_to_disk() {
            eprintln!(
                "Failed to migrate presets to {}: {err}",
                state.presets_path.display()
            );
        }
    }

    state
}

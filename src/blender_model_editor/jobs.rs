use crate::blender_model_editor::model::{ModelDefinition, ParamDefinition};
use crate::blender_model_editor::state::EditorState;
use bevy::prelude::{NonSendMut, ResMut};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone, Copy)]
pub enum JobKind {
    Validate,
    ExportGlb,
}

#[derive(Debug, Clone)]
pub struct JobResult {
    pub kind: JobKind,
    pub success: bool,
    pub command_line: String,
    pub stdout: String,
    pub stderr: String,
}

pub struct JobQueue {
    pub tx: Sender<JobResult>,
    pub rx: Receiver<JobResult>,
    pub running: bool,
}

impl Default for JobQueue {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            running: false,
        }
    }
}

pub fn spawn_blender_job(
    state: &mut EditorState,
    queue: &mut JobQueue,
    kind: JobKind,
) -> Result<(), String> {
    if queue.running {
        return Err("job already running".to_string());
    }

    let model = state.current_model().clone();
    let values = state.values.clone();
    let report_path = state.report_path.clone();
    let export_path = state.export_path.clone();

    let (args, command_line) = match kind {
        JobKind::Validate => build_blender_args_for_validate(&model, &values, &report_path),
        JobKind::ExportGlb => {
            build_blender_args_for_export(&model, &values, &export_path, &report_path)
        }
    };

    state.last_command = command_line.clone();
    state.status = "Running Blender...".to_string();
    queue.running = true;
    let tx = queue.tx.clone();

    std::thread::spawn(move || {
        let output = Command::new("blender").args(&args).output();
        let result = match output {
            Ok(out) => JobResult {
                kind,
                success: out.status.success(),
                command_line,
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            },
            Err(err) => JobResult {
                kind,
                success: false,
                command_line,
                stdout: String::new(),
                stderr: format!("failed to spawn blender: {err}"),
            },
        };
        let _ = tx.send(result);
    });

    Ok(())
}

pub fn poll_finished_jobs(mut state: ResMut<EditorState>, mut queue: NonSendMut<JobQueue>) {
    let Ok(result) = queue.rx.try_recv() else {
        return;
    };

    queue.running = false;
    state.last_stdout = result.stdout.clone();
    state.last_stderr = result.stderr.clone();
    state.last_command = result.command_line.clone();

    if result.success {
        match result.kind {
            JobKind::Validate => {
                state.status = "Validate succeeded".to_string();
            }
            JobKind::ExportGlb => {
                state.status = format!("Export succeeded: {}", state.export_path);
            }
        }
        if let Ok(text) = fs::read_to_string(&state.report_path) {
            state.report_text = text;
        }
    } else {
        state.status = "Blender command failed".to_string();
    }
}

fn build_blender_args_for_validate(
    model: &ModelDefinition,
    values: &HashMap<String, String>,
    report_path: &Path,
) -> (Vec<String>, String) {
    let mut args = vec![
        "-b".to_string(),
        "--factory-startup".to_string(),
        "--python".to_string(),
        model.script.clone(),
        "--".to_string(),
        "--validate".to_string(),
    ];
    append_param_args(&mut args, &model.params, values);
    args.push("--report-json".to_string());
    args.push(report_path.display().to_string());

    let command_line = format!("blender {}", args.join(" "));
    (args, command_line)
}

fn build_blender_args_for_export(
    model: &ModelDefinition,
    values: &HashMap<String, String>,
    export_path: &str,
    report_path: &Path,
) -> (Vec<String>, String) {
    let mut args = vec![
        "-b".to_string(),
        "--factory-startup".to_string(),
        "--python".to_string(),
        model.script.clone(),
        "--".to_string(),
        "--validate".to_string(),
    ];
    append_param_args(&mut args, &model.params, values);
    args.push("--export-glb".to_string());
    args.push(export_path.to_string());
    args.push("--report-json".to_string());
    args.push(report_path.display().to_string());

    let command_line = format!("blender {}", args.join(" "));
    (args, command_line)
}

fn append_param_args(
    args: &mut Vec<String>,
    params: &[ParamDefinition],
    values: &HashMap<String, String>,
) {
    for param in params {
        let key = format!("--{}", param.key);
        let raw = values
            .get(&param.key)
            .cloned()
            .unwrap_or_else(|| param.default.clone());

        match param.kind.as_str() {
            "bool" => {
                if parse_bool(&raw) {
                    args.push(key);
                }
            }
            _ => {
                args.push(key);
                args.push(raw);
            }
        }
    }
}

pub fn parse_bool(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

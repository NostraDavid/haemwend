use crate::blender_model_editor::camera::{
    CameraPreset, OrbitCameraState, UiInteractionState, apply_camera_preset,
};
use crate::blender_model_editor::jobs::{JobKind, JobQueue, parse_bool, spawn_blender_job};
use crate::blender_model_editor::model::ParamDefinition;
use crate::blender_model_editor::preview::grid_info_text;
use crate::blender_model_editor::state::EditorState;
use bevy::prelude::{NonSendMut, ResMut};
use bevy_egui::{EguiContexts, egui};
use std::collections::HashMap;

pub fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut queue: NonSendMut<JobQueue>,
    mut ui_state: ResMut<UiInteractionState>,
    mut orbit: ResMut<OrbitCameraState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("blender_model_editor_top_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Blender Model Editor");
            ui.separator();
            ui.label(format!("Status: {}", state.status));
            if queue.running {
                ui.label("(running)");
            }
            ui.separator();
            ui.small("Viewport controls: RMB rotate, MMB pan, wheel zoom.");
        });
    });

    let side_panel_response = egui::SidePanel::left("blender_model_editor_controls")
        .resizable(true)
        .default_width(440.0)
        .show(ctx, |ui| {
            ui.heading("Model");

            let prev_selected = state.selected_model_idx;
            let model_labels: Vec<(usize, String)> = state
                .config
                .models
                .iter()
                .enumerate()
                .map(|(idx, model)| (idx, format!("{} ({})", model.name, model.id)))
                .collect();

            egui::ComboBox::from_label("Active Model")
                .selected_text(state.current_model().name.clone())
                .show_ui(ui, |ui| {
                    for (idx, label) in &model_labels {
                        ui.selectable_value(&mut state.selected_model_idx, *idx, label);
                    }
                });

            if state.selected_model_idx != prev_selected {
                let loaded = state.reset_values_from_defaults();
                state.report_text.clear();
                state.last_stdout.clear();
                state.last_stderr.clear();
                state.request_center_view = true;
                state.status = if loaded {
                    "Model switched (saved parameters loaded)".to_string()
                } else {
                    "Model switched".to_string()
                };
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Center View").clicked() {
                    state.request_center_view = true;
                }
                ui.checkbox(&mut state.show_grid, "Show grid");
            });
            ui.small(grid_info_text());

            ui.horizontal_wrapped(|ui| {
                ui.label("View Presets:");
                if ui.button("Iso L").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::IsoLeft);
                }
                if ui.button("Iso R").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::IsoRight);
                }
                if ui.button("Front").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::Front);
                }
                if ui.button("Back").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::Back);
                }
                if ui.button("Left").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::Left);
                }
                if ui.button("Right").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::Right);
                }
                if ui.button("Top").clicked() {
                    apply_camera_preset(&mut orbit, &state, CameraPreset::Top);
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Save Params").clicked() {
                    match state.save_current_model_preset() {
                        Ok(()) => {
                            state.status =
                                format!("Parameters saved to {}", state.presets_path.display())
                        }
                        Err(err) => state.status = format!("Save failed: {err}"),
                    }
                }

                if ui.button("Load Params").clicked() {
                    match state.reload_presets_from_disk() {
                        Ok(()) => {
                            if state.apply_saved_for_current_model() {
                                state.dirty = true;
                                state.request_center_view = true;
                                state.status = "Parameters loaded".to_string();
                            } else {
                                state.status = "No saved parameters for this model".to_string();
                            }
                        }
                        Err(err) => state.status = format!("Load failed: {err}"),
                    }
                }

                if ui.button("Reset Defaults").clicked() {
                    state.reset_values_from_defaults();
                    state.request_center_view = true;
                    state.status = "Reset to defaults".to_string();
                }
            });

            ui.separator();
            ui.heading("Parameters");

            let params = state.current_model().params.clone();
            for param in &params {
                if draw_param_control(ui, &mut state.values, param) {
                    state.dirty = true;
                }
            }

            ui.separator();
            ui.heading("Output");
            ui.label("GLB export path");
            ui.text_edit_singleline(&mut state.export_path);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Rebuild Preview").clicked() {
                    state.dirty = true;
                }

                if ui
                    .add_enabled(!queue.running, egui::Button::new("Validate in Blender"))
                    .clicked()
                {
                    let _ = spawn_blender_job(&mut state, &mut queue, JobKind::Validate);
                }

                if ui
                    .add_enabled(!queue.running, egui::Button::new("Export GLB"))
                    .clicked()
                {
                    let _ = spawn_blender_job(&mut state, &mut queue, JobKind::ExportGlb);
                }
            });

            ui.separator();
            ui.collapsing("Last Blender Command", |ui| {
                ui.code(state.last_command.clone());
            });
            ui.collapsing("Report JSON", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.code(state.report_text.clone());
                    });
            });
            ui.collapsing("Stderr", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.code(state.last_stderr.clone());
                    });
            });
        });

    ui_state.wants_pointer_input = ctx.wants_pointer_input();
    ui_state.wants_keyboard_input = ctx.wants_keyboard_input();
    ui_state.side_panel_width = side_panel_response.response.rect.width();
}

fn draw_param_control(
    ui: &mut egui::Ui,
    values: &mut HashMap<String, String>,
    param: &ParamDefinition,
) -> bool {
    let current = values
        .get(&param.key)
        .cloned()
        .unwrap_or_else(|| param.default.clone());

    let changed = match param.kind.as_str() {
        "float" => {
            let mut value = current
                .parse::<f32>()
                .unwrap_or_else(|_| param.default.parse::<f32>().unwrap_or(0.0));
            let response = if let (Some(min), Some(max)) = (param.min, param.max) {
                let step = param.step.unwrap_or(0.01);
                ui.add(
                    egui::Slider::new(&mut value, min..=max)
                        .text(param.label.clone())
                        .step_by(step as f64),
                )
            } else {
                ui.horizontal(|ui| {
                    ui.label(param.label.clone());
                    ui.add(
                        egui::DragValue::new(&mut value).speed(param.step.unwrap_or(0.01) as f64),
                    )
                })
                .inner
            };
            if response.changed() {
                values.insert(param.key.clone(), format!("{value:.6}"));
                true
            } else {
                false
            }
        }
        "int" => {
            let mut value = current
                .parse::<i64>()
                .unwrap_or_else(|_| param.default.parse::<i64>().unwrap_or(0));
            let response = if let (Some(min), Some(max)) = (param.min, param.max) {
                ui.add(
                    egui::Slider::new(&mut value, min as i64..=max as i64)
                        .text(param.label.clone()),
                )
            } else {
                ui.horizontal(|ui| {
                    ui.label(param.label.clone());
                    ui.add(egui::DragValue::new(&mut value).speed(param.step.unwrap_or(1.0) as f64))
                })
                .inner
            };
            if response.changed() {
                values.insert(param.key.clone(), value.to_string());
                true
            } else {
                false
            }
        }
        "bool" => {
            let mut value = parse_bool(&current);
            let response = ui.checkbox(&mut value, param.label.clone());
            if response.changed() {
                values.insert(
                    param.key.clone(),
                    if value { "true" } else { "false" }.to_string(),
                );
                true
            } else {
                false
            }
        }
        "string" => {
            let mut value = current;
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label(param.label.clone());
                changed = ui.text_edit_singleline(&mut value).changed();
            });
            if changed {
                values.insert(param.key.clone(), value);
                true
            } else {
                false
            }
        }
        _ => {
            ui.label(format!(
                "{} (unsupported kind '{}')",
                param.label, param.kind
            ));
            false
        }
    };

    if let Some(help) = &param.help {
        ui.small(help);
    }

    changed
}

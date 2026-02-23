use super::*;

pub(super) fn parse_cli_options() -> CliOptions {
    let mut options = CliOptions::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" | "-s" => {
                let Some(value) = args.next() else {
                    eprintln!("--scenario verwacht een scenario-id");
                    print_cli_help_and_exit(2);
                };
                options.scenario_id = Some(value);
            }
            "--scenarios-dir" | "--scenarios-path" | "--scenarios-file" => {
                let Some(value) = args.next() else {
                    eprintln!("{arg} verwacht een pad");
                    print_cli_help_and_exit(2);
                };
                options.scenarios_path = value;
            }
            "--help" | "-h" => {
                print_cli_help_and_exit(0);
            }
            _ => {
                eprintln!("Onbekende optie: {arg}");
                print_cli_help_and_exit(2);
            }
        }
    }

    options
}

pub(super) fn print_cli_help_and_exit(code: i32) -> ! {
    println!(
        "Gebruik:\n  haemwend [opties]\n\nOpties:\n  -s, --scenario <id>         Start direct met scenario-id\n      --scenarios-dir <pad>   Map met scenario-bestanden (1 .ron per scenario)\n      --scenarios-path <pad>  Alias voor --scenarios-dir\n      --scenarios-file <pad>  Legacy alias (ondersteunt ook 1 bestand)\n  -h, --help                  Toon hulp"
    );
    std::process::exit(code);
}

pub(super) fn default_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        ScenarioDefinition {
            id: "greenwood".to_string(),
            name: "Greenwood Valley".to_string(),
            description: "Open veld met verspreide kratten en muursegmenten.".to_string(),
            ground_extent: 120.0,
            crate_grid_radius: 8,
            crate_spacing: 3.0,
            crate_pattern_mod: 4,
            wall_count: 5,
            wall_spacing: 3.2,
            wall_z: -20.0,
            tower_z: -30.0,
            sun_position: [18.0, 24.0, 12.0],
        },
        ScenarioDefinition {
            id: "arena".to_string(),
            name: "Iron Arena".to_string(),
            description: "Compacte arena met dichter op elkaar staande obstakels.".to_string(),
            ground_extent: 80.0,
            crate_grid_radius: 6,
            crate_spacing: 2.6,
            crate_pattern_mod: 3,
            wall_count: 7,
            wall_spacing: 2.6,
            wall_z: -16.0,
            tower_z: -24.0,
            sun_position: [14.0, 20.0, 10.0],
        },
        ScenarioDefinition {
            id: "canyon".to_string(),
            name: "Red Canyon".to_string(),
            description: "Langgerekte map met pilaren en sterke dieptewerking.".to_string(),
            ground_extent: 180.0,
            crate_grid_radius: 10,
            crate_spacing: 3.4,
            crate_pattern_mod: 5,
            wall_count: 9,
            wall_spacing: 3.5,
            wall_z: -30.0,
            tower_z: -42.0,
            sun_position: [22.0, 30.0, 14.0],
        },
        ScenarioDefinition {
            id: "gauntlet".to_string(),
            name: "Stone Gauntlet".to_string(),
            description: "Smalle route met dichte obstakels voor korte, intensieve runs."
                .to_string(),
            ground_extent: 72.0,
            crate_grid_radius: 5,
            crate_spacing: 2.2,
            crate_pattern_mod: 2,
            wall_count: 11,
            wall_spacing: 2.1,
            wall_z: -14.0,
            tower_z: -20.0,
            sun_position: [12.0, 18.0, 8.0],
        },
        ScenarioDefinition {
            id: "highlands".to_string(),
            name: "Frost Highlands".to_string(),
            description: "Grote open vlakte met weinig dekking en lange zichtlijnen.".to_string(),
            ground_extent: 240.0,
            crate_grid_radius: 12,
            crate_spacing: 4.2,
            crate_pattern_mod: 6,
            wall_count: 4,
            wall_spacing: 5.5,
            wall_z: -40.0,
            tower_z: -58.0,
            sun_position: [28.0, 35.0, 16.0],
        },
    ]
}

pub(super) fn is_ron_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ron"))
}

pub(super) fn filter_valid_scenarios(
    mut scenarios: Vec<ScenarioDefinition>,
    source: &str,
) -> Vec<ScenarioDefinition> {
    scenarios.retain(|scenario| !scenario.id.trim().is_empty() && !scenario.name.trim().is_empty());
    if scenarios.is_empty() {
        eprintln!("Scenario-bron ({source}) bevat geen geldige scenario's");
    }
    scenarios
}

pub(super) fn write_default_scenarios_to_dir(path: &Path) -> bool {
    if let Err(err) = fs::create_dir_all(path) {
        eprintln!("Kon scenario-map niet maken ({}): {err}", path.display());
        return false;
    }

    let pretty = ron::ser::PrettyConfig::default();
    for scenario in default_scenarios() {
        let file_path = path.join(format!("{}.ron", scenario.id));
        if file_path.exists() {
            continue;
        }

        let serialized = match ron::ser::to_string_pretty(&scenario, pretty.clone()) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Kon scenario '{}' niet serialiseren: {err}", scenario.id);
                return false;
            }
        };

        if let Err(err) = fs::write(&file_path, serialized) {
            eprintln!("Kon scenario niet opslaan ({}): {err}", file_path.display());
            return false;
        }
    }

    true
}

pub(super) fn write_default_scenarios_to_file(path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Kon scenario-map niet maken ({}): {err}", parent.display());
            return false;
        }
    }

    let serialized =
        match ron::ser::to_string_pretty(&default_scenarios(), ron::ser::PrettyConfig::default()) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Kon standaardscenario's niet serialiseren: {err}");
                return false;
            }
        };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!(
            "Kon scenario-bestand niet opslaan ({}): {err}",
            path.display()
        );
        return false;
    }

    true
}

pub(super) fn load_scenarios_from_file(path: &Path) -> Vec<ScenarioDefinition> {
    let source = path.display().to_string();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!(
                "Kon scenario-bestand niet lezen ({}): {err}",
                path.display()
            );
            return Vec::new();
        }
    };

    match ron::from_str::<Vec<ScenarioDefinition>>(&content) {
        Ok(scenarios) => return filter_valid_scenarios(scenarios, &source),
        Err(_) => {}
    }

    match ron::from_str::<ScenarioDefinition>(&content) {
        Ok(scenario) => filter_valid_scenarios(vec![scenario], &source),
        Err(err) => {
            eprintln!("Kon scenario's niet parsen ({source}): {err}");
            Vec::new()
        }
    }
}

pub(super) fn load_scenarios_from_dir(path: &Path) -> Vec<ScenarioDefinition> {
    let dir_iter = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Kon scenario-map niet lezen ({}): {err}", path.display());
            return Vec::new();
        }
    };

    let mut files = Vec::<PathBuf>::new();
    for entry in dir_iter {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && is_ron_file(&path) {
            files.push(path);
        }
    }
    files.sort();

    let mut scenarios = Vec::new();
    for file in files {
        let source = file.display().to_string();
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!(
                    "Kon scenario-bestand niet lezen ({}): {err}",
                    file.display()
                );
                continue;
            }
        };

        match ron::from_str::<ScenarioDefinition>(&content) {
            Ok(scenario) => {
                if scenario.id.trim().is_empty() || scenario.name.trim().is_empty() {
                    eprintln!("Scenario-bestand ({source}) mist id of naam");
                    continue;
                }
                scenarios.push(scenario);
            }
            Err(single_err) => match ron::from_str::<Vec<ScenarioDefinition>>(&content) {
                Ok(list) => {
                    eprintln!(
                        "Scenario-bestand ({source}) bevat een lijst; gebruik bij voorkeur 1 bestand per scenario"
                    );
                    scenarios.extend(filter_valid_scenarios(list, &source));
                }
                Err(_) => {
                    eprintln!("Kon scenario-bestand niet parsen ({source}): {single_err}");
                }
            },
        }
    }

    scenarios
}

pub(super) fn load_scenario_catalog(path: &Path) -> ScenarioCatalog {
    let mut scenarios = if path.exists() {
        if path.is_dir() {
            load_scenarios_from_dir(path)
        } else {
            load_scenarios_from_file(path)
        }
    } else if is_ron_file(path) {
        if write_default_scenarios_to_file(path) {
            println!("Scenario-bestand aangemaakt: {}", path.display());
            load_scenarios_from_file(path)
        } else {
            Vec::new()
        }
    } else if write_default_scenarios_to_dir(path) {
        println!("Scenario-map aangemaakt: {}", path.display());
        load_scenarios_from_dir(path)
    } else {
        Vec::new()
    };

    if scenarios.is_empty() {
        if path.is_dir() && write_default_scenarios_to_dir(path) {
            scenarios = load_scenarios_from_dir(path);
        }
    }

    if scenarios.is_empty() {
        eprintln!("Geen geldige scenario's beschikbaar, gebruik ingebouwde fallback.");
        scenarios = default_scenarios();
    }

    ScenarioCatalog { scenarios }
}

pub(super) fn load_persisted_config() -> PersistedConfig {
    let path = Path::new(CONFIG_PATH);

    let Ok(content) = fs::read_to_string(path) else {
        return PersistedConfig::default();
    };

    match ron::from_str::<PersistedConfig>(&content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Kon config niet lezen ({}): {err}", path.display());
            PersistedConfig::default()
        }
    }
}

pub(super) fn save_persisted_config(settings: &GameSettings, keybinds: &GameKeybinds) {
    let persisted = PersistedConfig {
        settings: settings.clone(),
        keybinds: PersistedKeybinds::from_runtime(keybinds),
    };

    let path = Path::new(CONFIG_PATH);
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Kon config-map niet maken ({}): {err}", parent.display());
            return;
        }
    }

    let pretty = ron::ser::PrettyConfig::default();
    let serialized = match ron::ser::to_string_pretty(&persisted, pretty) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Kon config niet serialiseren: {err}");
            return;
        }
    };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!("Kon config niet opslaan ({}): {err}", path.display());
    }
}

pub(super) fn action_matches_filter(action: GameAction, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }

    action
        .label()
        .to_ascii_lowercase()
        .contains(&filter.to_ascii_lowercase())
}

pub(super) fn keycode_to_filter_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

pub(super) fn keycodes_to_names(keys: &[KeyCode]) -> String {
    keys.iter()
        .map(|key| keycode_to_name(*key))
        .collect::<Vec<_>>()
        .join("|")
}

pub(super) fn keycodes_from_names(raw: &str) -> Vec<KeyCode> {
    let mut out = Vec::new();
    for segment in raw.split('|') {
        let key_name = segment.trim();
        if key_name.is_empty() {
            continue;
        }
        if let Some(key) = keycode_from_name(key_name) {
            if !out.contains(&key) {
                out.push(key);
            }
        }
    }
    out
}

pub(super) fn keycode_to_name(key: KeyCode) -> String {
    format!("{key:?}")
}

pub(super) fn keycode_to_label(key: KeyCode) -> String {
    match key {
        KeyCode::KeyA => "A".into(),
        KeyCode::KeyB => "B".into(),
        KeyCode::KeyC => "C".into(),
        KeyCode::KeyD => "D".into(),
        KeyCode::KeyE => "E".into(),
        KeyCode::KeyF => "F".into(),
        KeyCode::KeyG => "G".into(),
        KeyCode::KeyH => "H".into(),
        KeyCode::KeyI => "I".into(),
        KeyCode::KeyJ => "J".into(),
        KeyCode::KeyK => "K".into(),
        KeyCode::KeyL => "L".into(),
        KeyCode::KeyM => "M".into(),
        KeyCode::KeyN => "N".into(),
        KeyCode::KeyO => "O".into(),
        KeyCode::KeyP => "P".into(),
        KeyCode::KeyQ => "Q".into(),
        KeyCode::KeyR => "R".into(),
        KeyCode::KeyS => "S".into(),
        KeyCode::KeyT => "T".into(),
        KeyCode::KeyU => "U".into(),
        KeyCode::KeyV => "V".into(),
        KeyCode::KeyW => "W".into(),
        KeyCode::KeyX => "X".into(),
        KeyCode::KeyY => "Y".into(),
        KeyCode::KeyZ => "Z".into(),
        KeyCode::Digit0 => "0".into(),
        KeyCode::Digit1 => "1".into(),
        KeyCode::Digit2 => "2".into(),
        KeyCode::Digit3 => "3".into(),
        KeyCode::Digit4 => "4".into(),
        KeyCode::Digit5 => "5".into(),
        KeyCode::Digit6 => "6".into(),
        KeyCode::Digit7 => "7".into(),
        KeyCode::Digit8 => "8".into(),
        KeyCode::Digit9 => "9".into(),
        _ => format!("{key:?}"),
    }
}

pub(super) fn keycode_from_name(name: &str) -> Option<KeyCode> {
    match name {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        "Enter" => Some(KeyCode::Enter),
        "Backspace" => Some(KeyCode::Backspace),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "Minus" => Some(KeyCode::Minus),
        "Equal" => Some(KeyCode::Equal),
        "BracketLeft" => Some(KeyCode::BracketLeft),
        "BracketRight" => Some(KeyCode::BracketRight),
        "Semicolon" => Some(KeyCode::Semicolon),
        "Quote" => Some(KeyCode::Quote),
        "Backquote" => Some(KeyCode::Backquote),
        "Backslash" => Some(KeyCode::Backslash),
        "Comma" => Some(KeyCode::Comma),
        "Period" => Some(KeyCode::Period),
        "Slash" => Some(KeyCode::Slash),
        "Escape" => Some(KeyCode::Escape),
        "Insert" => Some(KeyCode::Insert),
        "Delete" => Some(KeyCode::Delete),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}

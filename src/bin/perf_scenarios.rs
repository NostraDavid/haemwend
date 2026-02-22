use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    entities: usize,
    steps: usize,
    complexity: usize,
}

#[derive(Debug)]
struct ScenarioResult {
    name: String,
    entities: usize,
    steps: usize,
    complexity: usize,
    ns_per_step_min: f64,
    ns_per_step_median: f64,
    ns_per_step_max: f64,
    checksum: u64,
}

#[derive(Debug)]
struct Args {
    scenario_file: PathBuf,
    output: PathBuf,
    repeats: usize,
    warmup: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;
    let scenarios = read_scenarios(&args.scenario_file)?;

    if scenarios.is_empty() {
        return Err("Geen scenarios gevonden".into());
    }

    let mut results = Vec::with_capacity(scenarios.len());

    for scenario in &scenarios {
        for _ in 0..args.warmup {
            let _ = run_scenario(scenario);
        }

        let mut samples = Vec::with_capacity(args.repeats);
        let mut checksum = 0_u64;

        for _ in 0..args.repeats {
            let (ns_per_step, run_checksum) = run_scenario(scenario);
            samples.push(ns_per_step);
            checksum ^= run_checksum;
        }

        samples.sort_by(f64::total_cmp);
        let min = samples[0];
        let max = samples[samples.len() - 1];
        let median = samples[samples.len() / 2];

        println!(
            "{name:>18}: median={median:>10.2} ns/step, min={min:>10.2}, max={max:>10.2}",
            name = scenario.name
        );

        results.push(ScenarioResult {
            name: scenario.name.clone(),
            entities: scenario.entities,
            steps: scenario.steps,
            complexity: scenario.complexity,
            ns_per_step_min: min,
            ns_per_step_median: median,
            ns_per_step_max: max,
            checksum,
        });
    }

    write_csv(&args.output, &results)?;
    println!("Resultaten weggeschreven naar {}", args.output.display());

    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let mut scenario_file = PathBuf::from("perf/scenarios.ron");
    let mut output = PathBuf::from("perf_results.csv");
    let mut repeats = 3_usize;
    let mut warmup = 1_usize;

    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scenario-file" => {
                let value = iter.next().ok_or("--scenario-file verwacht een pad")?;
                scenario_file = PathBuf::from(value);
            }
            "--output" => {
                let value = iter.next().ok_or("--output verwacht een pad")?;
                output = PathBuf::from(value);
            }
            "--repeats" => {
                let value = iter.next().ok_or("--repeats verwacht een getal")?;
                repeats = value.parse()?;
            }
            "--warmup" => {
                let value = iter.next().ok_or("--warmup verwacht een getal")?;
                warmup = value.parse()?;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("Onbekende argumenten: {arg}").into());
            }
        }
    }

    if repeats == 0 {
        return Err("--repeats moet >= 1 zijn".into());
    }

    Ok(Args {
        scenario_file,
        output,
        repeats,
        warmup,
    })
}

fn print_help() {
    println!(
        "Gebruik:\n\
         cargo run --release --bin perf_scenarios -- [opties]\n\n\
         Opties:\n\
         --scenario-file <pad>   Scenariobestand (default: perf/scenarios.ron)\n\
         --output <pad>          Output CSV (default: perf_results.csv)\n\
         --repeats <n>           Aantal metingen per scenario (default: 3)\n\
         --warmup <n>            Warmup-runs per scenario (default: 1)"
    );
}

fn read_scenarios(path: &Path) -> Result<Vec<Scenario>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let scenarios = ron::from_str::<Vec<Scenario>>(&content)?;
    Ok(scenarios)
}

fn write_csv(path: &Path, results: &[ScenarioResult]) -> Result<(), Box<dyn Error>> {
    let mut out = String::from(
        "name,entities,steps,complexity,ns_per_step_min,ns_per_step_median,ns_per_step_max,checksum\n",
    );

    for result in results {
        out.push_str(&format!(
            "{name},{entities},{steps},{complexity},{min:.4},{median:.4},{max:.4},{checksum}\n",
            name = result.name,
            entities = result.entities,
            steps = result.steps,
            complexity = result.complexity,
            min = result.ns_per_step_min,
            median = result.ns_per_step_median,
            max = result.ns_per_step_max,
            checksum = result.checksum,
        ));
    }

    fs::write(path, out)?;
    Ok(())
}

fn run_scenario(scenario: &Scenario) -> (f64, u64) {
    let mut pos_x = Vec::with_capacity(scenario.entities);
    let mut pos_y = Vec::with_capacity(scenario.entities);
    let mut vel_x = Vec::with_capacity(scenario.entities);
    let mut vel_y = Vec::with_capacity(scenario.entities);

    let mut seed = 0x9E37_79B9_7F4A_7C15_u64 ^ scenario.entities as u64 ^ scenario.steps as u64;

    for _ in 0..scenario.entities {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        let x = ((seed >> 16) as f32) * (1.0 / u32::MAX as f32);
        let y = ((seed >> 40) as f32) * (1.0 / u32::MAX as f32);

        pos_x.push(x);
        pos_y.push(y);
        vel_x.push((x - 0.5) * 0.1);
        vel_y.push((y - 0.5) * 0.1);
    }

    let start = Instant::now();
    let mut checksum = 0_u64;

    for step in 0..scenario.steps {
        let time_term = (step as f32 * 0.000_1).sin();

        for i in 0..scenario.entities {
            let mut x = pos_x[i];
            let mut y = pos_y[i];
            let mut vx = vel_x[i];
            let mut vy = vel_y[i];

            for inner in 0..scenario.complexity {
                let wobble = ((i + inner + step) as f32 * 0.000_31).cos() * 0.000_7;
                vx += (y * 0.001 + time_term) * 0.1 + wobble;
                vy += (x * 0.001 - time_term) * 0.1 - wobble;

                x += vx;
                y += vy;

                if x > 1.0 || x < -1.0 {
                    vx = -vx * 0.97;
                }
                if y > 1.0 || y < -1.0 {
                    vy = -vy * 0.97;
                }
            }

            pos_x[i] = x;
            pos_y[i] = y;
            vel_x[i] = vx;
            vel_y[i] = vy;
        }

        let probe = step % scenario.entities;
        checksum ^= (pos_x[probe].to_bits() as u64) << 1;
        checksum ^= pos_y[probe].to_bits() as u64;
    }

    let elapsed = start.elapsed();
    let ns_total = elapsed.as_nanos() as f64;
    let ns_per_step = ns_total / scenario.steps as f64;

    (ns_per_step, checksum)
}

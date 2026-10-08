use blobin_f1_cleanroom::diff_images;
use blobin_f2_topology::{
    cpu_mask, cpu_render, gpu, measure_image, measure_mask, Action, Engine, Scene,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn save_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())
}
fn run() -> Result<(), String> {
    let mut backend = "compare".to_string();
    let mut dpr = 1;
    let mut simulate_unavailable = false;
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f2-runs");
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backend" => backend = args.next().ok_or("missing backend")?,
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("missing DPR")?
                    .parse()
                    .map_err(|_| "bad DPR")?
            }
            "--out" => out = PathBuf::from(args.next().ok_or("missing output dir")?),
            "--simulate-gpu-unavailable" => simulate_unavailable = true,
            "--help" => {
                println!("F2 --backend cpu|gpu|auto|compare --dpr 1|2 --out DIR [--simulate-gpu-unavailable]");
                return Ok(());
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if !matches!(backend.as_str(), "cpu" | "gpu" | "auto" | "compare") {
        return Err("invalid backend".into());
    }
    if simulate_unavailable && backend != "auto" {
        return Err("fault injection requires auto".into());
    }
    let mut scene = Scene {
        dpr,
        ..Scene::default()
    };
    scene.validate()?;
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut engine = Engine::new(scene)?;
    let actions = [
        None,
        Some(Action::Merge),
        Some(Action::PunchHole),
        Some(Action::SealHole),
        Some(Action::Split),
    ];
    let mut ledger=String::from("step\tmode\tepoch\tleft_id\tright_id\tsemantic_digest\tcomponents\tholes\tselected_backend\tmax_channel\tmean_abs\n");
    let mut events=String::from("step\taction\tepoch_before\tepoch_after\tcomponents_before\tcomponents_after\tholes_before\tholes_after\tsemantic_digest_before\tsemantic_digest_after\n");
    for (step, action) in actions.iter().enumerate() {
        if let Some(action) = action {
            let event = engine.transition(*action)?;
            events.push_str(&format!(
                "{step}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\n",
                action.name(),
                event.from_epoch,
                event.to_epoch,
                event.before.components,
                event.after.components,
                event.before.holes,
                event.after.holes,
                event.before_digest,
                event.after_digest
            ));
        }
        scene = engine.scene;
        let canonical = measure_mask(&cpu_mask(scene)?, scene.width(), scene.height())?;
        if canonical != scene.layout.expected_topology() {
            return Err(format!(
                "F2_GROUND_TRUTH_MISMATCH step={step} expected={:?} actual={canonical:?}",
                scene.layout.expected_topology()
            ));
        }
        let cpu = cpu_render(scene)?;
        let cpu_topo = measure_image(&cpu, scene.width(), scene.height())?;
        if cpu_topo != canonical {
            return Err(format!(
                "F2_CPU_TOPOLOGY_MISMATCH step={step} cpu={cpu_topo:?} canonical={canonical:?}"
            ));
        }
        save_png(
            &out.join(format!("step{step}-cpu-dpr{dpr}.png")),
            scene.width(),
            scene.height(),
            &cpu,
        )?;
        let mut selected = "cpu";
        let mut max = 0u8;
        let mut mean = 0.0;
        if backend != "cpu" && !simulate_unavailable {
            match gpu::render(scene) {
                Ok((rgba, adapter)) => {
                    let observed = measure_image(&rgba, scene.width(), scene.height())?;
                    if observed != canonical {
                        return Err(format!("F2_GPU_TOPOLOGY_MISMATCH step={step} gpu={observed:?} canonical={canonical:?}"));
                    }
                    let (stats, delta) = diff_images(&cpu, &rgba)?;
                    max = stats.max_channel_error;
                    mean = stats.mean_abs_error;
                    save_png(
                        &out.join(format!("step{step}-gpu-dpr{dpr}.png")),
                        scene.width(),
                        scene.height(),
                        &rgba,
                    )?;
                    save_png(
                        &out.join(format!("step{step}-diff-dpr{dpr}.png")),
                        scene.width(),
                        scene.height(),
                        &delta,
                    )?;
                    println!("F2_GPU step={step} adapter={adapter}");
                    if max > 6 || mean > 0.15 {
                        return Err(format!(
                            "F2_GPU_CPU_PARITY_MISMATCH step={step} max={max} mean={mean:.6}"
                        ));
                    }
                    selected = "gpu";
                }
                Err(err) if backend == "auto" => {
                    println!("F2_CPU_RECOVERY step={step} reason={err}");
                }
                Err(err) => return Err(format!("GPU_REQUIRED step={step} {err}")),
            }
        } else if simulate_unavailable {
            println!("F2_FAULT_INJECTED step={step} selected=cpu");
        }
        ledger.push_str(&format!(
            "{step}\t{:?}\t{}\t{}\t{}\t{:016x}\t{}\t{}\t{}\t{}\t{:.6}\n",
            scene.layout,
            scene.epoch,
            scene.left_id,
            scene.right_id,
            scene.digest(),
            canonical.components,
            canonical.holes,
            selected,
            max,
            mean
        ));
        println!("F2_STEP step={step} mode={:?} epoch={} components={} holes={} ids={}:{} digest={:016x} backend={selected} max_diff={max} mae={mean:.6}",
            scene.layout,scene.epoch,canonical.components,canonical.holes,scene.left_id,scene.right_id,scene.digest());
    }
    fs::write(out.join(format!("ledger-dpr{dpr}.tsv")), ledger).map_err(|e| e.to_string())?;
    fs::write(out.join(format!("events-dpr{dpr}.tsv")), events).map_err(|e| e.to_string())?;
    println!(
        "F2_TRACE_PASS steps=5 events=4 backend={backend} dpr={dpr} out={}",
        out.display()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("F2_ERROR {e}");
        std::process::exit(1);
    }
}

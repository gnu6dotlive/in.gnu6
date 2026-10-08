use blobin_f1_cleanroom::diff_images;
use blobin_f2_topology::{Action, Engine, Scene};
use blobin_f3_material_height::{cpu_render, gpu, Material, Params};
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
fn max_abs(a: &[f32], b: &[f32]) -> Result<f32, String> {
    if a.len() != b.len() {
        return Err("F3 ANALYTIC LENGTH MISMATCH".into());
    }
    Ok(a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max))
}
fn run() -> Result<(), String> {
    let mut backend = "compare".to_string();
    let mut dpr = 1u32;
    let mut heat = 0.62f32;
    let mut inject = false;
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f3-runs");
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
            "--heat" => {
                heat = args
                    .next()
                    .ok_or("missing heat")?
                    .parse()
                    .map_err(|_| "bad heat")?
            }
            "--out" => out = PathBuf::from(args.next().ok_or("missing output")?),
            "--simulate-gpu-unavailable" => inject = true,
            "--help" => {
                println!("--backend cpu|gpu|auto|compare --dpr 1|2 --heat 0..1 [--out DIR] [--simulate-gpu-unavailable]");
                return Ok(());
            }
            _ => return Err(format!("unknown flag {arg}")),
        }
    }
    if !matches!(backend.as_str(), "cpu" | "gpu" | "auto" | "compare") {
        return Err("bad backend".into());
    }
    if inject && backend != "auto" {
        return Err("only auto permits injected failure".into());
    }
    let scene = Scene {
        dpr,
        ..Scene::default()
    };
    scene.validate()?;
    Params {
        heat_strength: heat,
        ..Default::default()
    }
    .validate()?;
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut engine = Engine::new(scene)?;
    let actions = [
        None,
        Some(Action::Merge),
        Some(Action::PunchHole),
        Some(Action::SealHole),
        Some(Action::Split),
    ];
    let mut ledger=String::from("step\tmaterial\tdpr\tepoch\tdigest\tcomponents\tholes\tbackend\tmax_rgb\tmean_rgb\tmax_height\tmax_heat\tmax_normal_z\n");
    let mut qualified = 0usize;
    let mut worst_rgb = 0u8;
    let mut worst_fields = 0.0f32;
    for (step, action) in actions.iter().enumerate() {
        if let Some(a) = action {
            engine.transition(*a)?;
        }
        let s = engine.scene;
        let canonical = blobin_f2_topology::cpu_mask(s)?;
        let topology = blobin_f2_topology::measure_mask(&canonical, s.width(), s.height())?;
        if topology != s.layout.expected_topology() {
            return Err(format!(
                "F3_F2_TOPOLOGY_ERROR step={step} observed={topology:?}"
            ));
        }
        for material in Material::ALL {
            let p = Params {
                material,
                heat_strength: heat,
            };
            let cpu = cpu_render(s, p)?;
            if cpu.semantic_digest != s.digest() {
                return Err("semantic digest changed".into());
            }
            for (index, (a, b)) in cpu.mask.iter().zip(&canonical).enumerate() {
                if *a != u32::from(*b) {
                    return Err(format!("F3_CPU_MASK_DRIFT step={step} px={index}"));
                }
            }
            let name = format!("s{step}-{}-dpr{dpr}", material.name());
            save_png(
                &out.join(format!("{name}-cpu.png")),
                s.width(),
                s.height(),
                &cpu.rgba,
            )?;
            let mut selected = "cpu";
            let mut max_rgb = 0u8;
            let mut mean_rgb = 0.0f64;
            let mut max_height = 0.0f32;
            let mut max_heat = 0.0f32;
            let mut max_normal = 0.0f32;
            if backend != "cpu" && !inject {
                match gpu::render(s, p) {
                    Ok(gpu) => {
                        if gpu.mask.len() != canonical.len() {
                            return Err("F3 GPU mask length mismatch".into());
                        }
                        for (i, (g, b)) in gpu.mask.iter().zip(&canonical).enumerate() {
                            if *g != u32::from(*b) {
                                return Err(format!(
                                    "F3_GPU_TOPOLOGY_DRIFT step={step} material={} px={i}",
                                    material.name()
                                ));
                            }
                        }
                        let (stats, difference) = diff_images(&cpu.rgba, &gpu.rgba)?;
                        max_rgb = stats.max_channel_error;
                        mean_rgb = stats.mean_abs_error;
                        max_height = max_abs(&cpu.height, &gpu.height)?;
                        max_heat = max_abs(&cpu.heat, &gpu.heat)?;
                        max_normal = max_abs(&cpu.normal_z, &gpu.normal_z)?;
                        save_png(
                            &out.join(format!("{name}-gpu.png")),
                            s.width(),
                            s.height(),
                            &gpu.rgba,
                        )?;
                        save_png(
                            &out.join(format!("{name}-diff.png")),
                            s.width(),
                            s.height(),
                            &difference,
                        )?;
                        // Thresholds frozen before qualification; a failure must remain a failure.
                        if max_rgb > 8
                            || mean_rgb > 0.35
                            || max_height > 0.003
                            || max_heat > 0.003
                            || max_normal > 0.003
                        {
                            return Err(format!("F3_PARITY_RED step={step} material={} rgbmax={max_rgb} mean={mean_rgb:.6} h={max_height:.6} heat={max_heat:.6} nz={max_normal:.6}",material.name()));
                        }
                        worst_rgb = worst_rgb.max(max_rgb);
                        worst_fields = worst_fields.max(max_height).max(max_heat).max(max_normal);
                        selected = "gpu";
                        if step == 0 && material == Material::Ceramic {
                            println!("F3_ADAPTER {}", gpu.adapter);
                        }
                    }
                    Err(error) if backend == "auto" => {
                        println!(
                            "F3_CPU_RECOVERY step={step} material={} {error}",
                            material.name()
                        );
                    }
                    Err(error) => return Err(format!("F3_GPU_REQUIRED {error}")),
                }
            } else if inject {
                println!(
                    "F3_INJECTED_PRE_DISPATCH step={step} material={} selected=cpu",
                    material.name()
                );
            }
            ledger.push_str(&format!("{step}\t{}\t{dpr}\t{}\t{:016x}\t{}\t{}\t{selected}\t{max_rgb}\t{mean_rgb:.6}\t{max_height:.7}\t{max_heat:.7}\t{max_normal:.7}\n",
                material.name(),s.epoch,s.digest(),topology.components,topology.holes));
            println!("F3_STEP step={step} material={} components={} holes={} backend={selected} max_rgb={max_rgb} mean_rgb={mean_rgb:.6} max_height={max_height:.6} max_heat={max_heat:.6} max_normal={max_normal:.6}",material.name(),topology.components,topology.holes);
            qualified += 1;
        }
    }
    fs::write(out.join(format!("material-ledger-dpr{dpr}.tsv")), ledger)
        .map_err(|e| e.to_string())?;
    println!("F3_PASS qualified={qualified} layouts=5 materials=4 dpr={dpr} backend={backend} max_rgb={worst_rgb} max_field={worst_fields:.7} out={}",out.display());
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("F3_ERROR {error}");
        std::process::exit(1);
    }
}

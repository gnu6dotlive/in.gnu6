use blobin_f1_cleanroom::diff_images;
use blobin_f2_topology::qualification_trace;
use blobin_f3_material_height::{Material, Params};
use blobin_f4_effects::{gpu, process, Plan, Rect, Temporal};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())
}
fn execute() -> Result<(), String> {
    let mut backend = "compare".to_string();
    let mut dpr = 1u32;
    let mut reduced = false;
    let mut inject = false;
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f4-runs");
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backend" => backend = args.next().ok_or("backend value missing")?,
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("DPR value missing")?
                    .parse()
                    .map_err(|_| "bad DPR")?
            }
            "--out" => out = PathBuf::from(args.next().ok_or("output path missing")?),
            "--reduced-motion" => reduced = true,
            "--simulate-gpu-unavailable" => inject = true,
            "--help" => {
                println!("--backend compare|cpu|gpu|auto --dpr 1|2 --out DIR [--reduced-motion] [--simulate-gpu-unavailable]");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if !matches!(backend.as_str(), "compare" | "cpu" | "gpu" | "auto") {
        return Err("unsupported backend".into());
    }
    if inject && backend != "auto" {
        return Err("fault injection is only for auto".into());
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut ledger=String::from("state\tmaterial\tframe\tdpr\tepoch\tdigest\tcomponents\tholes\tbackend\tdispatches\tmax_rgb\tmean_rgb\treset_cpu\treset_gpu\n");
    let mut qualified = 0usize;
    let mut gpu_count = 0usize;
    let mut worst_rgb = 0u8;
    let mut worst_mean = 0.0f64;
    for (step, mut scene) in qualification_trace()?.into_iter().enumerate() {
        scene.dpr = dpr;
        scene.validate()?;
        for material in Material::ALL {
            let mut cpu_t = Temporal::default();
            let mut gpu_t = gpu::GpuTemporal::default();
            for frame in 0..2 {
                // F4 echo proof: frame 1 changes heat while scene identity remains constant.
                let params = Params {
                    material,
                    heat_strength: if frame == 0 { 0.62 } else { 0.18 },
                };
                let plan = Plan {
                    reduced_motion: reduced,
                    dirty: Rect {
                        x: 0,
                        y: 0,
                        w: scene.width(),
                        h: scene.height(),
                    },
                    ..Plan::default()
                };
                let cpu = process(scene, params, &plan, &mut cpu_t)?;
                let kind = format!("state{step}-{}-frame{frame}-dpr{dpr}", material.name());
                png(
                    &out.join(format!("{kind}-cpu.png")),
                    scene.width(),
                    scene.height(),
                    &cpu.rgba,
                )?;
                let mut chosen = "cpu";
                let mut dispatches = 0usize;
                let mut max_rgb = 0u8;
                let mut mean_rgb = 0.0f64;
                if backend != "cpu" && !inject {
                    match gpu::process(scene, params, &plan, &mut gpu_t) {
                        Ok(g) => {
                            if g.digest != cpu.semantic_digest
                                || (g.topology.components, g.topology.holes)
                                    != (cpu.components, cpu.holes)
                            {
                                return Err(format!(
                                    "F4_SEMANTIC_DRIFT step={step} material={:?}",
                                    material
                                ));
                            }
                            let (stats, diff) = diff_images(&cpu.rgba, &g.rgba)?;
                            max_rgb = stats.max_channel_error;
                            mean_rgb = stats.mean_abs_error;
                            // Fixed before first device parity; fidelity bounds refer to RGBA RGB channels.
                            if max_rgb > 12 || mean_rgb > 0.5 {
                                return Err(format!("F4_PARITY_RED state={step} material={:?} frame={frame} max={max_rgb} mean={mean_rgb:.6}",material));
                            }
                            png(
                                &out.join(format!("{kind}-gpu.png")),
                                scene.width(),
                                scene.height(),
                                &g.rgba,
                            )?;
                            png(
                                &out.join(format!("{kind}-diff.png")),
                                scene.width(),
                                scene.height(),
                                &diff,
                            )?;
                            chosen = "gpu";
                            dispatches = g.dispatches;
                            gpu_count += 1;
                            worst_rgb = worst_rgb.max(max_rgb);
                            worst_mean = worst_mean.max(mean_rgb);
                            if step == 0 && frame == 0 && material == Material::Ceramic {
                                println!(
                                    "F4_ADAPTER {} pass_dispatches={}",
                                    g.adapter, g.dispatches
                                );
                            }
                            if reduced && (g.temporal_bytes != 0 || cpu.temporal_bytes != 0) {
                                return Err("F4_REDUCED_MOTION_TEMPORAL_LEAK".into());
                            }
                        }
                        Err(e) if backend == "auto" => {
                            println!("F4_CPU_RECOVERY state={step} material={:?} frame={frame} cause={e}",material);
                        }
                        Err(e) => return Err(format!("F4_GPU_REQUIRED {e}")),
                    }
                } else if inject {
                    println!(
                        "F4_INJECTED_PRE_DISPATCH state={step} material={:?} frame={frame}",
                        material
                    );
                }
                ledger.push_str(&format!(
                    "{step}\t{}\t{frame}\t{dpr}\t{}\t{:016x}\t{}\t{}\t{}\t{}\t{}\t{:.6}\t{}\t{}\n",
                    material.name(),
                    scene.epoch,
                    scene.digest(),
                    cpu.components,
                    cpu.holes,
                    chosen,
                    dispatches,
                    max_rgb,
                    mean_rgb,
                    cpu_t.resets,
                    gpu_t.resets
                ));
                qualified += 1;
            }
        }
    }
    fs::write(
        out.join(format!("ledger-dpr{dpr}-reduced{reduced}.tsv")),
        ledger,
    )
    .map_err(|e| e.to_string())?;
    println!("F4_PASS frames={qualified} gpu_frames={gpu_count} dpr={dpr} reduced={reduced} backend={backend} max_rgb={worst_rgb} max_mean={worst_mean:.6} output={}",out.display());
    Ok(())
}
fn main() {
    if let Err(error) = execute() {
        eprintln!("F4_ERROR {error}");
        std::process::exit(1);
    }
}

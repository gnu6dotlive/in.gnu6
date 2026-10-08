use blobin_f1_cleanroom::{bytes_digest, cpu_render, diff_images, FrameIr};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{env, fs};

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let file = fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let mut backend = "compare".to_string();
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f1-runs");
    let mut ir = FrameIr::default();
    let mut simulate_loss = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backend" => backend = args.next().ok_or("missing backend value")?,
            "--out" => out = PathBuf::from(args.next().ok_or("missing output directory")?),
            "--dpr" => {
                ir.dpr = args
                    .next()
                    .ok_or("missing DPR")?
                    .parse()
                    .map_err(|_| "invalid DPR")?;
            }
            "--simulate-gpu-loss" => simulate_loss = true,
            "--help" | "-h" => {
                println!("blobin-f1 --backend cpu|gpu|auto|compare [--dpr 1|2] [--out DIR] [--simulate-gpu-loss]");
                return Ok(());
            }
            other => return Err(format!("unsupported argument {other}")),
        }
    }
    ir.validate()?;
    if !matches!(backend.as_str(), "cpu" | "gpu" | "auto" | "compare") {
        return Err(format!("unsupported backend {backend}"));
    }
    if simulate_loss && backend != "auto" {
        return Err("fault injection is accepted only in auto fallback mode".into());
    }
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let begin = Instant::now();
    let cpu = cpu_render(&ir)?;
    let cpu_time = begin.elapsed();
    let mut selected = "cpu";
    let mut gpu_info = "NOT_RUN".to_owned();
    let mut stats = None;
    let mut gpu_time = None;
    match backend.as_str() {
        "cpu" => {}
        "auto" if simulate_loss => {
            gpu_info = "INJECTED_PRE_DISPATCH_UNAVAILABLE / CPU_RECOVERED".into();
        }
        "auto" | "gpu" | "compare" => {
            let gpu_start = Instant::now();
            match blobin_f1_cleanroom::gpu::render(&ir) {
                Ok((gpu, label)) => {
                    gpu_time = Some(gpu_start.elapsed());
                    gpu_info = label;
                    selected = "gpu";
                    write_png(
                        &out.join(format!("gpu-dpr{}.png", ir.dpr)),
                        ir.pixel_width(),
                        ir.pixel_height(),
                        &gpu,
                    )?;
                    let (measure, delta) = diff_images(&cpu, &gpu)?;
                    write_png(
                        &out.join(format!("diff-dpr{}.png", ir.dpr)),
                        ir.pixel_width(),
                        ir.pixel_height(),
                        &delta,
                    )?;
                    stats = Some(measure);
                }
                Err(err) => {
                    if backend == "auto" {
                        gpu_info = format!("GPU_UNAVAILABLE ({err}) / CPU_RECOVERED");
                        selected = "cpu";
                    } else {
                        return Err(format!("GPU_REQUIRED: {err}"));
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    write_png(
        &out.join(format!("cpu-dpr{}.png", ir.dpr)),
        ir.pixel_width(),
        ir.pixel_height(),
        &cpu,
    )?;
    println!("F1_FRAME logical={}x{} dpr={} physical={}x{} entity_id={} epoch={} frame_semantic_digest={:016x}",
        ir.logical_width, ir.logical_height, ir.dpr, ir.pixel_width(), ir.pixel_height(),
        ir.entity_id, ir.topology_epoch, ir.semantic_digest());
    println!(
        "F1_CPU pixels={} rgba_digest={:016x} render_wall_ms={:.3}",
        ir.pixel_width() * ir.pixel_height(),
        bytes_digest(&cpu),
        cpu_time.as_secs_f64() * 1000.0
    );
    println!("F1_BACKEND requested={backend} selected={selected} adapter={gpu_info}");
    if let Some(time) = gpu_time {
        println!(
            "F1_GPU init_compile_dispatch_readback_wall_ms={:.3}",
            time.as_secs_f64() * 1000.0
        );
    }
    if let Some(stats) = stats {
        println!("F1_DIFF max_channel={} mean_abs={:.6} changed_channels={} differing_pixels={} total_pixels={}",
            stats.max_channel_error, stats.mean_abs_error, stats.changed_channels,
            stats.differing_pixels, stats.total_pixels);
        // F1 qualification budget registered before first GPU run. Tighten in a later gate,
        // never reinterpret a miss as success.
        if stats.max_channel_error > 6 || stats.mean_abs_error > 0.15 {
            return Err("F1_PIXEL_PARITY_GATE_FAIL: max<=6 && mean<=0.15".into());
        }
        println!("F1_PIXEL_PARITY_GATE_PASS max<=6 mean<=0.15");
    }
    println!("F1_SUCCESS output={}", out.display());
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("F1_ERROR {e}");
        std::process::exit(1);
    }
}

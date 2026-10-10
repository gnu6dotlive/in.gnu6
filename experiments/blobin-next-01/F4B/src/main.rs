use blobin_f2_topology::Scene;
use blobin_f3_material_height::Params;
use blobin_f4_effects::{process, Plan, Rect, Temporal};
use blobin_f4b_resilience::{gpu::Resident, overlay, validate_damage, Damage};
use std::{env, fs, path::PathBuf, time::Instant};

fn run() -> Result<(), String> {
    let mut dpr = 1;
    let mut samples = 24usize;
    let mut mode = "compare".to_string();
    let mut reduced = false;
    let mut destroy = false;
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f4b-evidence");
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("missing dpr")?
                    .parse()
                    .map_err(|_| "invalid dpr")?
            }
            "--samples" => {
                samples = args
                    .next()
                    .ok_or("missing samples")?
                    .parse()
                    .map_err(|_| "invalid samples")?
            }
            "--mode" => mode = args.next().ok_or("missing mode")?,
            "--out" => out = PathBuf::from(args.next().ok_or("missing output")?),
            "--reduced-motion" => reduced = true,
            "--destroy-owned-device" => destroy = true,
            "--help" => {
                println!("--mode compare|auto|gpu|cpu --dpr 1|2 --samples 8..64 [--reduced-motion] [--destroy-owned-device] --out DIR");
                return Ok(());
            }
            _ => return Err(format!("unknown arg: {arg}")),
        }
    }
    if !(8..=64).contains(&samples) {
        return Err("F4B SAMPLE_BUDGET".into());
    }
    if !matches!(mode.as_str(), "compare" | "auto" | "gpu" | "cpu") {
        return Err("F4B INVALID_MODE".into());
    }
    let scene = Scene {
        dpr,
        ..Scene::default()
    };
    scene.validate()?;
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
    let baseline = process(scene, Params::default(), &plan, &mut Temporal::default())?;
    let semantic = baseline.semantic_digest;
    let mut current = baseline.rgba.clone();
    let mut resident = if mode == "cpu" {
        None
    } else {
        match Resident::new(scene.width(), scene.height(), &current) {
            Ok(device) => Some(device),
            Err(error) if mode == "auto" => {
                println!("F4B_CPU_FALLBACK adapter_unavailable={error}");
                None
            }
            Err(error) => return Err(format!("F4B_GPU_REQUIRED {error}")),
        }
    };
    let mut durations = Vec::new();
    let mut rows=String::from("sample\tdpr\treduced\tbackend\tdamage_x\tdamage_y\tdamage_w\tdamage_h\tchanged_pixels\texplicit_submits\tworkgroups\twall_ms\tsemantic_digest\n");
    let mut idle_count = 0usize;
    let mut changed_total = 0usize;
    let mut gpu_patches = 0usize;
    let rect = Damage {
        x: 23 * dpr,
        y: 19 * dpr,
        w: 12 * dpr,
        h: 10 * dpr,
    };
    let adapter = resident
        .as_ref()
        .map_or("CPU_ONLY".to_string(), |r| r.adapter.clone());
    println!(
        "F4B_ADAPTER {adapter} scene={}x{} dpr={dpr}",
        scene.width(),
        scene.height()
    );
    for iteration in 0..samples {
        // Idle is evaluated BEFORE CPU-side overlay changes. No explicit GPU queue.submit or encoder.
        if let Some(ref gpu) = resident {
            let submitted = gpu.explicit_submissions;
            let groups = gpu.dispatched_workgroups;
            if gpu.idle()? != current {
                return Err("F4B_IDLE_FRAME_CHANGED".into());
            }
            if gpu.explicit_submissions != submitted || gpu.dispatched_workgroups != groups {
                return Err("F4B_IDLE_GPU_WORK".into());
            }
        }
        idle_count += 1;
        let next = overlay(
            &current,
            scene.width(),
            scene.height(),
            rect,
            (iteration % 17 + 1) as u8,
        )?;
        let changed = validate_damage(&current, &next, scene.width(), scene.height(), rect)?;
        changed_total += changed;
        // Every pixel outside the declared region is unmodified, verified before GPU patch.
        let begin = Instant::now();
        let (backend, submits, workgroups) = if let Some(ref mut gpu) = resident {
            let result = gpu.patch(&next, rect)?;
            if result != next {
                return Err("F4B_GPU_ROI_MISMATCH".into());
            }
            gpu_patches += 1;
            ("gpu", gpu.explicit_submissions, gpu.dispatched_workgroups)
        } else {
            ("cpu", 0, 0)
        };
        let elapsed = begin.elapsed().as_secs_f64() * 1000.0;
        if backend == "gpu" {
            durations.push(elapsed);
        }
        current = next;
        rows.push_str(&format!("{iteration}\t{dpr}\t{reduced}\t{backend}\t{}\t{}\t{}\t{}\t{changed}\t{submits}\t{workgroups}\t{elapsed:.6}\t{semantic:016x}\n",
            rect.x,rect.y,rect.w,rect.h));
    }
    // Explicit wrong damage cannot be accepted, and must not alter GPU state.
    let too_small = Damage {
        x: rect.x + 1,
        y: rect.y + 1,
        w: rect.w - 2,
        h: rect.h - 2,
    };
    let changed = overlay(&current, scene.width(), scene.height(), rect, 5)?;
    if validate_damage(&current, &changed, scene.width(), scene.height(), too_small).is_ok() {
        return Err("F4B_NEGATIVE_UNDERDECLARED_GATE_RED".into());
    }
    let before_destroy = current.clone();
    let mut destroy_path = "NOT_REQUESTED";
    if destroy {
        if let Some(ref mut gpu) = resident {
            let prior = gpu.explicit_submissions;
            gpu.destroy_owned_device(); // application-controlled destruction, not physical loss
            if gpu.patch(&changed, rect).is_ok() {
                return Err("F4B_DESTROYED_GPU_WAS_USED".into());
            }
            if gpu.explicit_submissions != prior {
                return Err("F4B_DESTROYED_DEVICE_SUBMITTED".into());
            }
            destroy_path = "DESTROYED_AND_FAIL_CLOSED";
        } else {
            destroy_path = "NO_GPU_FOR_CONTROLLED_DESTRUCTION";
        }
        // Exact CPU continuity of the same semantic presentation stream.
        current = changed;
        if scene.digest() != semantic {
            return Err("F4B_CPU_SEMANTIC_IDENTITY_LOST".into());
        }
        if let Some(old) = resident.take() {
            // GPU no longer used: host side now owns the authoritative frame.
            drop(old);
            match Resident::new(scene.width(), scene.height(), &current) {
                Ok(mut acquired) => {
                    let third = overlay(&current, scene.width(), scene.height(), rect, 7)?;
                    let gpu_result = acquired.patch(&third, rect)?;
                    if gpu_result != third {
                        return Err("F4B_REACQUIRED_GPU_PARITY_RED".into());
                    }
                    current = third;
                    drop(acquired);
                    destroy_path = "CONTROLLED_DESTROY_CPU_CONTINUITY_GPU_REACQUIRED";
                }
                Err(_) => {
                    destroy_path = "CONTROLLED_DESTROY_CPU_RECOVERY_GPU_REACQUIRE_UNAVAILABLE";
                }
            }
        }
    }
    let mut sorted = durations.clone();
    sorted.sort_by(f64::total_cmp);
    let percentile = |fraction: f64| {
        if sorted.is_empty() {
            None
        } else {
            Some(
                sorted[(((sorted.len() as f64 * fraction).ceil() as usize).saturating_sub(1))
                    .min(sorted.len() - 1)],
            )
        }
    };
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let id = format!("dpr{dpr}-reduced{reduced}-mode{mode}-destroy{destroy}");
    let ledger = out.join(format!("{id}.tsv"));
    fs::write(&ledger, rows).map_err(|e| e.to_string())?;
    let summary=format!("F4B_PASS samples={samples} idle_checks={idle_count} gpu_patches={gpu_patches} changed_pixels={changed_total} dpr={dpr} reduced={reduced} mode={mode} semantic={semantic:016x} p50_cpu_observed_gpu_roi_end_to_end_ms={:?} p95_cpu_observed_gpu_roi_end_to_end_ms={:?} destruction={destroy_path} final_bytes={} prior_bytes={} out={}\n",
        percentile(0.5),percentile(0.95),current.len(),before_destroy.len(),ledger.display());
    print!("{summary}");
    fs::write(out.join(format!("{id}.summary.txt")), &summary).map_err(|e| e.to_string())?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("F4B_ERROR {error}");
        std::process::exit(1);
    }
}

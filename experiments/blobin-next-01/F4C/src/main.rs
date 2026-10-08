use blobin_f2_topology::{cpu_mask, qualification_trace};
use blobin_f3_material_height::{cpu_render, Material, Params};
use blobin_f4b_resilience::{overlay, Damage};
use blobin_f4c_incremental::{area, empty, full, gpu::Graph, Options, SceneKey};
use std::{env, fs, path::PathBuf, time::Instant};

fn run() -> Result<(), String> {
    let mut dpr = 1u32;
    let mut reduced = false;
    let mut out = PathBuf::from("/mnt/workbench/build/blobin-f4c-evidence");
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("missing dpr")?
                    .parse()
                    .map_err(|_| "bad dpr")?
            }
            "--reduced-motion" => reduced = true,
            "--out" => out = PathBuf::from(args.next().ok_or("missing output")?),
            "--help" => {
                println!("F4C --dpr 1|2 [--reduced-motion] [--out DIR]");
                return Ok(());
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut ledger=String::from("state\tmaterial\tdpr\treduced\tframe\tstage0_px\tstage1_px\tstage2_px\tstage3_px\tfull_px\tdispatches\tworkgroups\tsubmissions\tmax_channel\twall_ms\tdigest\n");
    let mut cases = 0usize;
    let mut roi_less_than_full = 0usize;
    let mut idle_proofs = 0usize;
    let mut max_channel = 0u8;
    let mut max_ratio = 0.0f64;
    for (state, mut scene) in qualification_trace()?.into_iter().enumerate() {
        scene.dpr = dpr;
        scene.validate()?;
        let key = SceneKey {
            width: scene.width(),
            height: scene.height(),
            digest: scene.digest(),
        };
        let mask = cpu_mask(scene)?;
        for material in Material::ALL {
            let params = Params {
                material,
                heat_strength: 0.62,
            };
            let source = cpu_render(scene, params)?.rgba;
            let mut gpu = Graph::new(
                key,
                Options {
                    radius: 3,
                    reduced_motion: reduced,
                },
                &source,
                &mask,
            )?;
            let initial_start = Instant::now();
            let first = gpu.update(&source, full(key))?;
            let elapsed = initial_start.elapsed().as_secs_f64() * 1000.0;
            if first.dispatches != 4 || first.regions.iter().any(|r| *r != full(key)) {
                return Err("F4C INITIAL FULL GRAPH GATE".into());
            }
            println!("F4C_INITIAL state={state} material={} dpr={dpr} adapter={} workgroups={} wall_ms={elapsed:.3}",
                material.name(),gpu.adapter,first.workgroups);
            let modified = Damage {
                x: 17 * dpr,
                y: 23 * dpr,
                w: 12 * dpr,
                h: 9 * dpr,
            };
            let next = overlay(&source, key.width, key.height, modified, 37)?;
            let started = Instant::now();
            let second = gpu.update(&next, modified)?;
            let elapsed = started.elapsed().as_secs_f64() * 1000.0;
            let final_ref = blobin_f4c_incremental::reference(
                key,
                Options {
                    radius: 3,
                    reduced_motion: reduced,
                },
                &next,
                &mask,
                if reduced {
                    None
                } else {
                    Some(&first.stages[3])
                },
            )?;
            for (i, reference_stage) in final_ref.iter().enumerate() {
                let error = second.stages[i]
                    .iter()
                    .zip(reference_stage)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap_or(0);
                max_channel = max_channel.max(error);
                if error > 2 {
                    return Err(format!("F4C STAGE PROOF RED stage={i} max={error}"));
                }
            }
            if second.regions[0] != modified {
                return Err("F4C REGION STAGE0 RED".into());
            }
            if area(second.regions[1]) >= area(full(key)) {
                return Err("F4C BLUR NOT REGIONAL".into());
            }
            if area(second.regions[2]) >= area(full(key)) {
                return Err("F4C BLOOM NOT REGIONAL".into());
            }
            if reduced && area(second.regions[3]) >= area(full(key)) {
                return Err("F4C ECHO NOT LOCAL IN REDUCED MODE".into());
            }
            let regions = second.regions.map(area);
            let total: u64 = regions.iter().sum();
            max_ratio = max_ratio.max(total as f64 / (4 * area(full(key))) as f64);
            if total < 4 * area(full(key)) {
                roi_less_than_full += 1;
            }
            let n = key.width as u64 * key.height as u64;
            ledger.push_str(&format!("{state}\t{}\t{dpr}\t{reduced}\tlocal\t{}\t{}\t{}\t{}\t{n}\t{}\t{}\t{}\t{}\t{elapsed:.6}\t{:016x}\n",
                material.name(),regions[0],regions[1],regions[2],regions[3],
                second.dispatches,second.workgroups,second.submissions,max_channel,key.digest));
            // A dirty-free reduced-motion frame needs no submission at all.
            if reduced {
                let prior = gpu.submissions;
                let idled = gpu.update(&next, empty())?;
                if idled.dispatches != 0 || idled.workgroups != 0 || gpu.submissions != prior {
                    return Err("F4C DIRTY FREE IDLE SUBMITTED".into());
                }
                idle_proofs += 1;
            } else {
                // Temporal echo changes even without source mutations. Spatial stages must stay idle.
                let temporal = gpu.update(&next, empty())?;
                if temporal.dispatches != 1
                    || temporal.regions[0] != empty()
                    || temporal.regions[1] != empty()
                    || temporal.regions[2] != empty()
                    || temporal.regions[3] != full(key)
                {
                    return Err("F4C TEMPORAL ECHO HAZARD".into());
                }
            }
            cases += 1;
        }
    }
    let filename = format!("f4c-dpr{dpr}-reduced{reduced}.tsv");
    fs::write(out.join(&filename), ledger).map_err(|e| e.to_string())?;
    println!("F4C_PASS cases={cases} dpr={dpr} reduced={reduced} local_effect_cases={roi_less_than_full} idle_proofs={idle_proofs} max_channel={max_channel} max_dispatch_area_ratio={max_ratio:.6} out={}",out.join(filename).display());
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("F4C_ERROR {e}");
        std::process::exit(1);
    }
}

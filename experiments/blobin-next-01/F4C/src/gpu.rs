//! Stateful Vulkan graph: stage buffers survive updates; GPU dispatch is limited to each pass' causal ROI.
//! Full-frame source uploads and stage readbacks are intentional qualification overhead.
use crate::{full, invalidations, reference, verify_damage, Options, SceneKey};
use blobin_f4b_resilience::Damage;
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;
pub struct ResultFrame {
    pub stages: [Vec<u8>; 4],
    pub regions: [Damage; 4],
    pub workgroups: u64,
    pub dispatches: u32,
    pub submissions: u64,
    pub source_upload_bytes: u64,
    pub readback_bytes: u64,
}
pub struct Graph {
    key: SceneKey,
    options: Options,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    source: wgpu::Buffer,
    mask: wgpu::Buffer,
    history: wgpu::Buffer,
    stage: [wgpu::Buffer; 4],
    mask_cpu: Vec<u8>,
    prev_source: Vec<u8>,
    previous_final: Option<Vec<u8>>,
    cached: [Vec<u8>; 4],
    pub submissions: u64,
    pub workgroups: u64,
    pub adapter: String,
}
fn initbuf(
    device: &wgpu::Device,
    bytes: &[u8],
    usage: wgpu::BufferUsages,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes,
        usage,
    })
}
impl Graph {
    pub fn new(
        key: SceneKey,
        options: Options,
        source: &[u8],
        mask: &[u8],
    ) -> Result<Self, String> {
        key.validate()?;
        options.validate()?;
        let n = key.width as usize * key.height as usize;
        if source.len() != n * 4 || mask.len() != n || mask.iter().any(|&m| m > 1) {
            return Err("F4C GPU SOURCE LENGTH/MASK".into());
        }
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .map_err(|e| format!("F4C GPU ADAPTER UNAVAILABLE {e}"))?;
        let info = adapter.get_info();
        let label = format!(
            "{} / {} / {:?} / {:?}",
            info.name, info.driver, info.backend, info.device_type
        );
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("F4C GPU DEVICE UNAVAILABLE {e}"))?;
        let source_gpu = initbuf(
            &device,
            source,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "f4c-original",
        );
        let stage = std::array::from_fn(|_| {
            initbuf(
                &device,
                source,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                "f4c-stage",
            )
        });
        let mut mask_bytes = Vec::with_capacity(n * 4);
        for &v in mask {
            mask_bytes.extend_from_slice(&(v as u32).to_le_bytes());
        }
        let mask_gpu = initbuf(
            &device,
            &mask_bytes,
            wgpu::BufferUsages::STORAGE,
            "f4c-f2-mask",
        );
        let history = initbuf(
            &device,
            source,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "f4c-history",
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("f4c-incremental-stage"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/f4c.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("f4c-typed-graph"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            key,
            options,
            device,
            queue,
            pipeline,
            source: source_gpu,
            mask: mask_gpu,
            history,
            stage,
            mask_cpu: mask.to_vec(),
            prev_source: source.to_vec(),
            previous_final: None,
            cached: std::array::from_fn(|_| source.to_vec()),
            submissions: 0,
            workgroups: 0,
            adapter: label,
        })
    }
    pub fn update(&mut self, source: &[u8], declared: Damage) -> Result<ResultFrame, String> {
        let actual = verify_damage(&self.prev_source, source, self.key, declared)?;
        let history_active = self.previous_final.is_some() && !self.options.reduced_motion;
        let initial = self.submissions == 0;
        let regions = if initial {
            [full(self.key); 4]
        } else {
            invalidations(actual, self.key, self.options, history_active)
        };
        let expected = reference(
            self.key,
            self.options,
            source,
            &self.mask_cpu,
            self.previous_final.as_deref(),
        )?;
        if regions.iter().all(|r| r.is_empty()) {
            if self.cached != expected {
                return Err("F4C IDLE CAUSAL ORACLE RED".into());
            }
            return Ok(ResultFrame {
                stages: self.cached.clone(),
                regions,
                workgroups: 0,
                dispatches: 0,
                submissions: self.submissions,
                source_upload_bytes: 0,
                readback_bytes: 0,
            });
        }
        let n = source.len() as u64;
        let source_uploaded = source != self.prev_source;
        if source_uploaded {
            self.queue.write_buffer(&self.source, 0, source);
        }
        if let Some(prev) = self.previous_final.as_ref() {
            self.queue.write_buffer(&self.history, 0, prev);
        }
        let mut uniforms = Vec::new();
        let mut bindings = Vec::new();
        let layout = self.pipeline.get_bind_group_layout(0);
        let mut dispatched = 0u32;
        let mut workgroups = 0u64;
        for (i, roi) in regions.iter().enumerate() {
            if roi.is_empty() {
                continue;
            }
            let values = [
                self.key.width,
                self.key.height,
                roi.x,
                roi.y,
                i as u32,
                self.options.radius,
                u32::from(history_active),
                u32::from(self.options.reduced_motion),
                roi.w,
                roi.h,
                0,
                0,
            ];
            let mut bytes = Vec::with_capacity(48);
            for v in values {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            let u = initbuf(
                &self.device,
                &bytes,
                wgpu::BufferUsages::UNIFORM,
                "f4c-stage-contract",
            );
            uniforms.push(u);
            let prev = if i == 0 {
                &self.source
            } else {
                &self.stage[i - 1]
            };
            let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("f4c-stage-resources"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforms.last().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.source.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: prev.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.stage[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.mask.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.history.as_entire_binding(),
                    },
                ],
            });
            bindings.push((i, *roi, bind));
            dispatched += 1;
            workgroups += roi.w.div_ceil(8) as u64 * roi.h.div_ceil(8) as u64;
        }
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("f4c-incremental"),
            });
        for (_i, roi, bind) in &bindings {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("f4c-bounded-effect-node"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind, &[]);
            pass.dispatch_workgroups(roi.w.div_ceil(8), roi.h.div_ceil(8), 1);
        }
        // Full readbacks are for the falsifiable oracle, not part of a shipping ROI presenter.
        let readbacks: Vec<wgpu::Buffer> = (0..4)
            .map(|i| {
                let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("f4c-stage-verification-readback"),
                    size: n,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                encoder.copy_buffer_to_buffer(&self.stage[i], 0, &buffer, 0, n);
                buffer
            })
            .collect();
        self.queue.submit(Some(encoder.finish()));
        self.submissions += 1;
        self.workgroups += workgroups;
        let mut results = Vec::new();
        for b in &readbacks {
            let slice = b.slice(..);
            let (tx, rx) = mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(15)),
                })
                .map_err(|e| format!("F4C GPU POLL FAILED {e}"))?;
            rx.recv_timeout(Duration::from_secs(15))
                .map_err(|e| e.to_string())?
                .map_err(|e| format!("F4C GPU MAP FAILED {e}"))?;
            let bytes = slice
                .get_mapped_range()
                .map_err(|e| e.to_string())?
                .to_vec();
            b.unmap();
            results.push(bytes);
        }
        let stages: [Vec<u8>; 4] = results
            .try_into()
            .map_err(|_| "F4C STAGE COUNT".to_string())?;
        for i in 0..4 {
            if stages[i].len() != expected[i].len() {
                return Err("F4C OUTPUT LENGTH".into());
            }
            let max = stages[i]
                .iter()
                .zip(&expected[i])
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap_or(0);
            if max > 2 {
                return Err(format!("F4C_GPU_STAGE_ORACLE_RED stage={i} max={max}"));
            }
            // Stage pixel parity is bounded by the same observed F4/F3 GPU property.
        }
        self.cached = stages.clone();
        self.prev_source = source.to_vec();
        self.previous_final = if self.options.reduced_motion {
            None
        } else {
            Some(stages[3].clone())
        };
        Ok(ResultFrame {
            stages,
            regions,
            workgroups,
            dispatches: dispatched,
            submissions: self.submissions,
            source_upload_bytes: if source_uploaded { n } else { 0 },
            readback_bytes: n * 4,
        })
    }
}

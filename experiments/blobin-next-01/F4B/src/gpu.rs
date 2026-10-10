//! Dedicated persistent Vulkan compute compositor. Destruction is application-driven:
//! this is NOT an external physical GPU fault, kernel reset or in-flight loss.
use crate::{validate_damage, Damage};
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

pub struct Resident {
    width: u32,
    height: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    target: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    last: Vec<u8>,
    pub explicit_submissions: u64,
    pub dispatched_workgroups: u64,
    pub device_destroyed: bool,
    pub adapter: String,
}
impl Resident {
    pub fn new(width: u32, height: u32, initial: &[u8]) -> Result<Self, String> {
        validate_damage(
            initial,
            initial,
            width,
            height,
            Damage {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        )?;
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .map_err(|e| format!("F4B GPU ADAPTER UNAVAILABLE: {e}"))?;
        let info = adapter.get_info();
        let label = format!(
            "{} / {} / {:?} / {:?}",
            info.name, info.driver, info.backend, info.device_type
        );
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("F4B GPU DEVICE UNAVAILABLE: {e}"))?;
        let target = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("F4B-resident-pixels"),
            contents: initial,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("F4B-ROI-patch"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/roi.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("F4B-region-only-dispatch"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            width,
            height,
            device,
            queue,
            target,
            pipeline,
            last: initial.to_vec(),
            explicit_submissions: 0,
            dispatched_workgroups: 0,
            device_destroyed: false,
            adapter: label,
        })
    }
    pub fn current(&self) -> &[u8] {
        &self.last
    }
    pub fn idle(&self) -> Result<&[u8], String> {
        if self.device_destroyed {
            return Err("F4B DEVICE DESTROYED".into());
        }
        // No encoder, no dispatch, no queue.submit, no poll.
        Ok(&self.last)
    }
    pub fn destroy_owned_device(&mut self) {
        if !self.device_destroyed {
            self.device.destroy();
            self.device_destroyed = true;
        }
    }
    pub fn patch(&mut self, next: &[u8], damage: Damage) -> Result<Vec<u8>, String> {
        if self.device_destroyed {
            return Err("F4B DEVICE DESTROYED".into());
        }
        validate_damage(&self.last, next, self.width, self.height, damage)?;
        if damage.is_empty() {
            return Ok(self.idle()?.to_vec());
        }
        let source = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("F4B-new-full-frame-input"),
                contents: next,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let mut bytes = Vec::with_capacity(32);
        for value in [
            self.width,
            self.height,
            damage.x,
            damage.y,
            damage.w,
            damage.h,
            0,
            0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let config = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("F4B-readonly-scissor"),
                contents: &bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let layout = self.pipeline.get_bind_group_layout(0);
        let bindings = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("F4B-three-resources"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: config.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: source.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.target.as_entire_binding(),
                },
            ],
        });
        let size = self.width as u64 * self.height as u64 * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("F4B-ROI-readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("F4B-bounded-region-submit"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("F4B-ROI-only"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(damage.w.div_ceil(8), damage.h.div_ceil(8), 1);
        }
        encoder.copy_buffer_to_buffer(&self.target, 0, &readback, 0, size);
        self.queue.submit(Some(encoder.finish()));
        self.explicit_submissions += 1;
        self.dispatched_workgroups += (damage.w.div_ceil(8) * damage.h.div_ceil(8)) as u64;
        let slice = readback.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(15)),
            })
            .map_err(|e| format!("F4B GPU POLL FAILED: {e}"))?;
        rx.recv_timeout(Duration::from_secs(15))
            .map_err(|e| format!("F4B GPU READBACK TIMEOUT: {e}"))?
            .map_err(|e| format!("F4B GPU MAP FAILED: {e}"))?;
        let bytes = slice
            .get_mapped_range()
            .map_err(|e| format!("F4B RANGE FAILED: {e}"))?
            .to_vec();
        readback.unmap();
        if bytes != next {
            return Err("F4B GPU COMPOSITE PARITY FAILED".into());
        }
        self.last = bytes.clone();
        Ok(bytes)
    }
}

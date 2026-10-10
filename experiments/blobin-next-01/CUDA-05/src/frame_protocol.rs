//! BLOB-FRAME-v1: explicitly bounded scene and a deterministic CPU presentation mapping.
//! Frames are derived from real CPU/CUDA/Vulkan SDF samples, never from HTML mocks.
use blobin_cuda02_provider::{Primitive, Request, Sample};
use blobin_f2_topology::Scene;
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "BLOB-FRAME-v1";
pub const MAX_CONTROL: usize = 16_384;
pub const MAX_PIXELS: usize = 1_048_576;
pub const MAX_FRAME: usize = MAX_PIXELS * 4;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveInput {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub op: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum ClientMessage {
    Hello {
        version: String,
    },
    Capabilities {
        version: String,
    },
    Ping {
        version: String,
    },
    RenderRequest {
        version: String,
        request_id: u64,
        scene_revision: u64,
        width_px: u32,
        height_px: u32,
        backend_requested: String,
        view_mode: String,
        primitives: Vec<PrimitiveInput>,
    },
}

impl ClientMessage {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() || bytes.len() > MAX_CONTROL {
            return Err("FRAME_CONTROL_LENGTH_INVALID".into());
        }
        let result: Self =
            serde_json::from_slice(bytes).map_err(|_| "FRAME_CONTROL_JSON_INVALID")?;
        if result.version() != VERSION {
            return Err("FRAME_PROTOCOL_VERSION_INVALID".into());
        }
        if let Self::RenderRequest {
            width_px,
            height_px,
            backend_requested,
            view_mode,
            primitives,
            ..
        } = &result
        {
            let pixels = (*width_px as usize)
                .checked_mul(*height_px as usize)
                .ok_or("FRAME_DIMENSIONS_OVERFLOW")?;
            if !(16..=1024).contains(width_px)
                || !(16..=1024).contains(height_px)
                || pixels > MAX_PIXELS
                || primitives.is_empty()
                || primitives.len() > 64
            {
                return Err("FRAME_SCENE_BOUNDS_INVALID".into());
            }
            if !matches!(backend_requested.as_str(), "cpu" | "cuda" | "vulkan")
                || !matches!(
                    view_mode.as_str(),
                    "lit" | "normals" | "distance" | "coverage"
                )
            {
                return Err("FRAME_BACKEND_OR_VIEW_INVALID".into());
            }
            if primitives[0].op != 0
                || primitives.iter().any(|p| {
                    !p.x.is_finite()
                        || !p.y.is_finite()
                        || !p.radius.is_finite()
                        || !(-2.0..=2.0).contains(&p.x)
                        || !(-2.0..=2.0).contains(&p.y)
                        || p.radius <= 0.0
                        || p.radius > 2.0
                        || p.op > 3
                })
            {
                return Err("FRAME_PRIMITIVES_INVALID".into());
            }
        }
        Ok(result)
    }

    fn version(&self) -> &str {
        match self {
            Self::Hello { version }
            | Self::Capabilities { version }
            | Self::Ping { version }
            | Self::RenderRequest { version, .. } => version,
        }
    }

    pub fn make_request(&self) -> Result<Request, String> {
        if let Self::RenderRequest {
            scene_revision,
            width_px,
            height_px,
            primitives,
            ..
        } = self
        {
            let scene = Scene {
                epoch: *scene_revision,
                logical_width: *width_px,
                logical_height: *height_px,
                dpr: 1,
                ..Scene::default()
            };
            Request::from_f2(
                scene,
                primitives
                    .iter()
                    .map(|p| Primitive {
                        x: p.x,
                        y: p.y,
                        radius: p.radius,
                        op: p.op,
                    })
                    .collect(),
            )
        } else {
            Err("FRAME_SCENE_REQUEST_REQUIRED".into())
        }
    }
}

pub fn colorize(samples: &[Sample], view_mode: &str) -> Result<Vec<u8>, String> {
    if samples.len() > MAX_PIXELS
        || !matches!(view_mode, "lit" | "normals" | "distance" | "coverage")
    {
        return Err("FRAME_COLORIZE_BOUNDS_OR_MODE".into());
    }
    let mut out = Vec::with_capacity(samples.len() * 4);
    for s in samples {
        if ![s.distance, s.nx, s.ny, s.coverage]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err("FRAME_NONFINITE_BACKEND_SAMPLES".into());
        }
        let a = s.coverage.clamp(0.0, 1.0);
        let color: [f32; 3] = match view_mode {
            "normals" => [(s.nx * 0.5 + 0.5), (s.ny * 0.5 + 0.5), 0.85],
            "distance" => {
                let t = (0.5 - s.distance * 3.5).clamp(0.0, 1.0);
                [t, (0.15 + 0.60 * t).clamp(0.0, 1.0), 1.0 - t * 0.75]
            }
            "coverage" => [a, a, a],
            "lit" => {
                let light = (s.nx * -0.46 + s.ny * -0.73 + 0.59).max(0.0);
                let spec = light.powi(12) * 0.36;
                let r = (0.11 + light * 0.55 + spec).min(1.0);
                let g = (0.32 + light * 0.63 + spec).min(1.0);
                let b = (0.41 + light * 0.49 + spec).min(1.0);
                // Diagnostic visual shading of real GPU-produced normal/coverage fields.
                [
                    0.028 * (1.0 - a) + r * a,
                    0.036 * (1.0 - a) + g * a,
                    0.052 * (1.0 - a) + b * a,
                ]
            }
            _ => unreachable!(),
        };
        for channel in color {
            out.push((channel.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
        out.push(255);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request() -> Vec<u8> {
        br#"{"version":"BLOB-FRAME-v1","type":"RENDER_REQUEST","request_id":12,"scene_revision":9,"width_px":64,"height_px":32,"backend_requested":"cpu","view_mode":"lit","primitives":[{"x":0.0,"y":0.0,"radius":0.15,"op":0}]}"#.to_vec()
    }
    #[test]
    fn valid_scene_preserves_dimensions_and_revision() {
        let msg = ClientMessage::parse(&test_request()).unwrap();
        let req = msg.make_request().unwrap();
        assert_eq!(req.dimensions(), (64, 32));
        assert_eq!(req.authority().epoch, 9);
        assert_eq!(req.primitives().len(), 1);
    }
    #[test]
    fn rejects_extra_fields_invalid_values_and_unknown_ops() {
        let orig = test_request();
        let v = String::from_utf8(orig).unwrap();
        for (a, b) in [
            ("\"op\":0", "\"op\":4"),
            ("\"width_px\":64", "\"width_px\":0"),
            ("\"view_mode\":\"lit\"", "\"view_mode\":\"material\""),
            (
                "\"backend_requested\":\"cpu\"",
                "\"backend_requested\":\"auto\"",
            ),
            (
                "\"scene_revision\":9",
                "\"scene_revision\":9,\"unexpected\":true",
            ),
            ("BLOB-FRAME-v1", "BLOB-FRAME-v2"),
        ] {
            assert!(
                ClientMessage::parse(v.replace(a, b).as_bytes()).is_err(),
                "{a}"
            );
        }
    }
    #[test]
    fn colorize_is_opaque_and_rejects_nonfinite() {
        let samples = [Sample {
            distance: -0.2,
            nx: 0.0,
            ny: 1.0,
            coverage: 1.0,
        }];
        for mode in ["lit", "normals", "distance", "coverage"] {
            let rgba = colorize(&samples, mode).unwrap();
            assert_eq!(rgba.len(), 4);
            assert_eq!(rgba[3], 255);
        }
        assert!(colorize(
            &[Sample {
                distance: f32::NAN,
                ..samples[0]
            }],
            "lit"
        )
        .is_err());
    }
}

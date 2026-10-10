//! GPU-03 experimental read-only projection boundary; not product/agent authority.
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

const MAX_BYTES: u64 = 65_536;
const MAX_ENTITIES: usize = 64;
const MAX_DELTAS: usize = 32;
const MAX_ID_BYTES: usize = 96;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Vec2 {
    x: f64,
    y: f64,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pose {
    position: Vec2,
    rotation: f64,
    scale: Vec2,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Shape {
    Rect {
        width: f64,
        height: f64,
        radius: f64,
    },
    Polygon {
        closed: bool,
        vertices: Vec<Vec2>,
    },
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Accessibility {
    role: String,
    label: Option<String>,
    hidden: bool,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entity {
    entity_id: String,
    role: String,
    visible: bool,
    z: f64,
    world_pose: Pose,
    shape: Shape,
    accessibility: Option<Accessibility>,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Frame {
    schema_version: u32,
    sequence: u64,
    scene_time: f64,
    motion_mode: String,
    entities: Vec<Entity>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Delta {
    schema_version: u32,
    from_sequence: u64,
    to_sequence: u64,
    scene_time: f64,
    motion_mode: String,
    order: Vec<String>,
    births: Vec<Entity>,
    updates: Vec<Entity>,
    deaths: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    kind: String,
    entity_id: String,
    meters_per_unit: f64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Document {
    schema: String,
    source: String,
    initial: Frame,
    deltas: Vec<Delta>,
    binding: Binding,
}

#[derive(Clone, Debug)]
pub struct SceneStream {
    frames: Vec<Frame>,
    entity_id: String,
    meters_per_unit: f64,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_BYTES
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
fn finite(x: f64) -> bool {
    x.is_finite() && x.abs() <= 1_000_000.0
}
fn validate_entity(e: &Entity) -> Result<(), String> {
    if !valid_id(&e.entity_id)
        || !matches!(
            e.role.as_str(),
            "card" | "surface" | "control" | "proxy" | "anchor" | "decoration"
        )
    {
        return Err("UNRECOGNIZED_IDENTITY_OR_ROLE".into());
    }
    let p = &e.world_pose;
    if ![
        e.z,
        p.position.x,
        p.position.y,
        p.rotation,
        p.scale.x,
        p.scale.y,
    ]
    .iter()
    .all(|&v| finite(v))
        || p.scale.x <= 0.0
        || p.scale.y <= 0.0
    {
        return Err("NONFINITE_OR_INVALID_POSE".into());
    }
    match &e.shape {
        Shape::Rect {
            width,
            height,
            radius,
        } => {
            if ![width, height, radius].iter().all(|&v| finite(*v))
                || *width <= 0.0
                || *height <= 0.0
                || *radius < 0.0
            {
                return Err("INVALID_RECT_GEOMETRY".into());
            }
        }
        Shape::Polygon { closed, vertices } => {
            if !closed
                || !(3..=256).contains(&vertices.len())
                || vertices.iter().any(|v| !finite(v.x) || !finite(v.y))
            {
                return Err("INVALID_POLYGON_GEOMETRY".into());
            }
        }
    }
    if let Some(a) = &e.accessibility {
        if a.role.len() > 64 || a.label.as_ref().is_some_and(|s| s.len() > 256) {
            return Err("INVALID_ACCESSIBILITY_METADATA".into());
        }
    }
    Ok(())
}
fn validate_frame(f: &Frame) -> Result<(), String> {
    if f.schema_version != 1
        || !finite(f.scene_time)
        || f.scene_time < 0.0
        || !matches!(f.motion_mode.as_str(), "normal" | "reduced")
        || f.entities.len() > MAX_ENTITIES
    {
        return Err("INVALID_FRAME_HEADER".into());
    }
    let mut ids = HashSet::new();
    for e in &f.entities {
        validate_entity(e)?;
        if !ids.insert(e.entity_id.as_str()) {
            return Err("DUPLICATE_IDENTITY".into());
        }
    }
    Ok(())
}
fn apply(mut old: Frame, d: Delta) -> Result<Frame, String> {
    if d.schema_version != 1
        || old.sequence != d.from_sequence
        || d.to_sequence <= old.sequence
        || !finite(d.scene_time)
        || d.scene_time < old.scene_time
        || !matches!(d.motion_mode.as_str(), "normal" | "reduced")
    {
        return Err("STALE_OR_INVALID_DELTA".into());
    }
    let mut entities: HashMap<String, Entity> = old
        .entities
        .into_iter()
        .map(|e| (e.entity_id.clone(), e))
        .collect();
    let mut touched = HashSet::new();
    for id in &d.deaths {
        if !touched.insert(id.clone()) || entities.remove(id).is_none() {
            return Err("UNKNOWN_OR_DUPLICATE_DEATH".into());
        }
    }
    for e in d.updates {
        validate_entity(&e)?;
        if !touched.insert(e.entity_id.clone()) || !entities.contains_key(&e.entity_id) {
            return Err("UNKNOWN_OR_DUPLICATE_UPDATE".into());
        }
        entities.insert(e.entity_id.clone(), e);
    }
    for e in d.births {
        validate_entity(&e)?;
        if !touched.insert(e.entity_id.clone()) || entities.contains_key(&e.entity_id) {
            return Err("DUPLICATE_BIRTH".into());
        }
        entities.insert(e.entity_id.clone(), e);
    }
    if d.order.len() != entities.len()
        || d.order.iter().collect::<HashSet<_>>().len() != entities.len()
    {
        return Err("DELTA_ORDER_PARTITION_INVALID".into());
    }
    old.entities = d
        .order
        .iter()
        .map(|id| {
            entities
                .remove(id)
                .ok_or("DELTA_UNKNOWN_ORDER_ID".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    old.sequence = d.to_sequence;
    old.scene_time = d.scene_time;
    old.motion_mode = d.motion_mode;
    validate_frame(&old)?;
    Ok(old)
}
impl SceneStream {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES as usize {
            return Err("SCENE_STREAM_TOO_LARGE".into());
        }
        let d: Document =
            serde_json::from_slice(bytes).map_err(|e| format!("SCENE_STREAM_INVALID_JSON {e}"))?;
        if d.schema != "blobin.gpu03.scene-stream.v1"
            || d.source != "in.gnu6"
            || d.deltas.len() > MAX_DELTAS
            || d.binding.kind != "cavity-control-x"
            || !valid_id(&d.binding.entity_id)
            || !finite(d.binding.meters_per_unit)
            || d.binding.meters_per_unit <= 0.0
            || d.binding.meters_per_unit > 0.01
        {
            return Err("SCENE_STREAM_UNSUPPORTED_OR_UNBOUNDED".into());
        }
        validate_frame(&d.initial)?;
        let mut frames = vec![d.initial];
        for delta in d.deltas {
            let next = apply(frames.last().expect("initial frame").clone(), delta)?;
            frames.push(next);
        }
        for f in &frames {
            let e = f
                .entities
                .iter()
                .find(|e| e.entity_id == d.binding.entity_id)
                .ok_or("CAVITY_BINDING_MISSING")?;
            if e.role != "control"
                || !matches!(e.shape, Shape::Rect { .. })
                || !e.visible
                || !finite(e.world_pose.position.x * d.binding.meters_per_unit)
                || (e.world_pose.position.x * d.binding.meters_per_unit).abs() > 0.2
            {
                return Err("CAVITY_BINDING_INVALID_OR_OUT_OF_RANGE".into());
            }
        }
        Ok(Self {
            frames,
            entity_id: d.binding.entity_id,
            meters_per_unit: d.binding.meters_per_unit,
        })
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let len = fs::metadata(path)
            .map_err(|e| format!("SCENE_PATH_UNAVAILABLE {e}"))?
            .len();
        if len > MAX_BYTES {
            return Err("SCENE_STREAM_TOO_LARGE".into());
        }
        Self::from_bytes(&fs::read(path).map_err(|e| format!("SCENE_READ_FAILED {e}"))?)
    }
    /// Validation of an untrusted, presentation-only delta.
    /// No mutation access to in.gnu6 scene truth escapes this boundary.
    pub fn apply_live_bytes(&mut self, bytes: &[u8]) -> Result<u64, String> {
        if bytes.len() > 8192 {
            return Err("GPU04_DELTA_TOO_LARGE".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Packet {
            schema: String,
            delta: Delta,
        }
        let packet: Packet =
            serde_json::from_slice(bytes).map_err(|_| "GPU04_DELTA_INVALID_PACKET".to_string())?;
        if packet.schema != "blobin.gpu04.delta.v1" {
            return Err("GPU04_DELTA_UNKNOWN_VERSION".into());
        }
        let next = apply(
            self.frames.last().ok_or("MISSING_SOURCE_FRAME")?.clone(),
            packet.delta,
        )?;
        let cavity = next
            .entities
            .iter()
            .find(|e| e.entity_id == self.entity_id)
            .ok_or("CAVITY_BINDING_MISSING")?;
        if cavity.role != "control"
            || !cavity.visible
            || !matches!(cavity.shape, Shape::Rect { .. })
            || (cavity.world_pose.position.x * self.meters_per_unit).abs() > 0.2
        {
            return Err("CAVITY_BINDING_INVALID_OR_OUT_OF_RANGE".into());
        }
        let sequence = next.sequence;
        // No unbounded history: one current semantic projection only.
        self.frames.clear();
        self.frames.push(next);
        Ok(sequence)
    }

    pub fn latest_offset(&self) -> f32 {
        self.offset(self.frames.last().expect("qualified initial source frame"))
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    pub fn cavity_offset_at(&self, seconds: f32) -> f32 {
        let period = self.frames.last().map_or(0.0, |f| f.scene_time);
        if period <= 0.0 {
            return self.offset(&self.frames[0]);
        }
        let t = f64::from(seconds.max(0.0)).rem_euclid(period);
        for pair in self.frames.windows(2) {
            if t <= pair[1].scene_time {
                let a = &pair[0];
                let b = &pair[1];
                let dt = b.scene_time - a.scene_time;
                if dt <= 0.0 {
                    return self.offset(b);
                }
                let alpha = ((t - a.scene_time) / dt) as f32;
                return self.offset(a) + (self.offset(b) - self.offset(a)) * alpha;
            }
        }
        self.offset(self.frames.last().expect("at least one frame"))
    }
    fn offset(&self, frame: &Frame) -> f32 {
        frame
            .entities
            .iter()
            .find(|e| e.entity_id == self.entity_id)
            .map_or(0.0, |e| {
                (e.world_pose.position.x * self.meters_per_unit) as f32
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOLDEN: &str = include_str!("../fixtures/scene-stream.v1.json");
    #[test]
    fn live_delta_accepts_valid_update_and_rejects_replay_without_mutation() {
        let mut stream = SceneStream::from_bytes(GOLDEN.as_bytes()).unwrap();
        assert_eq!(stream.frames.last().unwrap().sequence, 13);
        let mut d = serde_json::json!({
            "schema":"blobin.gpu04.delta.v1",
            "delta":{"schemaVersion":1,"fromSequence":13,"toSequence":14,
               "sceneTime":4.0,"motionMode":"normal",
               "order":["stage","cavity-control"],"births":[],"deaths":[],
               "updates":[{"entityId":"cavity-control","role":"control",
                   "visible":true,"z":0,
                   "worldPose":{"position":{"x":10,"y":0},"rotation":0,
                     "scale":{"x":1,"y":1}},
                   "shape":{"kind":"rect","width":24,"height":24,"radius":4},
                   "accessibility":null}]}
        });
        assert_eq!(stream.apply_live_bytes(d.to_string().as_bytes()), Ok(14));
        assert!((stream.latest_offset() - 0.1).abs() < 1e-6);
        assert!(stream
            .apply_live_bytes(d.to_string().as_bytes())
            .unwrap_err()
            .contains("STALE"));
        assert!((stream.latest_offset() - 0.1).abs() < 1e-6);
        d["delta"]["fromSequence"] = serde_json::json!(14);
        d["delta"]["toSequence"] = serde_json::json!(15);
        d["delta"]["updates"][0]["worldPose"]["position"]["x"] = serde_json::json!(100);
        assert!(stream
            .apply_live_bytes(d.to_string().as_bytes())
            .unwrap_err()
            .contains("OUT_OF_RANGE"));
        assert!((stream.latest_offset() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn live_delta_rejects_oversize_unknown_fields_and_bad_version() {
        let mut stream = SceneStream::from_bytes(GOLDEN.as_bytes()).unwrap();
        assert_eq!(
            stream.apply_live_bytes(&vec![b' '; 8193]).unwrap_err(),
            "GPU04_DELTA_TOO_LARGE"
        );
        assert_eq!(
            stream
                .apply_live_bytes(br#"{"schema":"bad","delta":{}}"#)
                .unwrap_err(),
            "GPU04_DELTA_INVALID_PACKET"
        );
        assert_eq!(
            stream
                .apply_live_bytes(br#"{"schema":"blobin.gpu04.delta.v1","delta":{},"tool":"bash"}"#)
                .unwrap_err(),
            "GPU04_DELTA_INVALID_PACKET"
        );
    }

    #[test]
    fn independent_rust_replay_and_motion_samples() {
        let s = SceneStream::from_bytes(GOLDEN.as_bytes()).unwrap();
        assert_eq!(s.frame_count(), 4);
        for (t, expected) in [
            (0.0, 0.0),
            (0.5, 0.05),
            (1.0, 0.1),
            (1.5, -0.005),
            (2.0, -0.11),
            (3.0, 0.0),
        ] {
            assert!((s.cavity_offset_at(t) - expected).abs() < 1e-6, "time={t}");
        }
    }
    #[test]
    fn rejects_duplicate_identity_and_unbounded_mapping() {
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        let first = doc["initial"]["entities"][0].clone();
        doc["initial"]["entities"]
            .as_array_mut()
            .unwrap()
            .push(first);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes())
            .unwrap_err()
            .contains("DUPLICATE"));
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["binding"]["metersPerUnit"] = serde_json::json!(0.5);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes()).is_err());
    }
    #[test]
    fn rejects_stale_delta_unknown_update_and_identity_loss() {
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["deltas"][0]["fromSequence"] = serde_json::json!(9);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes())
            .unwrap_err()
            .contains("STALE"));
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["deltas"][0]["updates"][0]["entityId"] = serde_json::json!("ghost");
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes())
            .unwrap_err()
            .contains("UNKNOWN"));
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["deltas"][0]["order"] = serde_json::json!(["stage", "ghost"]);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes()).is_err());
    }
    #[test]
    fn rejects_oversize_unknown_fields_and_nonfinite_values() {
        assert!(SceneStream::from_bytes(&vec![b'X'; 65_537]).is_err());
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["unexpected"] = serde_json::json!(true);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes()).is_err());
        let mut doc: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
        doc["initial"]["entities"][0]["worldPose"]["position"]["x"] = serde_json::json!(1e100);
        assert!(SceneStream::from_bytes(doc.to_string().as_bytes()).is_err());
    }
}

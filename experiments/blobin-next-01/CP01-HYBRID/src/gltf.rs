//! GEO-03: fail-closed, static, single-mesh triangle importer. NO filesystem access.
//! This is a strict documented subset of glTF 2.0 / GLB 2.0, not a full loader.
use crate::{mesh::CertifiedConvexSolid, GeometryError, MeshAsset, Vec3, MAX_TRIANGLES};
use serde_json::Value;

pub const MAX_CONTAINER: usize = 16 * 1024 * 1024;
pub const MAX_JSON: usize = 2 * 1024 * 1024;
pub const MAX_VERTICES: usize = 196_608;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportError {
    Empty,
    TooLarge,
    GlbHeader,
    GlbChunk,
    Json,
    Unsupported,
    UnsupportedUri,
    MissingField,
    OutOfBounds,
    Alignment,
    InvalidAccessor,
    InvalidGeometry,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Glb,
    GltfJsonExternalBuffer,
}
#[derive(Debug, Clone)]
pub struct ImportedSurface {
    pub mesh: MeshAsset,
    pub source: SourceFormat,
    pub mesh_index: usize,
    pub primitive_index: usize,
}
impl ImportedSurface {
    /// An explicit post-import promotion, never performed by either parser.
    /// Only the existing conservative convex validator can grant signed-volume
    /// semantics; any arbitrary nonconvex imported mesh remains a surface.
    pub fn try_certify_convex(self) -> Result<CertifiedConvexSolid, GeometryError> {
        CertifiedConvexSolid::certify(self.mesh)
    }
}
fn u32le(s: &[u8]) -> Result<u32, ImportError> {
    let a: [u8; 4] = s.try_into().map_err(|_| ImportError::OutOfBounds)?;
    Ok(u32::from_le_bytes(a))
}
fn usize_field(v: &Value, name: &str) -> Result<usize, ImportError> {
    v.get(name)
        .ok_or(ImportError::MissingField)?
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(ImportError::InvalidAccessor)
}
fn offset(v: &Value, name: &str) -> Result<usize, ImportError> {
    match v.get(name) {
        None => Ok(0),
        Some(x) => x
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or(ImportError::InvalidAccessor),
    }
}
fn only_array<'a>(v: &'a Value, name: &str) -> Result<&'a [Value], ImportError> {
    v.get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(ImportError::MissingField)
}
fn bounds(
    buf_len: usize,
    start: usize,
    size: usize,
) -> Result<std::ops::Range<usize>, ImportError> {
    let end = start.checked_add(size).ok_or(ImportError::OutOfBounds)?;
    if end > buf_len {
        return Err(ImportError::OutOfBounds);
    }
    Ok(start..end)
}
fn unsupported_nonempty(v: &Value, field: &str) -> bool {
    match v.get(field) {
        None => false,
        Some(x) if x.as_array().is_some_and(Vec::is_empty) => false,
        _ => true,
    }
}
fn parse_document(
    json: &[u8],
    data: &[u8],
    source: SourceFormat,
) -> Result<ImportedSurface, ImportError> {
    if json.is_empty() || data.is_empty() {
        return Err(ImportError::Empty);
    }
    if json.len() > MAX_JSON || data.len() > MAX_CONTAINER {
        return Err(ImportError::TooLarge);
    }
    let v: Value = serde_json::from_slice(json).map_err(|_| ImportError::Json)?;
    if !v.is_object() {
        return Err(ImportError::Json);
    }
    let asset = v.get("asset").ok_or(ImportError::MissingField)?;
    if asset.get("version").and_then(Value::as_str) != Some("2.0")
        || asset
            .get("minVersion")
            .and_then(Value::as_str)
            .is_some_and(|s| s != "2.0")
    {
        return Err(ImportError::Unsupported);
    }
    for disallowed in [
        "animations",
        "skins",
        "extensionsRequired",
        "extensionsUsed",
    ] {
        if unsupported_nonempty(&v, disallowed) {
            return Err(ImportError::Unsupported);
        }
    }
    // Refuse scene hierarchy, morph targets and transforms until explicitly evaluated.
    if let Some(nodes) = v.get("nodes") {
        let arr = nodes.as_array().ok_or(ImportError::Unsupported)?;
        if arr.len() != 1 {
            return Err(ImportError::Unsupported);
        }
        let n = &arr[0];
        if n.get("mesh").and_then(Value::as_u64) != Some(0)
            || [
                "matrix",
                "translation",
                "rotation",
                "scale",
                "children",
                "skin",
                "weights",
            ]
            .iter()
            .any(|name| n.get(*name).is_some())
        {
            return Err(ImportError::Unsupported);
        }
    }
    if let Some(scenes) = v.get("scenes") {
        let a = scenes.as_array().ok_or(ImportError::Unsupported)?;
        if a.len() != 1
            || a[0]
                .get("nodes")
                .and_then(Value::as_array)
                .is_none_or(|n| n.len() != 1 || n[0].as_u64() != Some(0))
        {
            return Err(ImportError::Unsupported);
        }
    }
    if v.get("scene").is_some() && v.get("scene").and_then(Value::as_u64) != Some(0) {
        return Err(ImportError::Unsupported);
    }
    let buffers = only_array(&v, "buffers")?;
    if buffers.len() != 1 {
        return Err(ImportError::Unsupported);
    }
    let declared = usize_field(&buffers[0], "byteLength")?;
    if declared == 0 || declared > data.len() || data.len() - declared > 3 {
        return Err(ImportError::OutOfBounds);
    }
    match source {
        SourceFormat::Glb if buffers[0].get("uri").is_some() => {
            return Err(ImportError::UnsupportedUri)
        }
        SourceFormat::GltfJsonExternalBuffer => {
            let uri = buffers[0]
                .get("uri")
                .and_then(Value::as_str)
                .ok_or(ImportError::UnsupportedUri)?;
            if uri.is_empty()
                || !uri.ends_with(".bin")
                || uri.contains('/')
                || uri.contains(char::from(92))
                || uri.contains("..")
                || uri.contains(':')
            {
                return Err(ImportError::UnsupportedUri);
            }
        }
        _ => {}
    }
    let views = only_array(&v, "bufferViews")?;
    let accessors = only_array(&v, "accessors")?;
    let meshes = only_array(&v, "meshes")?;
    if meshes.len() != 1 {
        return Err(ImportError::Unsupported);
    }
    let primitives = only_array(&meshes[0], "primitives")?;
    if primitives.len() != 1 {
        return Err(ImportError::Unsupported);
    }
    let primitive = &primitives[0];
    if primitive.get("mode").is_some() && primitive.get("mode").and_then(Value::as_u64) != Some(4) {
        return Err(ImportError::Unsupported);
    }
    if primitive.get("targets").is_some() || primitive.get("extensions").is_some() {
        return Err(ImportError::Unsupported);
    }
    let attrs = primitive
        .get("attributes")
        .ok_or(ImportError::MissingField)?;
    let pos_idx = usize_field(attrs, "POSITION")?;
    let pos = accessors.get(pos_idx).ok_or(ImportError::OutOfBounds)?;
    if pos.get("type").and_then(Value::as_str) != Some("VEC3")
        || pos.get("componentType").and_then(Value::as_u64) != Some(5126)
        || pos.get("normalized").and_then(Value::as_bool) == Some(true)
        || pos.get("sparse").is_some()
    {
        return Err(ImportError::Unsupported);
    }
    let pos_count = usize_field(pos, "count")?;
    if !(3..=MAX_VERTICES).contains(&pos_count) {
        return Err(ImportError::TooLarge);
    }
    let pos_view = views
        .get(usize_field(pos, "bufferView")?)
        .ok_or(ImportError::OutOfBounds)?;
    let positions = read_accessor(pos, pos_view, data, declared, 12, 4, pos_count)?;
    let mut vertices = Vec::with_capacity(pos_count);
    for chunk in positions.chunks_exact(12) {
        let x = f32::from_le_bytes(
            chunk[0..4]
                .try_into()
                .map_err(|_| ImportError::OutOfBounds)?,
        ) as f64;
        let y = f32::from_le_bytes(
            chunk[4..8]
                .try_into()
                .map_err(|_| ImportError::OutOfBounds)?,
        ) as f64;
        let z = f32::from_le_bytes(
            chunk[8..12]
                .try_into()
                .map_err(|_| ImportError::OutOfBounds)?,
        ) as f64;
        let p = Vec3::new(x, y, z);
        if !p.finite() {
            return Err(ImportError::InvalidGeometry);
        }
        vertices.push(p);
    }
    let indices: Vec<u32> = if let Some(idx) = primitive.get("indices") {
        let idx_index = idx
            .as_u64()
            .and_then(|x| usize::try_from(x).ok())
            .ok_or(ImportError::InvalidAccessor)?;
        let accessor = accessors.get(idx_index).ok_or(ImportError::OutOfBounds)?;
        if accessor.get("type").and_then(Value::as_str) != Some("SCALAR")
            || accessor.get("sparse").is_some()
            || accessor.get("normalized").and_then(Value::as_bool) == Some(true)
        {
            return Err(ImportError::Unsupported);
        }
        let component = usize_field(accessor, "componentType")?;
        let width = match component {
            5121 => 1,
            5123 => 2,
            5125 => 4,
            _ => return Err(ImportError::Unsupported),
        };
        let count = usize_field(accessor, "count")?;
        if count == 0 || count > MAX_TRIANGLES * 3 || count % 3 != 0 {
            return Err(ImportError::InvalidAccessor);
        }
        let view = views
            .get(usize_field(accessor, "bufferView")?)
            .ok_or(ImportError::OutOfBounds)?;
        // In core glTF, index accessors are tightly packed. byteStride
        // applies to interleaved vertex attributes, not ELEMENT_ARRAY_BUFFER.
        if view.get("byteStride").is_some() {
            return Err(ImportError::Unsupported);
        }
        let buf = read_accessor(accessor, view, data, declared, width, width, count)?;
        buf.chunks_exact(width)
            .map(|chunk| match width {
                1 => chunk[0] as u32,
                2 => u16::from_le_bytes([chunk[0], chunk[1]]) as u32,
                _ => u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            })
            .collect()
    } else {
        if pos_count % 3 != 0 || pos_count / 3 > MAX_TRIANGLES {
            return Err(ImportError::InvalidAccessor);
        }
        (0..pos_count as u32).collect()
    };
    let triangles = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mesh = MeshAsset {
        vertices,
        triangles,
    };
    mesh.preflight().map_err(|_| ImportError::InvalidGeometry)?;
    Ok(ImportedSurface {
        mesh,
        source,
        mesh_index: 0,
        primitive_index: 0,
    })
}
fn read_accessor(
    accessor: &Value,
    view: &Value,
    data: &[u8],
    declared: usize,
    element_size: usize,
    alignment: usize,
    count: usize,
) -> Result<Vec<u8>, ImportError> {
    if usize_field(view, "buffer")? != 0 {
        return Err(ImportError::Unsupported);
    }
    let v_start = offset(view, "byteOffset")?;
    let v_size = usize_field(view, "byteLength")?;
    bounds(declared, v_start, v_size)?;
    let a_offset = offset(accessor, "byteOffset")?;
    let stride = match view.get("byteStride") {
        None => element_size,
        Some(x) => x
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or(ImportError::InvalidAccessor)?,
    };
    if view.get("byteStride").is_some() && !(4..=252).contains(&stride) {
        return Err(ImportError::Alignment);
    }
    if stride < element_size
        || stride % alignment != 0
        || a_offset % alignment != 0
        || v_start % alignment != 0
        || (element_size == 12 && stride % 4 != 0)
    {
        return Err(ImportError::Alignment);
    }
    let last = count
        .checked_sub(1)
        .and_then(|n| n.checked_mul(stride))
        .and_then(|n| n.checked_add(a_offset))
        .and_then(|n| n.checked_add(element_size))
        .ok_or(ImportError::OutOfBounds)?;
    if last > v_size {
        return Err(ImportError::OutOfBounds);
    }
    let first = v_start
        .checked_add(a_offset)
        .ok_or(ImportError::OutOfBounds)?;
    let capacity = count
        .checked_mul(element_size)
        .ok_or(ImportError::TooLarge)?;
    if capacity > MAX_CONTAINER {
        return Err(ImportError::TooLarge);
    }
    let mut result = Vec::with_capacity(capacity);
    for i in 0..count {
        let offset = first
            .checked_add(i.checked_mul(stride).ok_or(ImportError::OutOfBounds)?)
            .ok_or(ImportError::OutOfBounds)?;
        result.extend_from_slice(&data[bounds(declared, offset, element_size)?]);
    }
    Ok(result)
}
pub fn import_glb(src: &[u8]) -> Result<ImportedSurface, ImportError> {
    if src.len() < 20 {
        return Err(ImportError::GlbHeader);
    }
    if src.len() > MAX_CONTAINER {
        return Err(ImportError::TooLarge);
    }
    if &src[0..4] != b"glTF"
        || u32le(&src[4..8])? != 2
        || usize::try_from(u32le(&src[8..12])?).ok() != Some(src.len())
    {
        return Err(ImportError::GlbHeader);
    }
    let mut at = 12usize;
    let mut json = None;
    let mut bin = None;
    let mut chunks = 0;
    while at < src.len() {
        if src.len() - at < 8 {
            return Err(ImportError::GlbChunk);
        }
        let n = u32le(&src[at..at + 4])? as usize;
        let kind = u32le(&src[at + 4..at + 8])?;
        if !n.is_multiple_of(4) {
            return Err(ImportError::GlbChunk);
        }
        let data_start = at.checked_add(8).ok_or(ImportError::GlbChunk)?;
        let range = bounds(src.len(), data_start, n).map_err(|_| ImportError::GlbChunk)?;
        match (chunks, kind) {
            (0, 0x4E4F534A) => json = Some(&src[range]),
            (1, 0x004E4942) => bin = Some(&src[range]),
            _ => return Err(ImportError::Unsupported),
        }
        at = data_start + n;
        chunks += 1;
    }
    if chunks != 2 {
        return Err(ImportError::Unsupported);
    }
    parse_document(
        json.ok_or(ImportError::GlbChunk)?,
        bin.ok_or(ImportError::GlbChunk)?,
        SourceFormat::Glb,
    )
}
pub fn import_gltf_json_with_supplied_buffer(
    json: &[u8],
    data: &[u8],
) -> Result<ImportedSurface, ImportError> {
    // The trusted caller supplies the bytes; no URI read or network egress occurs.
    parse_document(json, data, SourceFormat::GltfJsonExternalBuffer)
}
impl From<GeometryError> for ImportError {
    fn from(_: GeometryError) -> Self {
        ImportError::InvalidGeometry
    }
}

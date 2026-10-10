# BLOB-FRAME-v1 — independent viewer/runtime handoff

STATUS: `01A-runtime-implementation`, not yet approved as production ABI. Owner: CUDA-05 runtime. Claude owns the *viewer* and its test double; NEVER treat a fake frame as a real rendering qualification. No modifications to `~/Projects/blob.in-showcase` by the runtime worker.

**Transport**: AF_UNIX + SOCK_SEQPACKET, listening path `$XDG_RUNTIME_DIR/blobin-showcase/render.sock` (server creates parent directory mode `0700`), one complete JSON packet per control message, strict 16,384-byte maximum. Socket verifies `SO_PEERCRED` UID on each accepted connection. As both desktop viewer and preview service run under the same desktop UID in this prototype, this is **not** independent-principal isolation. No network, shell or ambient fallback.

Client sends `{"version":"BLOB-FRAME-v1","type":"HELLO"}`; server sends `{"version":"BLOB-FRAME-v1","type":"HELLO","server":"blobin-cuda05f-render-bridge","max_control_bytes":16384}`.

Client sends `{"version":"BLOB-FRAME-v1","type":"CAPABILITIES"}`; server sends enabled backend list and view modes. CUDA/Vulkan listed as *implemented*, not necessarily device-ready; success is only known after an actual compute.

Client sends `{"version":"BLOB-FRAME-v1","type":"PING"}`; server sends type `PONG`.

Client sends **one frame request** at a time:

```json
{
  "version": "BLOB-FRAME-v1", "type": "RENDER_REQUEST", "request_id": 101,
  "scene_revision": 7, "width_px": 512, "height_px": 384,
  "backend_requested": "cuda", "view_mode": "lit",
  "primitives": [
    {"x": 0.0, "y": 0.0, "radius": 0.19, "op": 0},
    {"x": 0.07, "y": 0.0, "radius": 0.10, "op": 3}
  ]
}
```

Fields are exact; unknown fields rejected. `request_id` and `scene_revision` are unsigned 64-bit, 0 allowed; width/height **physical pixels** 16–1024; product <=1,048,576; primitive count 1–64, finite coordinates [-2,2], radius (0,2], op `0=union`, `1=subtraction`, `2=intersection`, `3=smooth_union`; first primitive op MUST be 0. All fields required. Modes `lit`, `normals`, `distance`, `coverage`; no arbitrary provider library / executable paths in client input. No camera parameters, additional materials or variable controls in v1; viewer edits primitive fields to interact. Client SHOULD coalesce resize and scene events; bridge processes requests sequentially and does not reorder. A later `scene_revision` supersedes earlier frames **in the viewer**.

Server response contains **one** JSON `FRAME_READY` packet plus **exactly one** sealed read-only memfd with SCM_RIGHTS:

```json
{
  "version":"BLOB-FRAME-v1", "type":"FRAME_READY", "request_id":101,
  "scene_revision":7, "width_px":512, "height_px":384,
  "stride_bytes":2048, "format":"RGBA8_UNORM_OPAQUE", "buffer_bytes":786432,
  "backend_requested":"cuda", "backend_actual":"cuda", "view_mode":"lit",
  "generation": 1, "qualification":"CUDA05F_GATE_PASS", "cache_hit":false,
  "geometry_sha256":"<64 lowercase hex>"
}
```

**Cache provenance:** `FRAME_READY.cache_hit` is a Boolean. `true` means CUDA-04 reused previously computed samples for identical geometry on the same backend. This is a valid frame, not a worker failure, and the requested visualization is still applied to those samples. A changed `scene_revision` or visualization alone need not invalidate the SDF cache. Cached frames must not be misrepresented as new GPU kernel executions.

The FD length MUST equal `stride_bytes*height_px`; pixels are contiguous row-major RGBA8 with alpha=255 and stride exactly width*4, not premultiplied. Worker uses **actual CPU/CUDA/Vulkan SDF samples**, followed by CPU colorization for presentation. Modes are **2D signed field visualizations, not ray-traced 3D**. The `lit` mode is derived surface-normal shading, not a physically based lighting pipeline. Frame descriptor is a **sealed memfd**, not a GPU DMA-BUF; importing to GPU by the viewer may require a copy.

`RENDER_ERROR` has `{version,type,request_id,scene_revision,reason}`. The server never sends a successful frame if a worker fails, a gate fails, a timeout occurs, a packet is too large, a digest mismatches, or an output size/format is wrong. Fail-closed; never silently fall back from requested `cuda`/`vulkan` to `cpu`.

**Runtime authority and risk boundary:** GPU path runs the pre-existing CUDA-05F `IsolatedSupervisor` fixed-scene gate for approved GPU worker, then a separate **disposable frame worker** for this request, whose bytes and (for CUDA) provider library are pinned before spawn. The original 05F `RENDER` status command is unchanged. Frame workers run as a child of the same user (no seccomp, cgroups, separate UID or hostile-worker containment proof); the *05F lab qualified UID separation does not automatically transfer to this preview daemon*. Its `qualification` means fresh fixed-scene validation and the requested backend executed, NOT production GPU qualification.

`CAPABILITIES` is not a signed guarantee of device health. A disconnect closes memfd; GPU child timeout is 15 s. No public site/VPS changes. Complete native Wayland E2E must be done by Claude's viewer when it is ready.

# GPU-01 — physical Vulkan offscreen hybrid surface proof

Truth class: LOCALLY VERIFIED GPU PROBE, NOT CP02 checkpoint qualified.

This isolated experimental wgpu/Vulkan 30.0.1 binary imports the checked-in GLB tetrahedron, explicitly certifies convex geometry, uploads the original four triangles (and face planes) and runs an actual WGSL compute kernel that calculates exact Euclidean distance to the nearest triangle with convex inside/outside sign. Its hybrid field computes a tetrahedral MeshSolid minus an analytic sphere SDF, then raymarches the result to a 640x432 RGBA pixel buffer on the RTX 3070.

CPU↔GPU check: 1024 points x 3 fields = 3072 comparisons; maximum absolute error 1.938452514061e-7 on the v2 geometry, budget <=2e-4. All checked fields pass. No CPU substitution for GPU dispatch; CPU computes an independent oracle only for comparison.

Actual GPU-produced PNG: /mnt/workbench/evidence/runs/blobin-next-01/cp02/20261010-gpu01/gpu01-rendered-surface-v2.png
Actual Wayland window screenshot: /mnt/workbench/evidence/runs/blobin-next-01/cp02/20261010-gpu01/gpu01-wayland-window-screenshot.png (grim capture of temporary mpv display of GPU image, not live blob.in editor).

LIMITS: tetrahedron convex fixture only, no arbitrary nonconvex mesh shader, no compositor-presented realtime frames, no wp_presentation, no 60 FPS, no full CP02 seven gates. It is a genuine compute-produced 3D surface image, not an end-user interactive renderer. WGPU image is offscreen and the screenshot shows a static image inside mpv. Neither is proof of the CP03 presentation threshold.

Isolated source branch only. Do not push, merge, deploy or touch stable service.

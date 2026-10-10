# 08 — Experimental qualification / W08
Write-set: `docs/agent-results/upstream-rd-01/experiments.md`; unique evidence paths only. Treat unrun plans as NOT_RUN.
Order experiments by falsification cost, not hype. Prefer tiny read-only/low-CPU probes before GPU compilation:
E01 source retrieval / commit checksum / license contradiction; E02 hyperfine CLI availability and process timing; E03 MXC bwrap capability enforcement fixture; E04 Deno embedded native-op deny path; E05 GPUI Wayland focus semantics; E06 Uzu CPU feature/model support; E07 Plx transferable instrumentation design.
Every executed experiment records hardware/tool version, argv, bounded scope, stdout/stderr location, baseline, counterexample, result and limitations.
If toolchain absent, permission denied or upstream model incompatible, record BLOCKED or NOT_RUN and a minimal next gate; never install runtime dependencies automatically.
Heavy load must use protected/exclusive lab gating. Do not interfere with concurrently running CUDA/Wayland jobs or tests already closed.

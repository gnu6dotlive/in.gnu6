# 11 — Release and push gates / W11
G00: baseline root branch and clean isolated worktree; record historical/local divergence.
G01: six full 40-character source SHA pins acquired via GitHub and license metadata status explicit.
G02: six source-backed worker outputs or explicit NOT_RUN; do not claim independent reviewers were launched if they were not.
G03: cross-analysis coverage of all project surfaces and contradictions.
G04: bounded executable probes with captured output, or explicit NOT_RUN and reason.
G05: integrator checks source links, legal considerations, namespaces, no product authority expansion.
G06: public draft redacted, factual, versioned and not deployed without independent publication gate.
G07: `python3 -m json.tool` manifest/dispatch; source pin comparison; `git diff --check`; files in allowed write-set; commit on isolated branch; push explicit HEAD ref; verify `git ls-remote` equals local HEAD.
A partial batch can be PUSHED with missing gates marked open; it must not be labelled RELEASED or PROJECT_QUALIFIED.
Do not rebase remote/main, force-push, modify secrets, push 53 unrelated commits or run production mutation.

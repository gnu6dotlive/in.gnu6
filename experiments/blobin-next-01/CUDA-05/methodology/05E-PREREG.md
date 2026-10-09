# CUDA-05E-P01 — Preregistered host-principal identity gate

**Frozen before experimental acquisition.** Parent 05D result: subordinate `newuidmap` unavailable, namespace-local root mapped to host UID 1000. This is an alternative *disposable container* experiment, not proof that rootless subordinate maps were repaired.

## Question and hypotheses

Q: Can an ephemeral Unix-socket supervisor operate under a different **host UID** from its authorized client, and deny both a second client with access to the socket and an outsider blocked by discretionary access control (DAC)?

- H0: a namespace/container identifier alone is insufficient: the receiving server may see the same UID or all connected clients may be indiscriminately accepted.
- H1 (scoped): a server running inside a disposable, rootful-Docker-managed container as Linux UID 65534, with no userns remap, sees host client UID 1000 through SO_PEERCRED and denies UID 65533 even when GID 1000 permits the connection.
- The control for distinct host service identity is corroborated by container configuration UID 65534 and host kernel peer credentials from connected clients, not merely textual `id` in a namespace.
- A separately restricted UID 65532:GID65532 must fail to connect under DAC; inability to traverse a 0710 directory and 0660 socket is a **separate** barrier from the application's UID allowlist.

## Boundaries and method (precommitted)

- Use existing **local** `debian:trixie-slim` image (no download); disposable containers only, `--rm --network none --read-only --cap-drop ALL --security-opt no-new-privileges`, process and memory limits.
- Use a fresh uniquely named private experiment directory under `/mnt/workbench/build`, initially created and owned by the current user.
- Setup container, root **with CAP_CHOWN only**, is allowed to set **only the temporary experiment directory** owner to UID65534, GID=calling user's GID, mode 0710. This bounded Docker-root setup is **not** a proof of independence from the Docker daemon; it is not host account creation.
- The service container uses UID65534:GID(current user), creates socket mode 0660 (no world access), performs three finite accept cycles, and never accesses GPU or live services.
- Authorized host client (UID=current user, GID=current group): successful `STATUS` -> ALLOWED with UID verified by server's SO_PEERCRED.
- Deliberately rejected connected client from separate ephemeral container `--user 65533:<same group>`: connect succeeds under DAC; application denies by SO_PEERCRED UID.
- Disallowed-on-DAC client from `--user 65532:65532`: connection must fail before server accepts. This fourth attempted call is not counted among the three accept cycles.
- Final authorized host client must still be accepted, demonstrating no poisoned service state after rejection.
- The host must verify container userns settings and process identity; absent or ambiguous mapping means no host-principal qualification.
- Timeout all operations; force-remove only the experiment's own named containers; bounded cleanup of temporary path. No persistent Docker changes, networking, service reload, privileged host shell or password prompts.

## Predetermined pass/fail and stop

1. **Identity boundary** passes only if peer-authenticated host client UID equals the expected UID and differs from *configured actual service UID* 65534, and container uses host/disabled userns remap; UID65533 must appear in the rejected connected peer evidence (a configured-mismatch test alone does not pass).
2. **Socket DAC** passes only if the foreign-group client receives a permission error and server logs no accepted event for that attempt.
3. **Recovery after denial** passes only if an authorized client succeeds after the rejected connected attempt; this tests service continuity, not GPU recovery.
4. **Scientific result** is recorded as a one-run observation with exit codes, socket/parent mode, kernel peer credentials, Docker run and inspect configuration, and SHA-256 provenance.
5. **Disallowed interpretation:** production-ready agent authentication, resistance to daemon/root compromise, worker sandbox validation, driver fault handling, signed binary provenance, or all-Linux universality.
6. **STOP** immediately on non-ephemeral resource access, root host mutation outside the isolated experiment directory, lacking target image, or ambiguous host mapping. Report a partial/negative result without downgrading criteria.

## Data and publication

`05E-raw.json` private: mode/credentials/container configs/process exit codes/commands, error snippets and timestamps. `05E-derived.json` private: declared gates and interpretations. `05E-public.json` sanitized: only booleans/status, protocol commit, raw digest, limits and source availability. An independent validator must recompute gates and reject a deliberately forged UID. Public docs stay **DRAFT** until independent review; never include host PIDs/UIDs, user names, Docker socket paths or private traces.

**Preregistration is itself a release gate:** do not run the acquisition until this protocol and its measurement script have been committed.

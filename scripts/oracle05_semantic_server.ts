//! ORACLE-05: local read-only semantic observer. Not a network service.
//! Compile as an SSR-only Node artifact. Never interpret input as mutation.
// biome-ignore lint/correctness/noNodejsModules: dedicated local Node.js IPC qualification tool
import { createServer } from "node:net";
// biome-ignore lint/correctness/noNodejsModules: local Node-only filesystem and evidence store
import { existsSync, lstatSync, chmodSync, unlinkSync, openSync, closeSync, writeSync } from "node:fs";
// biome-ignore lint/correctness/noNodejsModules: private AF_UNIX socket location
import { dirname, isAbsolute } from "node:path";
import sceneFixture from "../experiments/blobin-next-01/CP02-VULKAN/fixtures/scene-stream.v1.json";
import type { RendererFrame } from "../src/renderer/contract";
import { applyRendererDelta } from "../src/renderer/delta";
import { asEntityId } from "../src/algebra/laws";
import { Oracle05SemanticSession } from "../src/interaction/wayland-optical-live";
// biome-ignore lint/correctness/noNodejsModules: only a CLI Node process
import process from "node:process";

const args = process.argv.slice(2);
function arg(name: string): string {
  const index = args.indexOf(name);
  if (index < 0 || !args[index + 1]) throw new Error("ORACLE05_MISSING_ARGUMENT_" + name);
  return args[index + 1]!;
}
const socketPath = arg("--socket");
const outputPath = arg("--output");
const seconds = Number(arg("--seconds"));
if (
  !isAbsolute(socketPath) ||
  !isAbsolute(outputPath) ||
  !Number.isInteger(seconds) ||
  seconds < 2 ||
  seconds > 90
) {
  throw new Error("ORACLE05_BAD_SOCKET_OUTPUT_OR_DURATION");
}
const folder = lstatSync(dirname(socketPath));
// biome-ignore lint/suspicious/noBitwiseOperators: POSIX 0700 permission mask
if (!folder.isDirectory() || (folder.mode & 0o077) !== 0 || existsSync(socketPath)) {
  throw new Error("ORACLE05_DIRECTORY_NOT_PRIVATE_OR_SOCKET_EXISTS");
}
let frame = sceneFixture.initial as unknown as RendererFrame;
for (const delta of sceneFixture.deltas) {
  frame = applyRendererDelta(frame, delta as never);
}
const observer = new Oracle05SemanticSession(frame, asEntityId("cavity-control"));
const fd = openSync(outputPath, "wx", 0o600);
let connected = false;
let lines = 0;
let failed = false;
let done = false;
let socketOpen = false;
let timer: NodeJS.Timeout | undefined;
function report(item: Record<string, unknown>): void {
  writeSync(fd, JSON.stringify(item) + "\n");
}
function terminate(reason: string): void {
  if (done) return;
  done = true;
  report({
    type: "summary",
    reason,
    packets: lines,
    pointer_events: observer.received,
    independent_photon_capture: "NOT_RUN",
    scene_mutation_authority: false,
    verified_live_render_pose: false,
    cp03_pass: false,
  });
  closeSync(fd);
  server.close();
  try {
    unlinkSync(socketPath);
  } catch {
    /* socket may already have been unlinked on shutdown */
  }
  if (timer) clearTimeout(timer);
  process.stdout.write(
    "ORACLE05_RECEIVER_EXIT " + reason + " pointer_events " + observer.received + "\n",
  );
}
const server = createServer((client) => {
  if (connected) {
    client.destroy();
    failed = true;
    terminate("UNEXPECTED_SECOND_CLIENT");
    return;
  }
  connected = true;
  socketOpen = true;
  let pending = "";
  let acceptedHeader = false;
  client.setEncoding("utf8");
  client.on("data", (chunk: string) => {
    if (failed || done) return;
    pending += chunk;
    if (Buffer.byteLength(pending, "utf8") > 4096) {
      failed = true;
      client.destroy();
      terminate("EXCESSIVE_UNDELIMITED_FRAME");
      return;
    }
    while (pending.includes("\n") && !done) {
      const next = pending.indexOf("\n");
      const line = pending.slice(0, next);
      pending = pending.slice(next + 1);
      if (Buffer.byteLength(line, "utf8") > 2048 || lines > 512 || !line) {
        failed = true;
        client.destroy();
        terminate("PACKET_BOUND_OR_EMPTY");
        return;
      }
      try {
        const event = JSON.parse(line);
        if (!acceptedHeader && event.type !== "hello") throw new Error("NO_FIRST_HANDSHAKE");
        if (acceptedHeader && event.type === "hello") throw new Error("DUPLICATE_HANDSHAKE");
        const result = observer.accept(event);
        acceptedHeader = true;
        report(result);
        lines++;
      } catch (error) {
        failed = true;
        report({
          type: "rejected",
          error: error instanceof Error ? error.message : "UNKNOWN",
          cp03_pass: false,
        });
        client.destroy();
        terminate("REJECTED_UNTRUSTED_EVENT");
        return;
      }
    }
  });
  client.on("error", () => {
    failed = true;
    terminate("UNIX_CLIENT_ERROR");
  });
  client.on("close", () => {
    socketOpen = false;
    if (!done) terminate(failed ? "CONNECTION_REJECTED" : "PRODUCER_DISCONNECTED");
  });
});
server.on("error", (err) => {
  process.stderr.write("ORACLE05_SERVER_ERROR " + err.message + "\n");
  terminate("SOCKET_ERROR");
});
server.listen(socketPath, () => {
  chmodSync(socketPath, 0o600);
  report({
    type: "observer_header",
    schema: "blobin.gpu05.semantic-observation.v1",
    listener: "private_af_unix",
    scene_frame_sequence: frame.sequence,
    calibration: "fixture_declared_not_physically_verified",
    source: "in.gnu6_InteractionProjector",
    cp03_pass: false,
  });
  process.stdout.write("ORACLE05_READY " + socketPath + " scene_sequence " + frame.sequence + "\n");
  timer = setTimeout(
    () => terminate(socketOpen ? "TIME_BOUNDED_WITH_CLIENT" : "TIME_BOUNDED_NO_CLIENT"),
    seconds * 1000,
  );
});
process.on("SIGTERM", () => terminate("SIGTERM"));

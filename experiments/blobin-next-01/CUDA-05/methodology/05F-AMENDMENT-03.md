# CUDA-05F — Amendment 03 (IPC client-loss safety)

**Protocol:** CUDA-05F-P01 at `03ba7945fe6740b2cf35dc28656ba866f66c33fb`.
**State:** post-acquisition intervention before a fourth run. The third raw run is retained as `05F-run03-peer-disconnect.json`.

## New disconfirming observation

In the third eight-scenario run, seven scenarios completed. The **Vulkan crash-injected scenario** failed at the IPC lifecycle: the client connection was lost and the Rust supervisor reported `CUDA05C_ERROR Broken pipe (os error 32)`, exited status 1, and refused later requests. Although CUDA/Vulkan normal GPU dispatches succeeded, this is a **real incomplete failure containment**. It is not evidence of driver/device loss, and no cause for the slow/disconnected client is asserted from the socket error alone.

## Repair before re-acquisition

The Rust service now treats a failed response write as a client lifecycle loss: **immediately revoke the active grant**, emit a private `CUDA05F_CLIENT_LOST` observation with the index and error kind, and continue to the next finite request instead of terminating the process. The client test timeout is increased from 20 seconds to 40 seconds to distinguish slow GPU context teardown from a broken lifecycle; timeout is still bounded, and the worker itself remains limited to 12 seconds (1 second for the injection test). Neither privilege configuration, predicates, gate definitions nor worker code are changed.

**No retroactive promotion:** run one remains Vulkan RED, run two lacks observed rejected peer UID, run three has IPC process failure. Run four is a separate new acquisition. A future independent negative test must exercise client disconnect under load and verify revocation before any production adoption.

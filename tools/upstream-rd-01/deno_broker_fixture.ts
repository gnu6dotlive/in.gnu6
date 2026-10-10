// Real Deno permission-broker IPC; only disposable file targets.
const target = Deno.args[0];
try {
  const value = await Deno.readTextFile(target);
  console.log(JSON.stringify({ok: true, matches: value === "GNU6_DENO_BROKER_ONLY\n"}));
} catch (error) {
  console.log(JSON.stringify({ok: false, name: error?.name ?? "Unknown", message: String(error?.message ?? error).slice(0, 180)}));
}

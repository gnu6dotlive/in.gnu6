// Deno 2.9.7 disposable permissions fixture. No host or production writes.
const [operation, target] = Deno.args;
try {
  if (operation === "read") {
    const text = await Deno.readTextFile(target);
    console.log(JSON.stringify({ ok: true, operation, matches_fixture: text === "GNU6-DENO-TEMP-FIXTURE\n" }));
  } else if (operation === "net") {
    const response = await fetch(target, { signal: AbortSignal.timeout(2000) });
    console.log(JSON.stringify({ ok: response.ok, operation, status: response.status }));
  } else {
    console.log(JSON.stringify({ ok: false, error: "UNKNOWN_OPERATION" }));
  }
} catch (error) {
  console.log(JSON.stringify({ ok: false, operation, name: error.name, message: String(error.message).slice(0, 200) }));
}

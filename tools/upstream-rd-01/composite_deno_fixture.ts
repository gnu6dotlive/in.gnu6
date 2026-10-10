// Disposable MXC × Deno permission gate. No network/production access.
const [target] = Deno.args;
try {
  await Deno.writeTextFile(target, "COMPOSITE_ALLOWED\n");
  console.log(JSON.stringify({ok:true,gate:"DENO_WRITE_COMPLETED"}));
} catch (error) {
  console.log(JSON.stringify({ok:false,name:error?.name ?? "Unknown",message:String(error?.message ?? error).slice(0,210)}));
}

// localhost-only network fixture, no external egress.
const [url] = Deno.args;
try {
  const response = await fetch(url, {signal: AbortSignal.timeout(1500)});
  console.log(JSON.stringify({ok:response.ok,status:response.status}));
} catch (error) {
  console.log(JSON.stringify({ok:false,name:error?.name ?? "Unknown",message:String(error?.message ?? error).slice(0,210)}));
}

# Scenario 11.1 – Initial page load bootstrap snapshot

On load, JS fetches a compressed Arrow IPC snapshot and hands bytes to WASM once.
```js
async function bootstrap() {
  const resp = await fetch("/init.arrow");
  const buf = new Uint8Array(await resp.arrayBuffer());
  replace(buf); // exported from Scenario 3
}
bootstrap();
```

Server prepares the snapshot at startup; WASM holds the table for later incremental updates.

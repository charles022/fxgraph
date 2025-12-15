# Scenario 5.1 – Large download (browser direct, no WASM)

Pure browser path: server exposes an HTTP URL; JS triggers download without touching WASM.
```js
document.getElementById("download").onclick = () => {
  window.location.href = "/exports/report.parquet";
};
```

No Rust/WASM allocation occurs; browser streaming and disk cache handle the payload.

import init, {
  id as stateId,
  ingest_snapshot,
  sum as stateSum,
  view_state,
  x as stateX,
  y as stateY,
} from "./pkg/client_wasm.js";

const statusEl = document.getElementById("status");
const idEl = document.getElementById("id");
const xEl = document.getElementById("x");
const yEl = document.getElementById("y");
const sumEl = document.getElementById("sum");
const fetchButton = document.getElementById("fetch");

function setStatus(text) {
  statusEl.textContent = text;
}

function render() {
  idEl.textContent = stateId();
  xEl.textContent = stateX().toFixed(2);
  yEl.textContent = stateY().toFixed(2);
  sumEl.textContent = stateSum().toFixed(2);
}

async function fetchSnapshot() {
  setStatus("Fetching /snapshot…");
  fetchButton.disabled = true;
  try {
    const resp = await fetch("/snapshot");
    const buf = await resp.arrayBuffer();
    const view = new Uint8Array(buf);

    const quickSum = view_state(view);
    await ingest_snapshot(view);
    render();

    setStatus(`Received ${view.length} bytes → sum = ${quickSum.toFixed(2)}`);
  } catch (err) {
    console.error(err);
    setStatus("Failed to fetch snapshot (see console)");
  } finally {
    fetchButton.disabled = false;
  }
}

async function main() {
  setStatus("Loading wasm bindings…");
  await init();
  setStatus("Ready – click fetch to load state");
  fetchButton.disabled = false;
}

fetchButton.addEventListener("click", fetchSnapshot);
main();

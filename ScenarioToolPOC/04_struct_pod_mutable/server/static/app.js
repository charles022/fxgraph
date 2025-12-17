import init, {
  id as stateId,
  ingest_snapshot,
  mutate_in_place,
  sum as stateSum,
  x as stateX,
  y as stateY,
} from "./pkg/client_wasm.js";

const statusEl = document.getElementById("status");
const idEl = document.getElementById("id");
const xEl = document.getElementById("x");
const yEl = document.getElementById("y");
const sumEl = document.getElementById("sum");
const connectBtn = document.getElementById("connect");
const mutateBtn = document.getElementById("mutate");

let socket = null;

function setStatus(text) {
  statusEl.textContent = text;
}

function renderState() {
  idEl.textContent = stateId();
  xEl.textContent = stateX().toFixed(2);
  yEl.textContent = stateY().toFixed(2);
  sumEl.textContent = stateSum().toFixed(2);
}

async function handleSnapshotMessage(event) {
  const bytes = new Uint8Array(await event.data.arrayBuffer());
  setStatus(`Received ${bytes.length} bytes — storing in WASM`);
  try {
    ingest_snapshot(bytes);
    renderState();
    mutateBtn.disabled = false;
    setStatus("Snapshot loaded; click mutate to change it in place");
  } catch (err) {
    console.error(err);
    setStatus("Failed to ingest snapshot (see console)");
  }
}

function connectWs() {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.close();
  }

  const scheme = location.protocol === "https:" ? "wss" : "ws";
  socket = new WebSocket(`${scheme}://${location.host}/ws`);
  socket.binaryType = "arraybuffer";
  setStatus("Connecting to WebSocket…");
  connectBtn.disabled = true;
  mutateBtn.disabled = true;

  socket.addEventListener("open", () => setStatus("Socket open — waiting for snapshot"));
  socket.addEventListener("message", handleSnapshotMessage);
  socket.addEventListener("close", () => {
    setStatus("Socket closed");
    connectBtn.disabled = false;
    mutateBtn.disabled = true;
  });
  socket.addEventListener("error", (err) => {
    console.error(err);
    setStatus("WebSocket error (see console)");
    connectBtn.disabled = false;
    mutateBtn.disabled = true;
  });
}

function mutateBuffer() {
  try {
    const newY = mutate_in_place();
    renderState();
    setStatus(`Mutated buffer inside WASM (y scaled to ${newY.toFixed(2)})`);
  } catch (err) {
    console.error(err);
    setStatus("Mutate failed (see console)");
  }
}

async function main() {
  setStatus("Loading wasm bindings…");
  connectBtn.disabled = true;
  mutateBtn.disabled = true;
  await init();
  setStatus("Ready — connect to fetch snapshot");
  connectBtn.disabled = false;
}

connectBtn.addEventListener("click", connectWs);
mutateBtn.addEventListener("click", mutateBuffer);
main();

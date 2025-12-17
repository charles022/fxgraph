import init, { ingest_snapshot, position, positions_len, score, tick } from "./pkg/client_wasm.js";

const tickEl = document.getElementById("tick");
const scoreEl = document.getElementById("score");
const positionsCountEl = document.getElementById("positions-count");
const positionsEl = document.getElementById("positions");
const statusEl = document.getElementById("status");

let socket;

function setStatus(text) {
  statusEl.textContent = text;
}

function render() {
  tickEl.textContent = tick();
  scoreEl.textContent = score();

  const count = positions_len();
  positionsCountEl.textContent = count;
  positionsEl.innerHTML = "";

  for (let i = 0; i < count; i++) {
    const pos = position(i);
    if (!pos) continue;
    const [x, y] = pos;
    const row = document.createElement("div");
    row.className = "pos";
    row.textContent = `#${i} → x: ${x.toFixed(2)} y: ${y.toFixed(2)}`;
    positionsEl.appendChild(row);
  }
}

function connect() {
  const protocol = location.protocol === "https:" ? "wss" : "ws";
  const url = `${protocol}://${location.host}/ws`;
  setStatus(`Connecting to ${url}…`);

  socket = new WebSocket(url);
  socket.binaryType = "arraybuffer";

  socket.onopen = () => setStatus("Connected – waiting for snapshots…");

  socket.onclose = () => {
    setStatus("Socket closed – retrying in 2s…");
    setTimeout(connect, 2000);
  };

  socket.onerror = () => setStatus("WebSocket error – see console");

  socket.onmessage = ({ data }) => {
    try {
      ingest_snapshot(new Uint8Array(data));
      render();
      setStatus("Live");
    } catch (err) {
      console.error("Failed to ingest snapshot", err);
      setStatus(`Decode error: ${err}`);
    }
  };
}

async function main() {
  setStatus("Loading wasm bindings…");
  await init();
  connect();
}

main();

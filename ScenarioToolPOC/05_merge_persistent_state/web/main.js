import init, {
  apply_delta,
  encode_delta,
  get_state_json,
  reset_state,
} from "../pkg/delta_state_poc.js";

class MockDeltaSocket {
  constructor(onMessage) {
    this.onMessage = onMessage;
    this.timer = null;
    this.tick = 0;
  }

  start() {
    if (this.timer) return;
    this.timer = setInterval(() => {
      const delta = this.buildDelta();
      this.onMessage({ data: delta });
    }, 900);
  }

  stop() {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  buildDelta() {
    this.tick += 1;
    const scoreInc = 1 + Math.floor(Math.random() * 5);
    const hops = Array.from({ length: 2 }, () => [
      parseFloat((Math.random() * 10).toFixed(2)),
      parseFloat((Math.random() * 10).toFixed(2)),
    ]);
    return encode_delta({ score_inc: scoreInc, new_positions: hops });
  }
}

const els = {};
const logLines = [];
const socket = new MockDeltaSocket(handleSocketMessage);

async function main() {
  await init();
  reset_state();
  cacheDom();
  bindControls();
  drawState();
  appendLog("ready: wasm initialized, stream idle");
}

function cacheDom() {
  els.score = document.querySelector("#score");
  els.positions = document.querySelector("#positions");
  els.log = document.querySelector("#log");
  els.start = document.querySelector("#start");
  els.stop = document.querySelector("#stop");
  els.reset = document.querySelector("#reset");
  els.single = document.querySelector("#single");
}

function bindControls() {
  els.start.addEventListener("click", () => {
    socket.start();
    appendLog("socket: start streaming deltas");
  });

  els.stop.addEventListener("click", () => {
    socket.stop();
    appendLog("socket: stopped");
  });

  els.reset.addEventListener("click", () => {
    socket.stop();
    reset_state();
    drawState();
    appendLog("state reset");
  });

  els.single.addEventListener("click", () => {
    const payload = socket.buildDelta();
    handleSocketMessage({ data: payload });
    appendLog("manual delta dispatched");
  });
}

function handleSocketMessage(event) {
  const ok = apply_delta(new Uint8Array(event.data));
  if (ok) {
    drawState();
  } else {
    appendLog("delta rejected");
  }
}

function drawState() {
  try {
    const state = JSON.parse(get_state_json());
    els.score.textContent = state.score;
    els.positions.innerHTML = "";
    state.positions.forEach((pos, idx) => {
      const li = document.createElement("li");
      li.textContent = `P${idx + 1}: (${pos[0].toFixed(2)}, ${pos[1].toFixed(
        2
      )})`;
      els.positions.appendChild(li);
    });
  } catch (err) {
    appendLog(`failed to draw state: ${err}`);
  }
}

function appendLog(line) {
  logLines.unshift(`[${new Date().toLocaleTimeString()}] ${line}`);
  logLines.splice(20);
  els.log.textContent = logLines.join("\n");
}

main();

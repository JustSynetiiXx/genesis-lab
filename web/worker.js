let wasm = null;
let handleId = null;
let running = false;
let tickCount = 0;
let lastTime = 0;
let tps = 0;

const TICKS_PER_BATCH = 10;
const MAP_INTERVAL = 5;
let batchesSinceMap = 0;

async function initWasm() {
  const { default: init, neue_simulation, tick, get_weltkarte, set_parameter } = await import('./pkg/genesis_lab_core.js');
  await init();
  wasm = { neue_simulation, tick, get_weltkarte, set_parameter };
  postMessage({ type: 'ready' });
}

function runLoop() {
  if (!running || !handleId) return;

  const now = performance.now();

  for (let i = 0; i < TICKS_PER_BATCH; i++) {
    const json = wasm.tick(handleId);
    tickCount++;

    if (i === TICKS_PER_BATCH - 1) {
      const elapsed = (now - lastTime) / 1000;
      if (elapsed > 0) {
        tps = Math.round(tickCount / elapsed);
      }

      const data = JSON.parse(json);
      data.tps = tps;

      batchesSinceMap++;
      if (batchesSinceMap >= MAP_INTERVAL) {
        const map = wasm.get_weltkarte(handleId);
        postMessage({ type: 'tick', data, map }, [map.buffer]);
        batchesSinceMap = 0;
      } else {
        postMessage({ type: 'tick', data });
      }
    }
  }

  if (now - lastTime >= 1000) {
    tickCount = 0;
    lastTime = now;
  }

  setTimeout(runLoop, 0);
}

onmessage = async function(e) {
  const msg = e.data;

  switch (msg.type) {
    case 'init':
      await initWasm();
      break;

    case 'start':
      if (!wasm) return;
      const config = msg.config || '{}';
      handleId = wasm.neue_simulation(config);
      running = true;
      tickCount = 0;
      lastTime = performance.now();
      batchesSinceMap = MAP_INTERVAL;
      runLoop();
      break;

    case 'stop':
      running = false;
      break;

    case 'set_parameter':
      if (wasm && handleId) {
        wasm.set_parameter(handleId, msg.key, msg.value);
      }
      break;
  }
};

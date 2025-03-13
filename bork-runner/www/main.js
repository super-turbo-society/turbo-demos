import initTurbo, * as turbo from "./pkg/turbo_genesis_impl_wasm_bindgen.js";

/**************************************************/
/* GAMEPAD SUPPORT                                */
/**************************************************/

class GamepadManager {
  constructor(canvas) {
    this.canvas = canvas;
    this.gamepads = {};
    this.prevButtonStates = {};
    this.axisStates = {
      ArrowUp: false,
      ArrowDown: false,
      ArrowLeft: false,
      ArrowRight: false,
    };
    this.init();
  }

  init() {
    window.addEventListener("gamepadconnected", (e) =>
      this.onGamepadConnected(e)
    );
    window.addEventListener("gamepaddisconnected", (e) =>
      this.onGamepadDisconnected(e)
    );
    this.poll();
  }

  onGamepadConnected(event) {
    const gamepad = event.gamepad;
    console.log(
      `Gamepad connected at index ${gamepad.index}: ${gamepad.id}. ${gamepad.buttons.length} buttons, ${gamepad.axes.length} axes.`
    );
    this.gamepads[gamepad.index] = gamepad;
    this.prevButtonStates[gamepad.index] = gamepad.buttons.map(
      (button) => button.pressed
    );
  }

  onGamepadDisconnected(event) {
    const gamepad = event.gamepad;
    console.log(
      `Gamepad disconnected from index ${gamepad.index}: ${gamepad.id}`
    );
    delete this.gamepads[gamepad.index];
    delete this.prevButtonStates[gamepad.index];
  }

  poll() {
    const connectedGamepads = navigator.getGamepads
      ? navigator.getGamepads()
      : navigator.webkitGetGamepads
      ? navigator.webkitGetGamepads()
      : [];

    for (let gp of connectedGamepads) {
      if (gp) {
        if (!this.gamepads[gp.index]) {
          this.onGamepadConnected({ gamepad: gp });
        } else {
          this.updateGamepadState(gp);
        }
      }
    }

    requestAnimationFrame(() => this.poll());
  }

  updateGamepadState(gamepad) {
    const prevStates = this.prevButtonStates[gamepad.index];
    gamepad.buttons.forEach((button, index) => {
      if (button.pressed !== prevStates[index]) {
        if (button.pressed) {
          this.dispatchButtonEvent(gamepad, index, "keydown");
        } else {
          this.dispatchButtonEvent(gamepad, index, "keyup");
        }
        this.prevButtonStates[gamepad.index][index] = button.pressed;
      }
    });

    // Handle axes (e.g., left stick)
    this.handleAxes(gamepad);
  }

  dispatchButtonEvent(gamepad, buttonIndex, eventType) {
    let keyEvent;
    switch (buttonIndex) {
      case 0: // A
        keyEvent = new KeyboardEvent(eventType, { key: "z", code: "KeyZ" });
        break;
      case 1: // B
        keyEvent = new KeyboardEvent(eventType, { key: "x", code: "KeyX" });
        break;
      case 12: // D-pad Up
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowUp",
          code: "ArrowUp",
        });
        break;
      case 13: // D-pad Down
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowDown",
          code: "ArrowDown",
        });
        break;
      case 14: // D-pad Left
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowLeft",
          code: "ArrowLeft",
        });
        break;
      case 15: // D-pad Right
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowRight",
          code: "ArrowRight",
        });
        break;
      // Add more mappings as needed
      default:
        return; // Unmapped button
    }
    console.log(keyEvent);

    this.canvas.dispatchEvent(keyEvent);
  }

  handleAxes(gamepad) {
    const threshold = 0.5;
    // Example: Left Stick Horizontal (axes[0]), Vertical (axes[1])
    const x = gamepad.axes[0];
    const y = gamepad.axes[1];

    // Horizontal
    if (x > threshold) {
      if (!this.axisStates.ArrowRight) {
        this.dispatchAxisEvent("ArrowRight", "keydown");
        this.axisStates.ArrowRight = true;
      }
    } else {
      if (this.axisStates.ArrowRight) {
        this.dispatchAxisEvent("ArrowRight", "keyup");
        this.axisStates.ArrowRight = false;
      }
    }

    if (x < -threshold) {
      if (!this.axisStates.ArrowLeft) {
        this.dispatchAxisEvent("ArrowLeft", "keydown");
        this.axisStates.ArrowLeft = true;
      }
    } else {
      if (this.axisStates.ArrowLeft) {
        this.dispatchAxisEvent("ArrowLeft", "keyup");
        this.axisStates.ArrowLeft = false;
      }
    }

    // Vertical
    if (y > threshold) {
      if (!this.axisStates.ArrowDown) {
        this.dispatchAxisEvent("ArrowDown", "keydown");
        this.axisStates.ArrowDown = true;
      }
    } else {
      if (this.axisStates.ArrowDown) {
        this.dispatchAxisEvent("ArrowDown", "keyup");
        this.axisStates.ArrowDown = false;
      }
    }

    if (y < -threshold) {
      if (!this.axisStates.ArrowUp) {
        this.dispatchAxisEvent("ArrowUp", "keydown");
        this.axisStates.ArrowUp = true;
      }
    } else {
      if (this.axisStates.ArrowUp) {
        this.dispatchAxisEvent("ArrowUp", "keyup");
        this.axisStates.ArrowUp = false;
      }
    }
  }

  dispatchAxisEvent(key, eventType) {
    const event = new KeyboardEvent(eventType, { key: key, code: key });
    this.canvas.dispatchEvent(event);
  }
}

/**************************************************/
/* WASM IMPORT PROXY                              */
/**************************************************/

// This proxy prevents WebAssembly.LinkingError from being thrown
// prettier-ignore
window.createWasmImportsProxy = (target = {}) => {
  console.log(target);
  return new Proxy(target, {
    get: (target, namespace) => {
      // Stub each undefined namespace with a Proxy
      target[namespace] = target[namespace] ?? new Proxy({}, {
        get: (_, prop) => {
          // Generate a sub function for any accessed property
          return (...args) => {
            console.log(`Calling ${namespace}.${prop} with arguments:`, args);
            // Implement the actual function logic here
          };
        }
      });
      return target[namespace];
    }
  })
};

/**************************************************/
/* GLOBAL STUBS                                   */
/**************************************************/

window.turboSolUser = window.turboSolUser ?? (() => null);
window.turboSolGetAccount = window.turboSolGetAccount ?? (async () => {});
window.turboSolSignAndSendTransaction =
  window.turboSolSignAndSendTransaction ?? (async () => {});

/**************************************************/
/* TOUCH CONTROLS                                 */
/**************************************************/

function initializeNipple(canvas) {
  const presses = {
    up: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowUp",
        code: "ArrowUp",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowUp",
        code: "ArrowUp",
      }),
    },
    down: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowDown",
        code: "ArrowDown",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowDown",
        code: "ArrowDown",
      }),
    },
    left: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowLeft",
        code: "ArrowLeft",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowLeft",
        code: "ArrowLeft",
      }),
    },
    right: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowRight",
        code: "ArrowRight",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowRight",
        code: "ArrowRight",
      }),
    },
  };
  let active = null;
  nipplejs
    .create({ dataOnly: true })
    .on("dir:up", (e) => {
      if (active && active !== presses.up) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.up.keydown);
      active = presses.up;
    })
    .on("dir:down", (e) => {
      if (active && active !== presses.down) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.down.keydown);
      active = presses.down;
    })
    .on("dir:left", (e) => {
      if (active && active !== presses.left) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.left.keydown);
      active = presses.left;
    })
    .on("dir:right", (e) => {
      if (active && active !== presses.right) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.right.keydown);
      active = presses.right;
    })
    .on("end", (e) => {
      if (active) {
        canvas.dispatchEvent(active.keyup);
      }
      active = null;
    });
  // Disable double-tap zoom on mobile
  document.addEventListener("dblclick", (e) => e.preventDefault());
}

/**************************************************/
/* FETCH WITH PROGRESS UTIL                       */
/**************************************************/

async function fetchWithProgress(init, cb) {
  const res = await fetch(init);
  const contentLength =
    res.headers.get("Content-Length") ??
    res.headers.get("x-goog-stored-content-length");
  if (!contentLength) return new Uint8Array(await res.arrayBuffer());
  const total = parseInt(contentLength, 10);
  let loaded = 0;
  // Pre-allocate the Uint8Array based on the known content length.
  const body = new Uint8Array(new ArrayBuffer(total));
  const reader = res.body.getReader();
  while (true) {
    const { done, value: chunk } = await reader.read();
    if (done) break;
    body.set(chunk, loaded);
    loaded += chunk.length;
    cb(loaded, total);
  }
  return body;
}

/**************************************************/
/* FILLTEXT                                       */
/**************************************************/

function fillText(context, msg) {
  console.log(msg);
  context.clearRect(0, 0, context.canvas.width, context.canvas.height);
  context.fillText(msg, context.canvas.width / 2, context.canvas.height / 2);
}

/**************************************************/
/* RUN TURBO GAME                                 */
/**************************************************/

async function run() {
  // Create the game's canvas
  const player = document.getElementById("player");

  // Initialize a temporary 2D context canvas for loading state
  const loading = document.createElement("canvas");
  player?.appendChild(loading);
  var context = loading.getContext("2d");
  context.fillStyle = "white";
  context.font = "bold 14px 04b03";
  context.textAlign = "center";
  context.textBaseline = "middle";

  // Download Turbo's WASM runtime
  const runtime = await fetchWithProgress(
    "pkg/turbo_genesis_impl_wasm_bindgen_bg.wasm",
    (loaded, total) => {
      const percent = Math.round((loaded / total) * 100);
      fillText(context, `Initializing runtime... ${percent}%`);
    }
  );

  // Initalize Turbo's WASM runtime
  await initTurbo({
    module_or_path: runtime.buffer,
  });

  // Fetch Turbo File
  let turbofile = await fetchWithProgress("main.turbo", (loaded, total) => {
    const percent = Math.round((loaded / total) * 100);
    fillText(context, `Loading game data... ${percent}%`);
  });

  // Decode Turbo File contents
  fillText(context, `Decompressing game data...`);
  let contents = turbo.decode_turbofile_v0_contents(new Uint8Array(turbofile));

  // Initialize context
  const canvas = document.createElement("canvas");
  canvas.width = 256;
  canvas.height = 144;

  // Initialize nipple (aka virtual analog stick)
  fillText(context, "Initializing touch controls...");
  initializeNipple(canvas);

  // Initialize Gamepad Support
  fillText(context, "Initializing gamepad...");
  const gamepadManager = new GamepadManager(canvas);
  gamepadManager.poll();

  // Remove loading state
  fillText(context, "Starting game...");
  player?.removeChild(loading);

  // Append game canvas
  player?.appendChild(canvas);

  // Run game
  await turbo.run(canvas, contents);
}

try {
  await run();
} catch (err) {
  console.error("Turbo failed to initialize", err);
}

import initTurbo, * as turbo from "./pkg/turbo_genesis_impl_wasm_bindgen.js";

/**************************************************/
/* CONFIGURATION                                  */
/**************************************************/

const APP_NAME = "Kiwi's Fruit Tree";
const APP_VERSION = "1.0.0";
const APP_AUTHOR = "Turbo";
const APP_DESCRIPTION = "Help Kiwi get his fruit back!";
const RESOLUTION = [384, 216];
const WASM_SRC = "kiwis_fruit_tree.wasm";

const SPRITES = ["./sprites/dirt_grass.png","./sprites/10_grass.png","./sprites/04_grass.png","./sprites/25_grass.png","./sprites/18_dirt.png","./sprites/67_stone.png","./sprites/09_stone_icon.png","./sprites/52_grass.png","./sprites/stone_pillar_bottom_001.png","./sprites/28_grass.png","./sprites/46_stone.png","./sprites/23_grass.png","./sprites/05_dirt.png","./sprites/58_drt.png","./sprites/stone_grass_001.png","./sprites/dirt.png","./sprites/54_grass.png","./sprites/40_stone.png","./sprites/31_dirt.png","./sprites/02_dirt.png","./sprites/30_grass.png","./sprites/24_grass.png","./sprites/kiwi_idle.png","./sprites/08_dirt.png","./sprites/66_stone.png","./sprites/47_stone.png","./sprites/37_dirt.png","./sprites/36_dirt.png","./sprites/53_grass.png","./sprites/29_grass.png","./sprites/22_grass.png","./sprites/62_dirt.png","./sprites/03_stone.png","./sprites/fruit_bowl_empty.png","./sprites/stone_grass_leftcorner_001.png","./sprites/41_stone.png","./sprites/57_dirt.png","./sprites/55_grass.png","./sprites/27_grass.png","./sprites/49_grass.png","./sprites/cloud_small.png","./sprites/12_stone.png","./sprites/68_stone.png","./sprites/33_dirt.png","./sprites/32_dirt.png","./sprites/06_stone.png","./sprites/50_grass.png","./sprites/stone_pillar_center_001.png","./sprites/44_stone.png","./sprites/39_dirt.png","./sprites/38_dirt.png","./sprites/65_stone.png","./sprites/stone_pillar_top_001.png","./sprites/sky.png","./sprites/14_grass.png","./sprites/21_stone.png","./sprites/kiwi_house.png","./sprites/63_stone.png","./sprites/59_dirt.png","./sprites/19_stone.png","./sprites/dirt_grass_002.png","./sprites/11_dirt.png","./sprites/56_grass.png","./sprites/42_stone.png","./sprites/48_stone.png","./sprites/26_grass.png","./sprites/fruit.png","./sprites/07_grass.png","./sprites/13_grass.png","./sprites/cloud_medium.png","./sprites/stone_grass_rightcorner_001.png","./sprites/45_stone.png","./sprites/51_grass.png","./sprites/17_dirt.png","./sprites/16_dirt.png","./sprites/60_dirt.png","./sprites/61_dirt.png","./sprites/dirt_grass_leftcorner_001.png","./sprites/64_stone.png","./sprites/dirt_grass_rightwall_001.png","./sprites/01_grass.png","./sprites/15_grass.png","./sprites/dirt_grass_leftwall_001.png","./sprites/34_dirt.png","./sprites/35_dirt.png","./sprites/20_stone_icon.png","./sprites/cloud_big.png","./sprites/fruit_tree.png","./sprites/dirt_grass_rightcorner_001.png","./sprites/43_stone.png",];

const SHADERS = [

];

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

window.turboSolUser = window.turboSolUser ?? (() => null);
window.turboSolGetAccount = window.turboSolGetAccount ?? (async () => {});
window.turboSolSignAndSendTransaction =
  window.turboSolSignAndSendTransaction ?? (async () => {});

/**************************************************/

try {
  // Initalize Turbo's WASM runtime
  await initTurbo();

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
  context.fillText("Loading...", loading.width / 2, loading.height / 2);

  // Fetch sprites
  const spriteData = await Promise.all(
    SPRITES.map(async (src) => {
      try {
        let res = await fetch(src);
        let buf = await res.arrayBuffer();
        return [
          src.replace(/^.*[\\/]/, "").replace(/.(png|jpg|jpeg|gif)$/, ""),
          buf,
        ];
      } catch (err) {
        console.error("Could not fetch sprite:", src);
        return null;
      }
    }).filter((x) => !!x)
  );

  // Fetch custom shaders
  const shaders = {
    main: null,
    surface: null,
  };
  for (const src of SHADERS) {
    if (src.endsWith("/surface.wgsl")) {
      try {
        let res = await fetch(src);
        let text = await res.text();
        shaders.surface = text;
      } catch (err) {
        console.error("Could not fetch shader:", src);
      }
    }
    if (src.endsWith("/main.wgsl")) {
      try {
        let res = await fetch(src);
        let text = await res.text();
        shaders.main = text;
      } catch (err) {
        console.error("Could not fetch shader:", src);
      }
    }
  }

  // Remove loading state
  player?.removeChild(loading);

  // Append game canvas
  const canvas = document.createElement("canvas");
  canvas.width = RESOLUTION[0];
  canvas.height = RESOLUTION[1];
  player?.appendChild(canvas);

  // Initialize nipple (aka virtual analog stick)
  initializeNipple(canvas);

  // Run game
  await turbo.run(canvas, spriteData, {
    source: WASM_SRC,
    meta: {
      appName: APP_NAME,
      appVersion: APP_VERSION,
      appAuthor: APP_AUTHOR,
      appDescription: APP_DESCRIPTION,
    },
    config: {
      resolution: RESOLUTION,
      shaders: shaders,
    },
  });
} catch (err) {
  console.error("Turbo failed to initialize", err);
}

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
    .create()
    .on("dir:up", (e) => {
      console.log(e);
      if (active && active !== presses.up) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.up.keydown);
      active = presses.up;
    })
    .on("dir:down", (e) => {
      console.log(e);
      if (active && active !== presses.down) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.down.keydown);
      active = presses.down;
    })
    .on("dir:left", (e) => {
      console.log(e);
      if (active && active !== presses.left) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.left.keydown);
      active = presses.left;
    })
    .on("dir:right", (e) => {
      console.log(e);
      if (active && active !== presses.right) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.right.keydown);
      active = presses.right;
    })
    .on("end", (e) => {
      console.log(e);
      if (active) {
        canvas.dispatchEvent(active.keyup);
      }
      active = null;
    });
}

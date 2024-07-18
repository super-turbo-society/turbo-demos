import initTurbo, * as turbo from "./pkg/turbo_genesis_host_wasm_bindgen.js";

/**************************************************/
/* CONFIGURATION                                  */
/**************************************************/

const APP_NAME = "";
const APP_VERSION = "";
const APP_AUTHOR = "";
const APP_DESCRIPTION = "";
const RESOLUTION = [384, 216];
const WASM_SRC = "tota.wasm";

const SPRITES = ["./sprites/bullet.png","./sprites/slime_spitter.png","./sprites/goldfish_gun.png","./sprites/auto_rifle.png","./sprites/lughead_small.png","./sprites/harpoon_bullet.png","./sprites/skull.png","./sprites/enemy_01_tires.png","./sprites/crap_stack.png","./sprites/title.png","./sprites/shoota_small.png","./sprites/zealots_small.png","./sprites/driver_frame.png","./sprites/boombox.png","./sprites/warboi.png","./sprites/road.png","./sprites/title_foreground.png","./sprites/meat_grinder.png","./sprites/the_persuader.png","./sprites/enemy_red_car.png","./sprites/arrow.png","./sprites/enemy_03_base.png","./sprites/enemy_02_base.png","./sprites/desert_bg.png","./sprites/psyko_juice.png","./sprites/geronimo_small.png","./sprites/fg_path.png","./sprites/can_of_worms.png","./sprites/boomer_bomb.png","./sprites/suzee.png","./sprites/zealots.png","./sprites/suzee_small.png","./sprites/skull_of_death.png","./sprites/mid_dunes.png","./sprites/lughead.png","./sprites/enemy_red_car_tires.png","./sprites/twiggy.png","./sprites/meatbag.png","./sprites/shoota.png","./sprites/enemy_green_truck_tires.png","./sprites/truck_tires.png","./sprites/teepee.png","./sprites/jailed_ducks.png","./sprites/enemy_blue_car_base.png","./sprites/main_grid_16x16.png","./sprites/explosion_small.png","./sprites/crooked_carburetor.png","./sprites/brutal_barrier.png","./sprites/enemy_02_tires.png","./sprites/war_boi.png","./sprites/enemy_gun_04.png","./sprites/twiggy_small.png","./sprites/enemy_01_base.png","./sprites/knuckle_buster.png","./sprites/truck.png","./sprites/truck_base.png","./sprites/enemy_gun_01.png","./sprites/enemy_green_truck_base.png","./sprites/enemy_blue_car_tires.png","./sprites/engine_shield.png","./sprites/the_ripper.png","./sprites/meatbag_small.png","./sprites/enemy_gun_02.png","./sprites/enemy_gun_03.png","./sprites/truck_shadow.png",];

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
    }).filter((x) => !!x),
  );

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

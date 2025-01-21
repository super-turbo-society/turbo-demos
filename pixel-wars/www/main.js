import initTurbo, * as turbo from "./pkg/turbo_genesis_impl_wasm_bindgen.js";

/**************************************************/
/* CONFIGURATION                                  */
/**************************************************/

const APP_NAME = "Pixel Wars";
const APP_VERSION = "1.0.0";
const APP_AUTHOR = "Turbo";
const APP_DESCRIPTION = "Epic Fantasy Battles of All Time";
const RESOLUTION = [384, 216];
const WASM_SRC = "pixel_wars.wasm";

const SPRITES = ["./sprites/status_rejuvenate.png","./sprites/tanker_attack.png","./sprites/bigpound_attack.png","./sprites/zombie_walk.png","./sprites/pyro_hit.png","./sprites/acidleak.png","./sprites/zombie_death.png","./sprites/draco_idle.png","./sprites/crater_01.png","./sprites/deathray_attack.png","./sprites/deathray_idle.png","./sprites/blood_16px_08.png","./sprites/shield_cheer.png","./sprites/tanker_idle.png","./sprites/axeman_idle.png","./sprites/sabre_walk.png","./sprites/darkknight_walk.png","./sprites/status_poisoned.png","./sprites/catapult_walk.png","./sprites/blade_walk.png","./sprites/pixelwars_title_static.png","./sprites/healing.png","./sprites/draco_cheer.png","./sprites/status_fleeing.png","./sprites/shield_death.png","./sprites/hunter_walk.png","./sprites/bigpound_walk.png","./sprites/flameboi_walk.png","./sprites/bazooka_attack.png","./sprites/zombie_cheer.png","./sprites/shield_walk.png","./sprites/draco_death.png","./sprites/saucer_walk.png","./sprites/status_burning.png","./sprites/yeti_walk.png","./sprites/pyro_walk.png","./sprites/zombie_attack.png","./sprites/status_healing.png","./sprites/hunter_hit.png","./sprites/bazooka_idle.png","./sprites/catapult_attack.png","./sprites/status_haste.png","./sprites/cosmo_walk.png","./sprites/you_lose_loop_02.png","./sprites/flameboi_attack.png","./sprites/status_frozen.png","./sprites/you_lose_loop_03.png","./sprites/blade_death.png","./sprites/shield_idle.png","./sprites/saucer_cheer.png","./sprites/bigpound_death.png","./sprites/cosmo_cheer.png","./sprites/landmine.png","./sprites/you_lose_loop_01.png","./sprites/bazooka_cheer.png","./sprites/flameboi_idle.png","./sprites/hunter_idle.png","./sprites/yeti_death.png","./sprites/bigpound_idle.png","./sprites/bazooka_walk.png","./sprites/catapult_death.png","./sprites/pyro_idle.png","./sprites/axeman_cheer.png","./sprites/status_bleeding.png","./sprites/hunter_attack.png","./sprites/cosmo_idle.png","./sprites/status_invincible.png","./sprites/spikes.png","./sprites/pyro_death.png","./sprites/darkknight_cheer.png","./sprites/sabre_death.png","./sprites/darkknight_attack.png","./sprites/flameboi_death.png","./sprites/deathray_cheer.png","./sprites/yeti_idle.png","./sprites/shield_attack.png","./sprites/saucer_idle.png","./sprites/hunter_death.png","./sprites/tanker_death.png","./sprites/poop.png","./sprites/blood_16px_02.png","./sprites/deathray_walk.png","./sprites/cosmo_death.png","./sprites/yeti_cheer.png","./sprites/cosmo_attack.png","./sprites/saucer_attack.png","./sprites/yeti_attack.png","./sprites/pyro_attack.png","./sprites/blood_16px_03.png","./sprites/bazooka_death.png","./sprites/blood_16px_01.png","./sprites/draco_walk.png","./sprites/sabre_attack.png","./sprites/blade_attack.png","./sprites/zombie_idle.png","./sprites/bigpound_cheer.png","./sprites/status_berserk.png","./sprites/blade_cheer.png","./sprites/saucer_death.png","./sprites/you_win_loop_03.png","./sprites/axeman_attack.png","./sprites/blood_16px_04.png","./sprites/explosion.png","./sprites/sabre_cheer.png","./sprites/catapult_idle.png","./sprites/blade_idle.png","./sprites/darkknight_death.png","./sprites/draco_attack.png","./sprites/hunter_cheer.png","./sprites/tanker_cheer.png","./sprites/blood_16px_05.png","./sprites/flameboi_cheer.png","./sprites/you_win_loop_02.png","./sprites/deathray_death.png","./sprites/sabre_idle.png","./sprites/catapult_cheer.png","./sprites/blood_16px_07.png","./sprites/axeman_death.png","./sprites/status_invisible.png","./sprites/tanker_walk.png","./sprites/axeman_walk.png","./sprites/pyro_cheer.png","./sprites/darkknight_idle.png","./sprites/blood_16px_06.png","./sprites/you_win_loop_01.png",];

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

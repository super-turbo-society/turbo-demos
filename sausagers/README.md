# Sausagers: The Game

![Sausagers: The Game title screen](./title_screen.png)

## Prerequisites

- [Install Turbo](https://turbo.computer/docs/quick-start)

## Development

**Note**: All commands shown in this section should be run from the root of the project directory.

### Documentation

- [Turbo CLI Reference](https://turbo.computer/docs/reference/cli/)
- [Turbo SDK Reference](https://turbo.computer/docs/reference/rust-sdk/)

### Running the game

This opens the game window natively. As you make changes to images in `/sprites` or modify code in `/src/lib.rs`, the game window will automatically update.

```sh
turbo run -w .
```

### Exporting for the web

Creates (or overwrite) a `www` directory with files that can be statically hosted on any web server.

```
turbo export \
    --app-name 'Sausagers: The Game' \
    --app-version '1.0.0' \
    --app-author 'Sausagers' \
    --app-description 'The greatest game in the world' \
    --app-resolution-x 256
    --app-resolution-y 387
```

After exporting, I recommend modifying the files in the `www` directory as-needed. You can probably remove the solana and service worker stuff. For example:

<details>
<summary><strong>Customizing the resolution</strong></summary>

In `www/main.js`, you could do something like this to support landscape and portrait and give the game a minimum size for the width or height:

```js
// The game's resolution
let RESOLUTION = [
  Math.floor(window.innerWidth / 3),
  Math.floor(window.innerHeight / 3),
];
const min_size = 256;
// landscape
if (window.innerWidth > window.innerHeight) {
  const ratio = window.innerWidth / window.innerHeight;
  RESOLUTION[0] = Math.floor(ratio * min_size);
  RESOLUTION[1] = min_size;
}
// portrait
else {
  const ratio = window.innerHeight / window.innerWidth;
  RESOLUTION[0] = min_size;
  RESOLUTION[1] = Math.floor(ratio * min_size);
}
```
</details>

<details>
<summary><strong>Customizing the virtual gamepad</strong></summary>

In `index.html` you can modify the gamepad to have a circular dpad and a single button for shooting:

```html
<div id="virtual-gamepad">
  <div class="circle">
    <div
      ontouchstart="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowUp',code:'ArrowUp'}));"
      ontouchend="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup',{key:'ArrowUp',code:'ArrowUp'}));"
    ></div>
    <div
      ontouchstart="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',code:'ArrowRight'}));"
      ontouchend="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup',{key:'ArrowRight',code:'ArrowRight'}));"
    ></div>
    <div
      ontouchstart="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowLeft',code:'ArrowLeft'}));"
      ontouchend="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup',{key:'ArrowLeft',code:'ArrowLeft'}));"
    ></div>
    <div
      ontouchstart="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',code:'ArrowDown'}));"
      ontouchend="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup',{key:'ArrowDown',code:'ArrowDown'}));"
    ></div>
  </div>
  <button
    class="a"
    ontouchstart="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown',{key:'z',code:'KeyZ'}));"
    ontouchend="event.preventDefault();document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup',{key:'z',code:'KeyZ'}));"
  >
    A
  </button>
</div>
```

You'd also replace the CSS in `style.css`:

```css
@font-face {
  font-family: "04b03";
  src: url("/04b03.ttf");
}

* {
  box-sizing: border-box;
}

:root {
  --doc-height: 100vh;
  --button-size: 120px;
  --button-font-size: 12px;
  --start-select-width: 50px;
  --start-select-height: 20px;
  --start-select-font-size: 8px;
  --gamepad-opacity: 0.25;
}

body {
  touch-action: none;
  display: flex;
  flex-direction: column;
  justify-content: center;
  /* align-items: center; */
  font-family: "04b03", monospace;
  align-items: flex-start;
  height: var(--doc-height);
  max-height: -webkit-fill-available;
  width: 100vw;
  margin: 0;
  background: #202124;
}

button {
  font-family: "04b03", monospace;
}

#player {
  position: relative;
  display: flex;
  flex: 1;
  width: 100vw;
  height: 100%;
  justify-content: center;
  align-items: center;
  background: #202124;
  user-select: none;
}

canvas {
  width: 100% !important;
  height: 100% !important;
  object-fit: contain;
  margin: auto;
  background-color: #202124;
  image-rendering: pixelated;
  outline: none;
}

p {
  color: #fff;
  font-size: 100px;
}

.circle {
  width: 180px;
  height: 180px;
  display: flex;
  flex-flow: row wrap;
  transform: translate(-20px, -48px) rotate(45deg);
}

.circle div {
  height: 90px;
  width: 90px;
  background-color: #fff;
  color: #000;
  border: 2px solid rgba(0, 0, 0, 0.1);
}

.circle div:nth-child(1) {
  border-radius: 90px 0 0 0;
}
.circle div:nth-child(2) {
  border-radius: 0 90px 0 0;
}
.circle div:nth-child(3) {
  border-radius: 0 0 0 90px;
}
.circle div:nth-child(4) {
  border-radius: 0 0 90px 0;
}

#virtual-gamepad {
  position: absolute;
  bottom: 60px;
  left: 10px;
  right: 10px;
  display: none;
  max-width: 100vw;
  height: 120px;
  margin: auto 20px;
  z-index: 1;
  opacity: var(--gamepad-opacity);
}
@media (hover: none) {
  #virtual-gamepad {
    display: flex;
  }
}

#virtual-gamepad button {
  position: absolute;
  width: var(--button-size);
  height: var(--button-size);
  color: #000;
  font-size: var(--button-font-size);
  box-shadow: 0 0 0px 4px #000;
  background: #fff;
  user-select: none;
  padding: 0;
  transition: transform 0.5s ease;
}

#virtual-gamepad button:active {
  transform: translateY(10px) scale(0.9);
}

#virtual-gamepad button.up,
#virtual-gamepad button.down,
#virtual-gamepad button.left,
#virtual-gamepad button.right {
  bottom: var(--button-size);
  left: var(--button-size);
  border-radius: 10px;
}

#virtual-gamepad button.up {
  margin-bottom: var(--button-size);
}

#virtual-gamepad button.down {
  margin-bottom: calc(-1 * var(--button-size));
}

#virtual-gamepad button.left {
  margin-left: calc(-1 * var(--button-size));
}

#virtual-gamepad button.right {
  margin-left: var(--button-size);
}

#virtual-gamepad button.a,
#virtual-gamepad button.b,
#virtual-gamepad button.x,
#virtual-gamepad button.y {
  bottom: var(--button-size);
  right: var(--button-size);
  border-radius: 100%;
}

#virtual-gamepad button.a {
  margin-right: calc(-1 * var(--button-size));
  top: -14px;
  right: 100px;
}

#virtual-gamepad button.b {
  margin-bottom: calc(-1 * var(--button-size));
}

#virtual-gamepad button.x {
  margin-bottom: var(--button-size);
}

#virtual-gamepad button.y {
  margin-right: var(--button-size);
}

#virtual-gamepad button.start,
#virtual-gamepad button.select {
  left: 0;
  right: 0;
  bottom: 0;
  margin: auto;
  border-radius: 24px;
  width: var(--start-select-width);
  height: var(--start-select-height);
  font-size: var(--start-select-font-size);
}

#virtual-gamepad button.start {
  left: calc(-1 * var(--start-select-width) + -10px);
}

#virtual-gamepad button.select {
  right: calc(-1 * var(--start-select-width) + -10px);
}
```
</details>

## Deployment

You can take the exported `www` dir files and put them into any static hosting service to deploy to the web. But if you want something free + quick-n-dirty + collaborative, [fork this codesandbox](https://codesandbox.io/p/devbox/tender-mclean-s27lh8) and replace the contents of the `public` dir with the contents of the `www` dir.

## Debugging

If the game doesn't run on the web, it's usually due to one of the following:

1. The request for the game's wasm file is failing. Usually, because the `WASM_SRC` var in `main.js` doesn't match the wasm file being requested.

2. Requests for sprites are failing. Usually because the `SPRITES` var in `main.js` has some incorrect paths to files or includes files that don't exist in the `sprites` dir.

3. Decoding sprites is failing. This happens when there's a non-sprite in the `SPRITES` var in `main.js`. In particular, look out for `.DS_Store` sneaking in there. `turbo export` is a bit new, so there are some small footguns still.

When in doubt, double-check the files in the `www` dir. Particularly `main.js` and `index.html`.
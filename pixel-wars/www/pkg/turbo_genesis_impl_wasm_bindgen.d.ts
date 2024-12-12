/* tslint:disable */
/* eslint-disable */
/**
* @param {HTMLCanvasElement} canvas
* @param {Array<any>} sprites
* @param {RunOptions} opts
* @returns {Promise<void>}
*/
export function run(canvas: HTMLCanvasElement, sprites: Array<any>, opts: RunOptions): Promise<void>;
/**
* Handler for `console.log` invocations.
*
* If a test is currently running it takes the `args` array and stringifies
* it and appends it to the current output of the test. Otherwise it passes
* the arguments to the original `console.log` function, psased as
* `original`.
* @param {Array<any>} args
*/
export function __wbgtest_console_log(args: Array<any>): void;
/**
* Handler for `console.debug` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_debug(args: Array<any>): void;
/**
* Handler for `console.info` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_info(args: Array<any>): void;
/**
* Handler for `console.warn` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_warn(args: Array<any>): void;
/**
* Handler for `console.error` invocations. See above.
* @param {Array<any>} args
*/
export function __wbgtest_console_error(args: Array<any>): void;
export interface AppMetadata {
    appAuthor: string | null;
    appDescription: string | null;
    appName: string | null;
    appVersion: string | null;
}

export interface AppConfiguration {
    fps: number | null;
    resolution: [number, number] | null;
    tileSize: [number, number] | null;
    shaders: ShaderConfig;
}

export interface ShaderConfig {
    main: string | null;
    surface: string | null;
}

export interface RunOptions {
    source: string;
    meta: AppMetadata | null;
    config: AppConfiguration | null;
}

/**
* Runtime test harness support instantiated in JS.
*
* The node.js entry script instantiates a `Context` here which is used to
* drive test execution.
*/
export class WasmBindgenTestContext {
  free(): void;
/**
* Creates a new context ready to run tests.
*
* A `Context` is the main structure through which test execution is
* coordinated, and this will collect output and results for all executed
* tests.
*/
  constructor();
/**
* Inform this context about runtime arguments passed to the test
* harness.
* @param {any[]} args
*/
  args(args: any[]): void;
/**
* Executes a list of tests, returning a promise representing their
* eventual completion.
*
* This is the main entry point for executing tests. All the tests passed
* in are the JS `Function` object that was plucked off the
* `WebAssembly.Instance` exports list.
*
* The promise returned resolves to either `true` if all tests passed or
* `false` if at least one test failed.
* @param {any[]} tests
* @returns {Promise<any>}
*/
  run(tests: any[]): Promise<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run: (a: number, b: number, c: number) => number;
  readonly wgpu_compute_pass_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_compute_pass_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_compute_pass_set_push_constant: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_compute_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_push_debug_group: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_pop_debug_group: (a: number) => void;
  readonly wgpu_compute_pass_write_timestamp: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_begin_pipeline_statistics_query: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_end_pipeline_statistics_query: (a: number) => void;
  readonly wgpu_compute_pass_dispatch_workgroups: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_compute_pass_dispatch_workgroups_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_bundle_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_bundle_draw_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_render_pass_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_vertex_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_draw: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_draw_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_draw_indexed_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_multi_draw_indirect: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_render_pass_multi_draw_indexed_indirect: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_render_pass_multi_draw_indirect_count: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_multi_draw_indexed_indirect_count: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_set_blend_constant: (a: number, b: number) => void;
  readonly wgpu_render_pass_set_scissor_rect: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wgpu_render_pass_set_stencil_reference: (a: number, b: number) => void;
  readonly wgpu_render_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_push_debug_group: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_pop_debug_group: (a: number) => void;
  readonly wgpu_render_pass_write_timestamp: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_begin_occlusion_query: (a: number, b: number) => void;
  readonly wgpu_render_pass_end_occlusion_query: (a: number) => void;
  readonly wgpu_render_pass_begin_pipeline_statistics_query: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_end_pipeline_statistics_query: (a: number) => void;
  readonly wgpu_render_pass_execute_bundles: (a: number, b: number, c: number) => void;
  readonly wgpu_render_bundle_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
  readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
  readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
  readonly wgpu_render_pass_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_wasmbindgentestcontext_free: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_new: () => number;
  readonly wasmbindgentestcontext_args: (a: number, b: number, c: number) => void;
  readonly wasmbindgentestcontext_run: (a: number, b: number, c: number) => number;
  readonly __wbgtest_console_log: (a: number) => void;
  readonly __wbgtest_console_debug: (a: number) => void;
  readonly __wbgtest_console_info: (a: number) => void;
  readonly __wbgtest_console_warn: (a: number) => void;
  readonly __wbgtest_console_error: (a: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd78e770e95449d3f: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6a13323d0b62b505: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h623122c64a2b1052: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h76fba3c2b7303611: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h30af0d3aace084b2: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd8bfa0201d3bb99c: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3bb6f0960bd2c1b4: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4679d841afe6d1dd: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4a3169a81ff1c832: (a: number, b: number, c: number, d: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h9a33d3af177742ad: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h20adf445c91fe78b: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h42b0be5d02556461: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J_K_L___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h577c24be08d29fb0: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h333a4f5c40bf14f7: (a: number, b: number, c: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h19e889c1496f7c3e: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h06b5273b65a89154: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2415369b1a83afd0: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
  readonly _dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc6422e781f8c6a70: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h838d9b1021920e20: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h503d116f9a777473: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0163129d5332f486: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he5165488fbaaa92e: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0061450e0e82dfbb: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h785d0c0e11cfe9f7: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1178f536abafd290: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3a22a842e492117a: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures__invoke3_mut__h60ab9e8431bdefc9: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h018c6610aa0611d9: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

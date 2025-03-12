/* tslint:disable */
/* eslint-disable */
/**
* @param {Uint8Array} bytes
* @returns {TurboFileV0Contents}
*/
export function decode_turbofile_v0_contents(bytes: Uint8Array): TurboFileV0Contents;
/**
* @param {TurboFileV0Contents} contents
* @returns {Promise<TurboApp>}
*/
export function create_app_from_v0_contents(contents: TurboFileV0Contents): Promise<TurboApp>;
/**
* @param {HTMLCanvasElement} canvas
* @param {TurboFileV0Contents} v0_contents
* @returns {Promise<void>}
*/
export function run(canvas: HTMLCanvasElement, v0_contents: TurboFileV0Contents): Promise<void>;
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
export interface ShaderConfig {
    canvas: Record<string, string>;
    surface: Record<string, string>;
}

/**
*/
export class TurboApp {
  free(): void;
}
/**
*/
export class TurboFileV0Contents {
  free(): void;
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
  readonly __wbg_turbofilev0contents_free: (a: number, b: number) => void;
  readonly __wbg_turboapp_free: (a: number, b: number) => void;
  readonly decode_turbofile_v0_contents: (a: number, b: number) => void;
  readonly create_app_from_v0_contents: (a: number) => number;
  readonly run: (a: number, b: number) => number;
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
  readonly wgpu_render_pass_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
  readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
  readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
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
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h161c54fd178bf91d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2ae8ed5115181fdd: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf6dcca4c44ccf724: (a: number, b: number, c: number, d: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0a49c8f77d43b3d6: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => number;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h12ac25ec3fa362e6: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h04c9fb4110131864: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0e0bacd9500fee95: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he42895901169c1c9: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3c7680a3352c13e9: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h05eb9c53ee2e00b4: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdd9bd477561aa61d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h36ae4b302def00bd: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J_K_L___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha7b44fff6ce86f15: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7cfb047f9aa6b2ac: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1fda849736229213: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbb720ec6edcd4f5f: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c3144be7a60a728: (a: number, b: number, c: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb00baa4ac20ddb87: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => number;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha20d4f629fac8293: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hfd30a9336f657bda: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdc147d6638f1275f: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h399dd21fa179106a: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h772b66740572b8d1: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hccef0dcde61e042d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1691d30387bd8462: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdd6c5ec5a8be9079: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb9dd283590838b58: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h22df361fb6088299: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures__invoke3_mut__h70ff84634a24fb2b: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h5345ea8aea305903: (a: number, b: number, c: number, d: number) => void;
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

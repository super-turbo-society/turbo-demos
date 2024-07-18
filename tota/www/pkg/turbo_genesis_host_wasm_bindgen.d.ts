/* tslint:disable */
/* eslint-disable */
/**
* @param {HTMLCanvasElement} canvas
* @param {Array<any>} sprites
* @param {RunOptions} opts
* @returns {Promise<void>}
*/
export function run(canvas: HTMLCanvasElement, sprites: Array<any>, opts: RunOptions): Promise<void>;
export interface RunOptions {
    source: string;
    meta: AppMetadata | null;
    config: AppConfiguration | null;
}

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
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hac939693f8874440: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h285b2ac368611e19: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6bfa3ff8a2c3eb33: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h00db23992091380f: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h15cb1e719d7d63fc: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb329e375d4f8dd0e: (a: number, b: number, c: number, d: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf8e894a4d2af51a7: (a: number, b: number) => number;
  readonly _dyn_core__ops__function__Fn__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf5d22f8b28421f65: (a: number, b: number, c: number, d: number) => number;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6d0841ce525153be: (a: number, b: number, c: number) => number;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he0671150e9601ec2: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha9c0a3e8a62b1134: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4003979d8cd15bab: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd5b612a5ef4d4f60: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D_E_F_G_H_I_J___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb865d715bdf2d2a2: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
  readonly _dyn_core__ops__function__Fn__A_B_C_D___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4100f73e9a750b4b: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasm_bindgen__convert__closures__invoke1_mut__h1e0639c385801739: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures__invoke0_mut__h63e78c5f93000380: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures__invoke1_mut__h10dd7b2fac2af1c0: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h70b4ae5db8810aac: (a: number, b: number, c: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h21117bfbc70a7d76: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;

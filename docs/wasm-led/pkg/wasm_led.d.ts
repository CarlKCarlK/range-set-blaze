/* tslint:disable */
/* eslint-disable */
/**
*/
export function on_module_load(): void;
/**
* @param {number} movie_id
* @param {number} now_milliseconds
* @returns {LedState}
*/
export function get_led_state_and_duration(movie_id: number, now_milliseconds: number): LedState;
/**
*/
export class LedState {
  free(): void;
/**
*/
  duration: number;
/**
*/
  state: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_ledstate_free: (a: number) => void;
  readonly __wbg_get_ledstate_state: (a: number) => number;
  readonly __wbg_set_ledstate_state: (a: number, b: number) => void;
  readonly __wbg_get_ledstate_duration: (a: number) => number;
  readonly __wbg_set_ledstate_duration: (a: number, b: number) => void;
  readonly on_module_load: () => void;
  readonly get_led_state_and_duration: (a: number, b: number) => number;
  readonly __wbindgen_start: () => void;
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

/* tslint:disable */
/* eslint-disable */
export function initialize(): void;
/**
 * Public entry point for consumers.
 */
export class Engine {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Haal de fouten van de laatste evaluatie op als leesbare strings.
   */
  get_errors(): any;
  /**
   * Haal input controls (sliders, toggles) op voor UI-generatie.
   */
  get_sliders(): any;
  /**
   * Haalt de geometrie op van de laatste evaluatie in een "diff" formaat.
   */
  get_geometry(): any;
  get_node_info(): any;
  /**
   * Geeft terug of de engine de minimale initialisatie heeft doorlopen.
   */
  is_initialized(): boolean;
  /**
   * Haalt een tekstuele weergave op van de topologisch gesorteerde graaf.
   */
  get_topology_map(): any;
  /**
   * Stel een slider- of togglewaarde in op basis van id of naam.
   */
  set_slider_value(id_or_name: string, value: any): void;
  constructor();
  /**
   * Evalueer de geladen graph.
   */
  evaluate(): void;
  /**
   * Laad een GHX-bestand in de engine en prepareer slider-informatie.
   */
  load_ghx(xml: string): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_engine_free: (a: number, b: number) => void;
  readonly engine_evaluate: (a: number) => [number, number];
  readonly engine_get_errors: (a: number) => [number, number, number];
  readonly engine_get_geometry: (a: number) => [number, number, number];
  readonly engine_get_node_info: (a: number) => [number, number, number];
  readonly engine_get_sliders: (a: number) => [number, number, number];
  readonly engine_get_topology_map: (a: number) => [number, number, number];
  readonly engine_is_initialized: (a: number) => number;
  readonly engine_load_ghx: (a: number, b: number, c: number) => [number, number];
  readonly engine_new: () => number;
  readonly engine_set_slider_value: (a: number, b: number, c: number, d: any) => [number, number];
  readonly initialize: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
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

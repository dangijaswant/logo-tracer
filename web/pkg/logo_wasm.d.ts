/* tslint:disable */
/* eslint-disable */

/**
 * Tunable options passed from JS (all optional; sensible defaults applied).
 */
export class LogoOptions {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Defaults matching the CLI `--preset logo`.
     */
    constructor();
    color_precision: number;
    corner_threshold_deg: number;
    filter_speckle: number;
    max_colors: number;
    palette_merge_threshold: number;
    precision: number;
}

/**
 * Trace raw RGBA pixels (length must be w*h*4) into a logo-mode SVG string.
 */
export function trace_logo(rgba: Uint8Array, width: number, height: number, opts: LogoOptions): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_logooptions_color_precision: (a: number) => number;
    readonly __wbg_get_logooptions_corner_threshold_deg: (a: number) => number;
    readonly __wbg_get_logooptions_filter_speckle: (a: number) => number;
    readonly __wbg_get_logooptions_max_colors: (a: number) => number;
    readonly __wbg_get_logooptions_palette_merge_threshold: (a: number) => number;
    readonly __wbg_get_logooptions_precision: (a: number) => number;
    readonly __wbg_logooptions_free: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_color_precision: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_corner_threshold_deg: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_filter_speckle: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_max_colors: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_palette_merge_threshold: (a: number, b: number) => void;
    readonly __wbg_set_logooptions_precision: (a: number, b: number) => void;
    readonly logooptions_new: () => number;
    readonly trace_logo: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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

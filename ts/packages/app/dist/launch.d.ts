import { AppShell, threeRendererPort, type AppBridgeBoot, type FixtureChoice, type HostCapabilities, type ShellReadout } from './shell.js';
/** Which runtime the launch targets (mirrors the smoke harness intents). */
export type LaunchMode = 'reference' | 'authority';
/** The documented dev/launch commands (referenced by the host READMEs and Den docs). */
export declare const SHELL_LAUNCH_COMMAND = "pnpm --filter @asha/app dev:asha-shell";
export declare const AUTHORITY_SHELL_LAUNCH_COMMAND = "ASHA_SHELL_MODE=authority pnpm --filter @asha/app dev:asha-shell";
/** The headless host descriptor (model-only; no real a11y tree to render into). */
export declare function headlessHost(): HostCapabilities;
/**
 * The reference boot: the deterministic mock facade, while *probing* native
 * availability for an honest readout. The reference path never depends on the addon.
 */
export declare function referenceBoot(): AppBridgeBoot;
/**
 * The authority boot: attempt the real native path. If the addon is not loadable, the
 * boot fails *closed* with a classified error — the shell reports `unavailable`, never a
 * silent downgrade to the mock.
 */
export declare function authorityBoot(): AppBridgeBoot;
/** Pick a boot strategy from an explicit launch mode. */
export declare function bootForMode(mode: LaunchMode): AppBridgeBoot;
/**
 * The canonical runtime-selectable fixture catalog for the launch. Two fixtures prove
 * selection is data (runtime), not a compile-time switch. `launch-grid` is the seeded
 * launch world (grid 1, materials 1–3 — matching the reference bridge seed); `alt-grid`
 * is a second world with a single material to exercise palette/fixture switching.
 */
export declare function defaultFixtures(): FixtureChoice[];
/** Options for {@link runHeadlessLaunch} (all injectable for tests). */
export interface HeadlessLaunchOptions {
    readonly mode?: LaunchMode;
    readonly host?: HostCapabilities;
    readonly fixtures?: readonly FixtureChoice[];
    readonly initialFixtureId?: string;
    /** Override the bridge boot directly (tests inject degraded/unavailable). */
    readonly bootBridge?: () => AppBridgeBoot;
    /** Inject a renderer port; defaults to a real headless three renderer. */
    readonly renderer?: ReturnType<typeof threeRendererPort> | null;
}
/**
 * Compose the shell for a headless launch and drive load → projection so the returned
 * readout reflects a real assembled run. The shell instance is returned alongside the
 * readout so callers (tests) can drive further interactions.
 */
export declare function launchShell(options?: HeadlessLaunchOptions): AppShell;
/** Run the headless launch and return the deterministic readout. */
export declare function runHeadlessLaunch(options?: HeadlessLaunchOptions): ShellReadout;
//# sourceMappingURL=launch.d.ts.map
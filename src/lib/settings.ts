import { persisted } from "svelte-persisted-store";
import themes, { type ThemeName, defaultTheme } from "./ui/themes";
import { derived, type Readable } from "svelte/store";

export type Settings = {
  name: string;
  theme: ThemeName;
  scrollback: number;
  background: string; // board background color (CSS color string)
  tileCols: number; // remembered custom column count for the layout menu
  snippets: string[]; // saved command snippets, click-to-paste into a terminal
};

export const DEFAULT_BACKGROUND = "#0e0e10";
export const DEFAULT_SNIPPETS = ["ls -la", "git status", "docker ps", "df -h"];

const storedSettings = persisted<Partial<Settings>>("sshx-settings-store", {});

/** A persisted store for settings of the current user. */
export const settings: Readable<Settings> = derived(
  storedSettings,
  ($storedSettings) => {
    // Do some validation on all of the stored settings.
    const name = $storedSettings.name ?? "";

    let theme = $storedSettings.theme;
    if (!theme || !Object.hasOwn(themes, theme)) {
      theme = defaultTheme;
    }

    let scrollback = $storedSettings.scrollback;
    if (typeof scrollback !== "number" || scrollback < 0) {
      scrollback = 5000;
    }

    const background =
      typeof $storedSettings.background === "string" &&
      $storedSettings.background
        ? $storedSettings.background
        : DEFAULT_BACKGROUND;

    let tileCols = $storedSettings.tileCols;
    if (typeof tileCols !== "number" || tileCols < 1 || tileCols > 8) {
      tileCols = 2;
    }

    const snippets = Array.isArray($storedSettings.snippets)
      ? $storedSettings.snippets.filter((s) => typeof s === "string")
      : DEFAULT_SNIPPETS;

    return {
      name,
      theme,
      scrollback,
      background,
      tileCols,
      snippets,
    };
  },
);

export function updateSettings(values: Partial<Settings>) {
  storedSettings.update((settings) => ({ ...settings, ...values }));
}

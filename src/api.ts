import { invoke } from "@tauri-apps/api/core";
import type { Macro, AppSettings } from "./types";

export const api = {
  getMacros: () => invoke<Macro[]>("get_macros"),

  addMacro: (title: string, content: string) =>
    invoke<Macro>("add_macro", { title, content }),

  updateMacro: (id: string, title: string, content: string) =>
    invoke<void>("update_macro", { id, title, content }),

  deleteMacro: (id: string) => invoke<void>("delete_macro", { id }),

  reorderMacros: (ids: string[]) => invoke<void>("reorder_macros", { ids }),

  exportMacros: () => invoke<void>("export_macros"),

  importMacros: () => invoke<Macro[]>("import_macros"),

  pasteText: (content: string) => invoke<void>("paste_text", { content }),

  getSettings: () => invoke<AppSettings>("get_settings"),

  updateHotkey: (hotkey: string) => invoke<void>("update_hotkey", { hotkey }),

  getAutostart: () => invoke<boolean>("get_autostart"),

  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),

  showSettings: () => invoke<void>("show_settings"),

  checkAccessibility: () => invoke<boolean>("check_accessibility"),

  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
};

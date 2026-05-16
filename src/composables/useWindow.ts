import { getCurrentWindow } from "@tauri-apps/api/window"

export function useWindow() {
  const appWindow = getCurrentWindow()

  return {
    minimize: () => appWindow.minimize(),
    toggleMaximize: () => appWindow.toggleMaximize(),
    close: () => appWindow.close(),
    isMaximized: async () => appWindow.isMaximized(),
  }
}

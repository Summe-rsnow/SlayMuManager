import { getCurrentWindow } from "@tauri-apps/api/window"

const appWindow = getCurrentWindow()

export function minimizeWindow() { appWindow.minimize() }
export function toggleMaximizeWindow() { appWindow.toggleMaximize() }
export function closeWindow() { appWindow.close() }
export function isWindowMaximized() { return appWindow.isMaximized() }

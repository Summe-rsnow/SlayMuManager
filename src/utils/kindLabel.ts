/** 存档类型标签翻译 */
export function kindLabel(t: (key: string) => string, kind: string): string {
  return kind === "vanilla" ? t("saves.kind.vanilla") : t("saves.kind.modded")
}

// ============================================================
// SlayMuManager 前端类型声明
// 与 Rust 端 domain/ 下的数据模型一一对应
// ============================================================

// --- Mod 相关 ---

export interface InstalledMod {
  id: string
  name: string
  version: string | null
  author: string | null
  folderName: string
  installDir: string
  manifestPath: string | null
  affectsGameplay: boolean
  state: ModState
}

export type ModState = "enabled" | "disabled" | "update_available" | "conflict" | "broken" | "unknown"

// --- 存档相关 ---

export interface SaveSlot {
  steamUserId: string
  kind: "vanilla" | "modded"
  slotIndex: number
  path: string
  hasData: boolean
  hasCurrentRun: boolean
  fileCount: number
  lastModifiedAt: string | null
}

export interface SaveBackupEntry {
  id: string
  steamUserId: string
  kind: "vanilla" | "modded"
  slotIndex: number
  backupPath: string
  createdAt: string
  reason: string
}

export interface SaveSyncPair {
  vanillaSlot: number
  moddedSlot: number
}

export interface SaveSyncResult {
  syncedCount: number
  details: Array<{
    slotIndex: number
    direction: "vanilla_to_modded" | "modded_to_vanilla"
    backupCreated: boolean
  }>
}

// --- Steam 云存档 ---

export interface CloudSaveStatus {
  isAvailable: boolean
  cloudPath: string | null
  localPath: string | null
  hasMismatch: boolean
  localOnlyCount: number
  cloudOnlyCount: number
  differentCount: number
  localFileCount: number
  cloudFileCount: number
  localAppliedToCloud: boolean
  cloudAppliedToLocal: boolean
  diagnostic: string
}

export interface CloudSaveDiffEntry {
  relativePath: string
  kind: "in_sync" | "different" | "local_only" | "cloud_only"
  localExists: boolean
  cloudExists: boolean
  localSize: number | null
  cloudSize: number | null
  localSha: string | null
  cloudSha: string | null
}

// --- 预设相关 ---

export interface ModProfile {
  id: string
  name: string
  description: string | null
  modIds: string[]
  createdAt: string
  updatedAt: string
  builtin: boolean
}

export interface ApplyProfileResult {
  profile: ModProfile
  enabledModIds: string[]
  disabledModIds: string[]
  missingModIds: string[]
}

// --- Nexus Mods 相关 ---

export interface RemoteMod {
  remoteId: string
  provider: string
  name: string
  summary: string | null
  author: string | null
  latestVersion: string | null
  pictureUrl: string | null
  thumbnailUrl: string | null
  thumbnailLargeUrl: string | null
  detailUrl: string
  endorsementCount: number
  downloadCount: number
  uniqueDownloads: number
}

export interface RemoteModSearchResult {
  items: RemoteMod[]
  totalCount: number
  offset: number
  count: number
}

// --- 批量导入 ---

export type DiscoveredModStatus = "ready" | "conflict" | "unsupported_format" | "error"
export type DiscoveredModSourceType = "folder" | "archive"

export interface DiscoveredMod {
  modId: string
  name: string
  version: string | null
  author: string | null
  folderName: string
  status: DiscoveredModStatus
  conflicts: string[]
  statusMessage: string | null
  sourceArchive: string | null
  sourceType: DiscoveredModSourceType
}

export interface BatchImportPreview {
  totalTargetsScanned: number
  discoveredMods: DiscoveredMod[]
}

export interface BatchInstallResult {
  successCount: number
  failureCount: number
  results: Array<{
    modId: string
    name: string
    success: boolean
    errorMessage: string | null
  }>
}

export type ConflictResolution = "skip" | "replace" | "rename"

// --- 整合包 ---

export interface BundleModInfo {
  modId: string
  name: string
  version: string | null
  folderName: string
}

export interface BundleProfileInfo {
  name: string
  description: string | null
  modIds: string[]
  createdAt: string
}

export interface BundleManifest {
  format: string
  profile: BundleProfileInfo
  mods: BundleModInfo[]
}

export interface BundleConflict {
  modId: string
  name: string
  reason: string
}

export interface BundlePreview {
  manifest: BundleManifest
  conflicts: BundleConflict[]
  missingIds: string[]
}

// --- Mod 切换保护 ---

export interface SaveGuardInfo {
  pathSwitched: boolean
  direction: string | null
  hadPairs: boolean
  savesSynced: number
  backupsCreated: number
  error: string | null
}

export interface ModToggleResult {
  modItem: InstalledMod
  saveGuard: SaveGuardInfo
}

// --- 通用 ---

export interface AppBootstrap {
  appName: string
  appVersion: string
  gameDirectory: string | null
  gameDirectoryValid: boolean
  installedCount: number
  disabledCount: number
  activeProfileName: string
  locale: string
  saveAutoSync: boolean
  saveSyncPairs: SaveSyncPair[]
  nexusApiKey: string | null
  nexusIsPremium: boolean
  nexusUserName: string | null
  proxyUrl: string | null
  autoBackupKeepCount: number
  themeMode: string
  themeColor: string
  launchMode: string
  launchCheckCloudSave: boolean
}

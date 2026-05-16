<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NTag, NIcon, NSpace, NModal, NPopconfirm, NSwitch,
  NSelect, NRadioGroup, NRadio, useMessage,
} from "naive-ui"
import {
  HardDrive, ArrowRightLeft, RefreshCw, Clock, Database,
  History, RotateCcw, Trash2, Upload, Download, Cloud, AlertTriangle,
  ShieldAlert,
} from "lucide-vue-next"
import type {
  SaveSlot, SaveBackupEntry, SaveSyncPair, SaveSyncResult,
  CloudSaveStatus, CloudSaveDiffEntry, AppBootstrap,
} from "../types"

const { t } = useI18n()
const message = useMessage()

// --- 状态 ---
const slots = ref<SaveSlot[]>([])
const showBackupsDialog = ref(false)
const backups = ref<SaveBackupEntry[]>([])
const autoSync = ref(false)
const syncPairs = ref<SaveSyncPair[]>([])
const loading = ref(false)
const steamUserId = ref("")

// 备份恢复到指定槽位
const showRestoreToSlotDialog = ref(false)
const restoreToSlotBackup = ref<SaveBackupEntry | null>(null)
const restoreToSlotTarget = ref<{ kind: string; slotIndex: number }>({ kind: "vanilla", slotIndex: 1 })

// 云存档
const cloudStatus = ref<CloudSaveStatus | null>(null)
const cloudDiffs = ref<CloudSaveDiffEntry[]>([])
const showCloudDialog = ref(false)

// --- 计算 ---
const vanillaSlots = computed(() =>
  slots.value.filter((s) => s.kind === "vanilla").sort((a, b) => a.slotIndex - b.slotIndex),
)
const moddedSlots = computed(() =>
  slots.value.filter((s) => s.kind === "modded").sort((a, b) => a.slotIndex - b.slotIndex),
)

const pairOptions = computed<any[]>(() => [
  { label: t("saves.pairSync.noPair"), value: null },
  { label: t("saves.pairSync.slotWithNumber", { n: 1 }), value: 1 },
  { label: t("saves.pairSync.slotWithNumber", { n: 2 }), value: 2 },
  { label: t("saves.pairSync.slotWithNumber", { n: 3 }), value: 3 },
])

function kindLabel(kind: string): string {
  return kind === "vanilla" ? t("saves.kind.vanilla") : t("saves.kind.modded")
}

// --- 加载 ---
async function loadSlots() {
  loading.value = true
  try {
    slots.value = await invoke<SaveSlot[]>("list_save_slots")
    if (slots.value.length > 0) {
      steamUserId.value = slots.value[0].steamUserId
    }
  } catch {
    slots.value = []
  } finally {
    loading.value = false
  }
}

// --- 删除存档 ---
async function deleteSaveSlot(slot: SaveSlot) {
  try {
    await invoke("delete_save_slot", {
      steamUserId: slot.steamUserId,
      kind: slot.kind,
      slotIndex: slot.slotIndex,
    })
    message.success(t("saves.success.slotDeleted", { i: slot.slotIndex }))
    await loadSlots()
  } catch (e: any) {
    message.error(t("saves.error.deleteSlotFailed") + ": " + e)
  }
}

// --- 全部历史备份（全局入口）---
async function openAllBackups() {
  loading.value = true
  try {
    // 不带过滤参数 → 加载全部槽位的全部备份
    backups.value = await invoke<SaveBackupEntry[]>("list_save_backups", {})
    showBackupsDialog.value = true
  } catch (e: any) {
    message.error(t("saves.error.loadBackupsFailed") + ": " + e)
  } finally {
    loading.value = false
  }
}

async function createBackup(slot: SaveSlot): Promise<SaveBackupEntry | null> {
  try {
    const entry = await invoke<SaveBackupEntry>("create_save_backup", {
      steamUserId: slot.steamUserId,
      kind: slot.kind,
      slotIndex: slot.slotIndex,
      reason: t("saves.backups.manualReason"),
    })
    message.success(t("saves.success.slotBackedUp", { i: slot.slotIndex }))
    return entry
  } catch (e: any) {
    message.error(t("saves.error.backupFailed") + ": " + e)
    return null
  }
}

// 迁移：先备份当前槽位，再用「恢复到...」对话框选择目标
async function migrateSlot(slot: SaveSlot) {
  const backup = await createBackup(slot)
  if (backup) {
    openRestoreToSlot(backup)
  }
}

// 恢复到指定槽位
function setRestoreTargetKind(v: string) {
  restoreToSlotTarget.value.kind = v
}
function setRestoreTargetSlot(v: number) {
  restoreToSlotTarget.value.slotIndex = v
}

function openRestoreToSlot(backup: SaveBackupEntry) {
  restoreToSlotBackup.value = backup
  restoreToSlotTarget.value = { kind: "vanilla", slotIndex: 1 }
  showRestoreToSlotDialog.value = true
}

async function doRestoreToSlot() {
  if (!restoreToSlotBackup.value) return
  const b = restoreToSlotBackup.value
  const target = restoreToSlotTarget.value
  try {
    await invoke("restore_save_backup_to_slot", {
      backupId: b.id,
      targetSteamUserId: steamUserId.value,
      targetKind: target.kind,
      targetSlotIndex: target.slotIndex,
    })
    message.success(t("saves.success.backupRestoredToSlot", { kind: kindLabel(target.kind), i: target.slotIndex }))
    showRestoreToSlotDialog.value = false
    showBackupsDialog.value = false
    await loadSlots()
  } catch (e: any) {
    message.error(t("saves.error.restoreFailed") + ": " + e)
  }
}

async function deleteBackup(backup: SaveBackupEntry) {
  try {
    await invoke("delete_save_backup", { backupId: backup.id })
    message.success(t("saves.success.backupDeleted"))
    backups.value = backups.value.filter((b) => b.id !== backup.id)
  } catch (e: any) {
    message.error(t("saves.error.backupDeleteFailed") + ": " + e)
  }
}

// --- 同步 ---
async function handleSync() {
  loading.value = true
  try {
    const result = await invoke<SaveSyncResult>("sync_saves")
    if (result.syncedCount > 0) {
      message.success(t("saves.success.syncPairsDone", { n: result.syncedCount }))
    } else {
      message.info(t("saves.info.noSyncNeeded"))
    }
    await loadSlots()
  } catch (e: any) {
    message.error(t("saves.error.syncFailed") + ": " + e)
  } finally {
    loading.value = false
  }
}

async function toggleAutoSync(val: boolean) {
  try {
    await invoke("toggle_save_auto_sync", { enabled: val })
    autoSync.value = val
  } catch (e: any) {
    message.error(t("saves.error.toggleFailed") + ": " + e)
  }
}

function getPairedModdedSlot(vanillaSlot: number): number | null {
  return syncPairs.value.find((p) => p.vanillaSlot === vanillaSlot)?.moddedSlot ?? null
}

function updatePair(vanillaSlot: number, moddedSlot: number | null) {
  if (moddedSlot !== null && moddedSlot >= 0) {
    syncPairs.value = syncPairs.value
      .filter((p) => p.vanillaSlot !== vanillaSlot)
      .concat({ vanillaSlot, moddedSlot })
  } else {
    syncPairs.value = syncPairs.value.filter((p) => p.vanillaSlot !== vanillaSlot)
  }
}

async function saveSyncPairs() {
  try {
    await invoke("update_save_sync_pairs", { pairs: syncPairs.value })
    message.success(t("saves.success.syncPairsSaved"))
  } catch (e: any) {
    message.error(t("saves.error.pairSaveFailed") + ": " + e)
  }
}

// --- 云存档 ---
async function openCloudDialog() {
  loading.value = true
  try {
    cloudStatus.value = await invoke<CloudSaveStatus>("get_cloud_save_status")
    cloudDiffs.value = await invoke<CloudSaveDiffEntry[]>("list_cloud_save_diff_entries")
    showCloudDialog.value = true
  } catch (e: any) {
    message.error(t("saves.error.cloudUnavailable") + ": " + e)
  } finally {
    loading.value = false
  }
}

async function copyCloudSide(relPath: string, side: string) {
  try {
    await invoke("copy_cloud_save_diff_side", { relativePath: relPath, side })
    message.success(t("saves.success.copied"))
    await openCloudDialog()
  } catch (e: any) {
    message.error(t("saves.error.copyFailed") + ": " + e)
  }
}

async function ascendFull() {
  loading.value = true
  try {
    await invoke("ascend_to_cloud_full")
    message.success(t("saves.success.ascended"))
    await openCloudDialog()
  } catch (e: any) {
    message.error(t("saves.error.ascendFailed") + ": " + e)
  } finally {
    loading.value = false
  }
}

async function descendFull() {
  loading.value = true
  try {
    await invoke("descend_from_cloud_full")
    message.success(t("saves.success.descended"))
    await openCloudDialog()
  } catch (e: any) {
    message.error(t("saves.error.descendFailed") + ": " + e)
  } finally {
    loading.value = false
  }
}

async function cleanupArtifacts() {
  try {
    await invoke("cleanup_backup_artifacts")
    message.success(t("saves.success.artifactsCleaned"))
  } catch (e: any) {
    message.error(t("saves.error.cleanupFailed") + ": " + e)
  }
}

function diffKindLabel(k: string): string {
  const m: Record<string, string> = {
    in_sync: t("saves.cloud.diffKind.inSync"),
    different: t("saves.cloud.diffKind.different"),
    local_only: t("saves.cloud.diffKind.localOnly"),
    cloud_only: t("saves.cloud.diffKind.cloudOnly"),
  }
  return m[k] ?? k
}

function diffKindType(k: string): "success" | "warning" | "error" | "info" {
  const m: Record<string, "success" | "warning" | "error" | "info"> = {
    in_sync: "success", different: "warning", local_only: "info", cloud_only: "info",
  }
  return m[k] ?? "default"
}

function mismatchLabel(s: CloudSaveStatus): string {
  if (!s.hasMismatch) return t("saves.cloud.synced")
  const parts: string[] = []
  if (s.differentCount > 0) parts.push(t("saves.cloud.mismatch.different", { n: s.differentCount }))
  if (s.localOnlyCount > 0) parts.push(t("saves.cloud.mismatch.localOnly", { n: s.localOnlyCount }))
  if (s.cloudOnlyCount > 0) parts.push(t("saves.cloud.mismatch.cloudOnly", { n: s.cloudOnlyCount }))
  return parts.join(" + ") || t("saves.cloud.mismatch.other")
}

onMounted(async () => {
  await loadSlots()
  // 恢复上次保存的配对同步配置
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    syncPairs.value = bootstrap.saveSyncPairs ?? []
    autoSync.value = bootstrap.saveAutoSync ?? false
  } catch { /* ignore */ }
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-800">{{ t("saves.title") }}</h1>
        <p class="text-sm text-gray-500 mt-1">{{ t("saves.subtitle") }}</p>
      </div>
      <NSpace>
        <NButton secondary :loading="loading" @click="loadSlots">
          <template #icon><NIcon :size="16"><RefreshCw /></NIcon></template>
          {{ t("common.refresh") }}
        </NButton>
        <NButton secondary @click="handleSync">
          <template #icon><NIcon :size="16"><ArrowRightLeft /></NIcon></template>
          {{ t("saves.sync") }}
        </NButton>
        <NButton secondary @click="openAllBackups">
          <template #icon><NIcon :size="16"><History /></NIcon></template>
          {{ t("saves.allHistoryBackups") }}
        </NButton>
      </NSpace>
    </div>

    <!-- 空状态 -->
    <NCard v-if="slots.length === 0 && !loading" size="small">
      <div class="text-center py-12 text-gray-400">
        <NIcon :size="48" class="c-gray-300 mb-3"><HardDrive /></NIcon>
        <p>{{ t("saves.empty.setGamePath") }}</p>
      </div>
    </NCard>

    <template v-else>
      <!-- 双列布局 -->
      <div class="grid grid-cols-2 gap-4">
        <!-- 原版存档 -->
        <NCard size="small">
          <template #header>
            <div class="flex items-center gap-2">
              <NTag type="info" size="small" :bordered="false">{{ t("saves.kind.vanilla") }}</NTag>
              <span class="text-xs text-gray-400">
                {{ t("saves.slotCount", { n: vanillaSlots.length }) }}
              </span>
            </div>
          </template>

          <NSpace v-if="vanillaSlots.length > 0" vertical :size="8">
            <div
              v-for="slot in vanillaSlots"
              :key="`v-${slot.slotIndex}`"
              :class="slot.hasData
                ? 'p-3 rounded-lg border border-gray-100 bg-white'
                : 'p-2 rounded-lg border border-dashed border-gray-200 bg-gray-50/70'"
            >
              <div :class="slot.hasData ? 'flex items-center justify-between mb-2' : 'flex items-center justify-between'">
                <span class="font-medium text-sm text-gray-700">
                  {{ t("saves.slotIndex", { i: slot.slotIndex }) }}
                </span>
                <NSpace :size="4">
                  <NTag v-if="slot.hasCurrentRun" type="warning" size="tiny" :bordered="false">
                    {{ t("saves.currentRun") }}
                  </NTag>
                  <NTag
                    :type="slot.hasData ? 'success' : 'default'"
                    size="tiny"
                    :bordered="false"
                  >
                    {{ slot.hasData ? t("saves.fileCount", { n: slot.fileCount }) : t("saves.empty.empty") }}
                  </NTag>
                </NSpace>
              </div>

              <div v-if="slot.hasData" class="text-xs text-gray-400 mb-2">
                <NIcon :size="12"><Clock /></NIcon>
                {{ slot.lastModifiedAt ? new Date(slot.lastModifiedAt).toLocaleString("zh-CN") : t("common.unknown") }}
              </div>

              <!-- 操作按钮行 -->
              <div v-if="slot.hasData" class="flex items-center gap-1 flex-wrap">
                <NButton size="tiny" secondary @click="createBackup(slot)">
                  {{ t("saves.backup") }}
                </NButton>
                <NButton size="tiny" secondary @click="migrateSlot(slot)">
                  <template #icon><NIcon :size="12"><Upload /></NIcon></template>
                  {{ t("saves.migrate") }}
                </NButton>
                <NPopconfirm
                  @positive-click="() => deleteSaveSlot(slot)"
                >
                  <template #trigger>
                    <NButton size="tiny" type="error" text>
                      <template #icon><NIcon :size="12"><Trash2 /></NIcon></template>
                      {{ t("saves.deleteSlot") }}
                    </NButton>
                  </template>
                  <div class="max-w-64">
                    <div class="flex items-center gap-2 mb-1">
                      <NIcon :size="16" color="#d03050"><ShieldAlert /></NIcon>
                      <span class="font-medium">{{ t("saves.confirmDeleteSlotTitle") }}</span>
                    </div>
                    <p class="text-xs text-gray-500">
                      {{ t("saves.confirmDeleteSlotDesc", { kind: kindLabel(slot.kind), i: slot.slotIndex }) }}
                    </p>
                    <p class="text-xs text-amber-600 mt-1">
                      {{ t("saves.confirmDeleteSlotNote") }}
                    </p>
                  </div>
                </NPopconfirm>
              </div>
            </div>
          </NSpace>
        </NCard>

        <!-- 模组版存档 -->
        <NCard size="small">
          <template #header>
            <div class="flex items-center gap-2">
              <NTag type="warning" size="small" :bordered="false">{{ t("saves.kind.modded") }}</NTag>
              <span class="text-xs text-gray-400">
                {{ t("saves.slotCount", { n: moddedSlots.length }) }}
              </span>
            </div>
          </template>

          <NSpace v-if="moddedSlots.length > 0" vertical :size="8">
            <div
              v-for="slot in moddedSlots"
              :key="`m-${slot.slotIndex}`"
              :class="slot.hasData
                ? 'p-3 rounded-lg border border-gray-100 bg-white'
                : 'p-2 rounded-lg border border-dashed border-gray-200 bg-gray-50/70'"
            >
              <div :class="slot.hasData ? 'flex items-center justify-between mb-2' : 'flex items-center justify-between'">
                <span class="font-medium text-sm text-gray-700">
                  {{ t("saves.slotIndex", { i: slot.slotIndex }) }}
                </span>
                <NSpace :size="4">
                  <NTag v-if="slot.hasCurrentRun" type="warning" size="tiny" :bordered="false">
                    {{ t("saves.currentRun") }}
                  </NTag>
                  <NTag
                    :type="slot.hasData ? 'success' : 'default'"
                    size="tiny"
                    :bordered="false"
                  >
                    {{ slot.hasData ? t("saves.fileCount", { n: slot.fileCount }) : t("saves.empty.empty") }}
                  </NTag>
                </NSpace>
              </div>

              <div v-if="slot.hasData" class="text-xs text-gray-400 mb-2">
                <NIcon :size="12"><Clock /></NIcon>
                {{ slot.lastModifiedAt ? new Date(slot.lastModifiedAt).toLocaleString("zh-CN") : t("common.unknown") }}
              </div>

              <div v-if="slot.hasData" class="flex items-center gap-1 flex-wrap">
                <NButton size="tiny" secondary @click="createBackup(slot)">
                  {{ t("saves.backup") }}
                </NButton>
                <NButton size="tiny" secondary @click="migrateSlot(slot)">
                  <template #icon><NIcon :size="12"><Upload /></NIcon></template>
                  {{ t("saves.migrate") }}
                </NButton>
                <NPopconfirm
                  @positive-click="() => deleteSaveSlot(slot)"
                >
                  <template #trigger>
                    <NButton size="tiny" type="error" text>
                      <template #icon><NIcon :size="12"><Trash2 /></NIcon></template>
                      {{ t("saves.deleteSlot") }}
                    </NButton>
                  </template>
                  <div class="max-w-64">
                    <div class="flex items-center gap-2 mb-1">
                      <NIcon :size="16" color="#d03050"><ShieldAlert /></NIcon>
                      <span class="font-medium">{{ t("saves.confirmDeleteSlotTitle") }}</span>
                    </div>
                    <p class="text-xs text-gray-500">
                      {{ t("saves.confirmDeleteSlotDesc", { kind: kindLabel(slot.kind), i: slot.slotIndex }) }}
                    </p>
                    <p class="text-xs text-amber-600 mt-1">
                      {{ t("saves.confirmDeleteSlotNote") }}
                    </p>
                  </div>
                </NPopconfirm>
              </div>
            </div>
          </NSpace>
        </NCard>
      </div>

      <!-- 配对同步配置 -->
      <NCard class="mt-4">
        <template #header>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <NIcon :size="20" color="#6366f1"><ArrowRightLeft /></NIcon>
              <span class="text-lg font-semibold">{{ t("saves.pairSync.title") }}</span>
              <NTag :type="autoSync ? 'success' : 'default'" size="small" :bordered="false">
                {{ autoSync ? t("saves.pairSync.autoSyncing") : t("saves.pairSync.manual") }}
              </NTag>
            </div>
            <NSwitch :value="autoSync" @update:value="toggleAutoSync">
              <template #checked>{{ t("saves.pairSync.on") }}</template>
              <template #unchecked>{{ t("saves.pairSync.off") }}</template>
            </NSwitch>
          </div>
        </template>

        <p class="text-sm text-gray-500 mb-4">
          {{ t("saves.pairSync.description") }}
        </p>

        <div class="grid grid-cols-1 gap-3 mb-4">
          <div
            v-for="i in 3"
            :key="`pair-${i}`"
            class="pair-card rounded-xl p-4 border-2 transition-all"
            :class="getPairedModdedSlot(i) !== null
              ? 'border-indigo-200 bg-indigo-50/30'
              : 'border-gray-100 bg-gray-50/50'"
          >
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-3 flex-1">
                <div class="w-10 h-10 rounded-xl bg-blue-100 flex items-center justify-center flex-shrink-0">
                  <span class="text-sm font-bold text-blue-600">V{{ i }}</span>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-gray-400">{{ t("saves.pairSync.vanillaSlot") }}</span>
                  <span class="text-sm font-medium text-gray-700">{{ t("saves.pairSync.slotWithNumber", { n: i }) }}</span>
                </div>
              </div>

              <div class="flex flex-col items-center flex-shrink-0">
                <NIcon :size="22" :color="getPairedModdedSlot(i) !== null ? '#6366f1' : '#d1d5db'">
                  <ArrowRightLeft />
                </NIcon>
                <span
                  v-if="getPairedModdedSlot(i) !== null"
                  class="text-[10px] text-indigo-400 mt-0.5"
                >{{ t("saves.pairSync.paired") }}</span>
                <span v-else class="text-[10px] text-gray-300 mt-0.5">{{ t("saves.pairSync.unpaired") }}</span>
              </div>

              <div class="flex items-center gap-3 flex-1 justify-end">
                <div
                  class="w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0"
                  :class="getPairedModdedSlot(i) !== null ? 'bg-purple-100' : 'bg-gray-100'"
                >
                  <span
                    class="text-sm font-bold"
                    :class="getPairedModdedSlot(i) !== null ? 'text-purple-600' : 'text-gray-400'"
                  >{{ getPairedModdedSlot(i) !== null ? `M${getPairedModdedSlot(i)}` : '?' }}</span>
                </div>
                <div class="flex flex-col items-end">
                  <span class="text-xs text-gray-400">{{ t("saves.pairSync.moddedSlot") }}</span>
                  <NSelect
                    :value="getPairedModdedSlot(i)"
                    :options="pairOptions"
                    size="small"
                    style="width: 110px"
                    :placeholder="t('saves.pairSync.chooseSlot')"
                    @update:value="(v: number | null) => updatePair(i, v)"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex items-center justify-between pt-3 border-t border-gray-100">
          <span class="text-xs text-gray-400">
            {{ t("saves.pairSync.pairCount", { n: syncPairs.length }) }}
          </span>
          <NButton type="primary" size="small" @click="saveSyncPairs">
            <template #icon><NIcon :size="14"><ArrowRightLeft /></NIcon></template>
            {{ t("saves.pairSync.savePairs") }}
          </NButton>
        </div>
      </NCard>

      <!-- Steam 云存档 -->
      <NCard size="small" class="mt-4">
        <template #header>
          <div class="flex items-center gap-2">
            <NIcon :size="16" color="#6366f1"><Cloud /></NIcon>
            <span>{{ t("saves.cloud.title") }}</span>
          </div>
        </template>
        <p class="text-xs text-gray-400 mb-3">
          {{ t("saves.cloud.description") }}
        </p>
        <NSpace>
          <NButton secondary size="small" :loading="loading" @click="openCloudDialog">
            <template #icon><NIcon :size="14"><Cloud /></NIcon></template>
            {{ t("saves.cloud.viewStatus") }}
          </NButton>
          <NButton secondary size="small" @click="ascendFull">
            <template #icon><NIcon :size="14"><Upload /></NIcon></template>
            {{ t("saves.cloud.uploadAll") }}
          </NButton>
          <NButton secondary size="small" @click="descendFull">
            <template #icon><NIcon :size="14"><Download /></NIcon></template>
            {{ t("saves.cloud.downloadAll") }}
          </NButton>
        </NSpace>
      </NCard>
    </template>

    <!-- 备份列表对话框 -->
    <NModal
      :show="showBackupsDialog"
      @update:show="(v: boolean) => !v && (showBackupsDialog = false)"
    >
      <NCard style="width: 560px; max-height: 75vh" :bordered="false" role="dialog">
        <template #header>
          <span class="text-lg font-semibold">{{ t("saves.allHistoryBackups") }}</span>
        </template>

        <div v-if="backups.length === 0" class="text-center py-8 text-gray-400">
          <NIcon :size="32" class="c-gray-300 mb-2"><Database /></NIcon>
          <p class="text-sm">{{ t("saves.backups.empty") }}</p>
        </div>

        <div v-else class="max-h-96 overflow-auto">
          <div
            v-for="b in backups"
            :key="b.id"
            class="flex items-center justify-between p-3 border-b border-gray-50 last:border-b-0"
          >
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-700">
                {{ new Date(b.createdAt).toLocaleString("zh-CN") }}
              </div>
              <div class="text-xs text-gray-400">
                {{ b.reason }} · {{ kindLabel(b.kind) }} {{ t("saves.slotIndex", { i: b.slotIndex }) }}
              </div>
            </div>
            <NSpace :size="4">
              <!-- 恢复到指定槽位 -->
              <NButton size="tiny" secondary @click="openRestoreToSlot(b)">
                <template #icon><NIcon :size="12"><RotateCcw /></NIcon></template>
                {{ t("saves.backups.restoreTo") }}
              </NButton>
              <NPopconfirm @positive-click="() => deleteBackup(b)">
                <template #trigger>
                  <NButton size="tiny" type="error" text>
                    <template #icon><NIcon :size="12"><Trash2 /></NIcon></template>
                  </NButton>
                </template>
                {{ t("saves.backups.confirmDelete") }}
              </NPopconfirm>
            </NSpace>
          </div>
        </div>
      </NCard>
    </NModal>

    <!-- 恢复到指定槽位对话框 -->
    <NModal
      :show="showRestoreToSlotDialog"
      @update:show="(v: boolean) => !v && (showRestoreToSlotDialog = false)"
    >
      <NCard v-if="restoreToSlotBackup" style="width: 400px" :bordered="false" role="dialog">
        <template #header>
          <div class="flex items-center gap-2">
            <NIcon :size="18" color="#6366f1"><RotateCcw /></NIcon>
            <span class="text-lg font-semibold">{{ t("saves.backups.restoreToTitle") }}</span>
          </div>
        </template>

        <div class="text-sm text-gray-600 mb-4">
          <div class="mb-2">
            <span class="text-gray-400">{{ t("saves.backups.restoreFrom") }}:</span>
            <span class="ml-2 font-medium">
              {{ new Date(restoreToSlotBackup.createdAt).toLocaleString("zh-CN") }}
            </span>
          </div>
          <div class="text-xs text-gray-400 mb-3">
            {{ restoreToSlotBackup.reason }} · {{ kindLabel(restoreToSlotBackup.kind) }}
            {{ t("saves.slotIndex", { i: restoreToSlotBackup.slotIndex }) }}
          </div>

          <div class="p-3 rounded-lg bg-gray-50 border border-gray-100">
            <div class="text-xs text-gray-500 mb-2">{{ t("saves.backups.restoreTarget") }}</div>
            <NSpace vertical :size="8">
              <NRadioGroup
                :value="restoreToSlotTarget.kind"
                @update:value="(v: string) => setRestoreTargetKind(v)"
              >
                <NSpace>
                  <NRadio value="vanilla">{{ t("saves.kind.vanilla") }}</NRadio>
                  <NRadio value="modded">{{ t("saves.kind.modded") }}</NRadio>
                </NSpace>
              </NRadioGroup>
              <NSelect
                :value="restoreToSlotTarget.slotIndex"
                :options="[1, 2, 3].map(i => ({ label: t('saves.slotIndex', { i }), value: i }))"
                size="small"
                style="width: 120px"
                :placeholder="t('saves.backups.chooseSlot')"
                @update:value="(v: number) => setRestoreTargetSlot(v)"
              />
            </NSpace>
          </div>

          <div class="mt-3 p-2 rounded bg-amber-50 text-xs text-amber-700 flex items-start gap-1.5">
            <NIcon :size="14"><AlertTriangle /></NIcon>
            <span>{{ t("saves.backups.restoreConfirmHint") }}</span>
          </div>
        </div>

        <template #footer>
          <NSpace justify="end">
            <NButton @click="showRestoreToSlotDialog = false">{{ t("common.cancel") }}</NButton>
            <NButton type="primary" @click="doRestoreToSlot">
              {{ t("saves.backups.confirmRestore") }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </NModal>

    <!-- Steam 云存档对话框 -->
    <NModal
      :show="showCloudDialog"
      @update:show="(v: boolean) => !v && (showCloudDialog = false)"
    >
      <NCard style="width: 640px; max-height: 80vh" :bordered="false" role="dialog">
        <template #header>
          <div class="flex items-center justify-between">
            <span class="text-lg font-semibold">{{ t("saves.cloud.title") }}</span>
            <NButton size="tiny" @click="cleanupArtifacts">{{ t("saves.cloud.cleanupArtifacts") }}</NButton>
          </div>
        </template>

        <div v-if="cloudStatus && cloudStatus.isAvailable" class="grid grid-cols-4 gap-2 mb-4">
          <div class="text-center p-2 rounded bg-gray-50">
            <div class="text-lg font-bold text-gray-700">{{ cloudStatus.localFileCount }}</div>
            <div class="text-xs text-gray-400">{{ t("saves.cloud.localFiles") }}</div>
          </div>
          <div class="text-center p-2 rounded bg-gray-50">
            <div class="text-lg font-bold text-gray-700">{{ cloudStatus.cloudFileCount }}</div>
            <div class="text-xs text-gray-400">{{ t("saves.cloud.cloudFiles") }}</div>
          </div>
          <div class="text-center p-2 rounded bg-gray-50">
            <div class="text-lg font-bold text-amber-600">{{ cloudStatus.differentCount }}</div>
            <div class="text-xs text-gray-400">{{ t("saves.cloud.differences") }}</div>
          </div>
          <div
            class="text-center p-2 rounded"
            :class="cloudStatus.hasMismatch ? 'bg-amber-50' : 'bg-green-50'"
          >
            <NIcon :size="24" :color="cloudStatus.hasMismatch ? '#f0a020' : '#18a058'">
              <AlertTriangle v-if="cloudStatus.hasMismatch" />
              <Cloud v-else />
            </NIcon>
            <div class="text-xs mt-1" :class="cloudStatus.hasMismatch ? 'text-amber-600' : 'text-green-600'">
              {{ mismatchLabel(cloudStatus) }}
            </div>
          </div>
        </div>

        <div
          v-if="cloudStatus && !cloudStatus.isAvailable"
          class="p-4 rounded-lg bg-amber-50 border border-amber-200 mb-4"
        >
          <div class="flex items-start gap-2">
            <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
            <div>
              <div class="text-sm font-medium text-amber-800 mb-1">{{ t("saves.cloud.unavailable") }}</div>
              <div class="text-xs text-amber-600">{{ cloudStatus.diagnostic }}</div>
              <div v-if="cloudStatus.cloudPath" class="text-xs text-gray-400 mt-1 font-mono">
                {{ t("saves.cloud.cloudLabel") }}: {{ cloudStatus.cloudPath }}
              </div>
              <div v-if="cloudStatus.localPath" class="text-xs text-gray-400 mt-0.5 font-mono">
                {{ t("saves.cloud.localLabel") }}: {{ cloudStatus.localPath }}
              </div>
            </div>
          </div>
        </div>

        <NSpace class="mb-4">
          <NButton size="small" secondary :disabled="!cloudStatus?.isAvailable" @click="ascendFull">{{ t("saves.cloud.ascend") }}</NButton>
          <NButton size="small" secondary :disabled="!cloudStatus?.isAvailable" @click="descendFull">{{ t("saves.cloud.descend") }}</NButton>
        </NSpace>

        <div v-if="cloudDiffs.length > 0" class="max-h-64 overflow-auto border border-gray-100 rounded-lg">
          <div
            v-for="d in cloudDiffs"
            :key="d.relativePath"
            class="flex items-center justify-between p-2 border-b border-gray-50 last:border-b-0 text-sm"
          >
            <div class="flex-1 min-w-0 flex items-center gap-2">
              <NTag :type="diffKindType(d.kind)" size="tiny" :bordered="false">
                {{ diffKindLabel(d.kind) }}
              </NTag>
              <span class="text-gray-700 truncate font-mono text-xs">{{ d.relativePath }}</span>
            </div>
            <NSpace :size="4" class="flex-shrink-0 ml-3">
              <NButton
                v-if="d.kind === 'local_only'"
                size="tiny"
                secondary
                @click="copyCloudSide(d.relativePath, 'local_to_cloud')"
              >
                {{ t("saves.cloud.toCloud") }}
              </NButton>
              <NButton
                v-if="d.kind === 'cloud_only'"
                size="tiny"
                secondary
                @click="copyCloudSide(d.relativePath, 'cloud_to_local')"
              >
                {{ t("saves.cloud.toLocal") }}
              </NButton>
            </NSpace>
          </div>
        </div>
        <div v-else class="text-center py-6 text-gray-400 text-sm">
          {{ t("saves.cloud.inSync") }}
        </div>
      </NCard>
    </NModal>
  </div>
</template>

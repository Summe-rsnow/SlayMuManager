<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { currentLocale } from "@/i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NTag, NIcon, NSpace, NPopconfirm, NSwitch,
  NSelect, NRadioGroup, NRadio, useMessage,
} from "naive-ui"
import SlotCard from "@/components/SlotCard.vue"
import CountBadge from "@/components/CountBadge.vue"
import {
  HardDrive, ArrowRightLeft, RefreshCw, Database,
  History, RotateCcw, Trash2, Upload, Download, Cloud, AlertTriangle, User,
} from "@lucide/vue"
import type {
  SaveSlot, SaveBackupEntry, SaveSyncPair, SaveSyncResult,
  CloudSaveStatus, CloudSaveDiffEntry, AppBootstrap,
} from "../types"
import { useStorage } from "@/composables/useStorage"
import { useIsActive } from "@/composables/useIsActive"
import { kindLabel } from "@/utils/kindLabel"
import EmptyState from "@/components/EmptyState.vue"
import AppDialog from "@/components/AppDialog.vue"
import FloatingTip from "@/components/FloatingTip.vue"
import PageHeader from "@/components/PageHeader.vue"

const { t } = useI18n()
const message = useMessage()
const { isActive } = useIsActive()

// --- 状态 ---
const slots = ref<SaveSlot[]>([])
const showBackupsDialog = ref(false)
const backups = ref<SaveBackupEntry[]>([])
const autoSync = ref(false)
const syncPairs = ref<SaveSyncPair[]>([])
const loading = ref(false)
const activeUserId = useStorage<string>("slaymgr:active-steam-user", "")

// 备份恢复到指定槽位
const showRestoreToSlotDialog = ref(false)
const restoreToSlotBackup = ref<SaveBackupEntry | null>(null)
const restoreToSlotTarget = ref<{ kind: string; slotIndex: number }>({ kind: "vanilla", slotIndex: 1 })

// 云存档
const cloudStatus = ref<CloudSaveStatus | null>(null)
const cloudDiffs = ref<CloudSaveDiffEntry[]>([])
const showCloudDialog = ref(false)

// 备份计数
const slotBackupCounts = ref<Record<string, { manual: number; auto: number; keepCount: number }>>({})
const autoBackupKeepCount = ref(5)

/** 所有槽位的备份汇总 */
const totalBackupStats = computed(() => {
  let manual = 0, auto = 0
  for (const c of Object.values(slotBackupCounts.value)) {
    manual += c.manual
    auto += c.auto
  }
  return { manual, auto, limit: autoBackupKeepCount.value }
})

async function loadBackupCounts() {
  try {
    const [allBackups, bootstrap] = await Promise.all([
      invoke<SaveBackupEntry[]>("list_save_backups", {}),
      invoke<AppBootstrap>("get_app_bootstrap"),
    ])
    if (!isActive.value) return
    autoBackupKeepCount.value = bootstrap.autoBackupKeepCount ?? 5
    const counts: Record<string, { manual: number; auto: number; keepCount: number }> = {}
    for (const b of allBackups) {
      const key = `${b.steamUserId}:${b.kind}:${b.slotIndex}`
      if (!counts[key]) counts[key] = { manual: 0, auto: 0, keepCount: autoBackupKeepCount.value }
      if (b.manual) counts[key].manual++
      else counts[key].auto++
    }
    slotBackupCounts.value = counts
  } catch {
    // 静默失败，不影响主流程
  }
}

// --- 根据 steamUserId 分组 ---
const userGroups = computed(() => {
  const groups = new Map<string, SaveSlot[]>()
  for (const slot of slots.value) {
    const list = groups.get(slot.steamUserId) || []
    list.push(slot)
    groups.set(slot.steamUserId, list)
  }
  return groups
})

const userIds = computed(() => Array.from(userGroups.value.keys()))

// 确保 activeUserId 有效
const safeActiveUserId = computed(() => {
  if (activeUserId.value && userIds.value.includes(activeUserId.value)) {
    return activeUserId.value
  }
  return userIds.value[0] || ""
})

const currentUserSlots = computed(() => userGroups.value.get(safeActiveUserId.value) || [])

const vanillaSlots = computed(() =>
  currentUserSlots.value.filter((s) => s.kind === "vanilla").sort((a, b) => a.slotIndex - b.slotIndex),
)
const moddedSlots = computed(() =>
  currentUserSlots.value.filter((s) => s.kind === "modded").sort((a, b) => a.slotIndex - b.slotIndex),
)

const pairOptions = computed(() => {
  const slots = [1, 2, 3]
  return slots.map((m) => ({ label: t("saves.pairSync.slotWithNumber", { n: m }), value: m }))
})

function pairOptionsForSlot(vanillaSlot: number): any[] {
  // 已被其他原版槽位占用的模组版槽位
  const taken = syncPairs.value
    .filter((p) => p.vanillaSlot !== vanillaSlot)
    .map((p) => p.moddedSlot)
  return [
    { label: t("saves.pairSync.noPair"), value: null as number | null },
    ...pairOptions.value.filter((o) => !taken.includes(o.value)),
  ]
}

// 截断用户 ID 显示
function shortUserId(id: string): string {
  if (id.length <= 8) return id
  return id.slice(0, 4) + "..." + id.slice(-4)
}

// --- 加载 ---
async function loadSlots() {
  loading.value = true
  try {
    const data = await invoke<SaveSlot[]>("list_save_slots")
    if (!isActive.value) return
    slots.value = data
    if (slots.value.length > 0 && !activeUserId.value) {
      activeUserId.value = slots.value[0].steamUserId
    }
    await loadBackupCounts()
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
    if (!isActive.value) return
    message.success(t("saves.success.slotDeleted", { i: slot.slotIndex }))
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.deleteSlotFailed") + ": " + String(e))
  }
}

// --- 历史备份（全局入口）---
async function openAllBackups() {
  loading.value = true
  try {
    const data = await invoke<SaveBackupEntry[]>("list_save_backups", {})
    if (!isActive.value) return
    backups.value = data
    showBackupsDialog.value = true
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.loadBackupsFailed") + ": " + String(e))
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
    if (!isActive.value) return null
    message.success(t("saves.success.slotBackedUp", { i: slot.slotIndex }))
    await loadBackupCounts()
    return entry
  } catch (e: unknown) {
    if (!isActive.value) return null
    message.error(t("saves.error.backupFailed") + ": " + String(e))
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
      targetSteamUserId: b.steamUserId,
      targetKind: target.kind,
      targetSlotIndex: target.slotIndex,
    })
    if (!isActive.value) return
    message.success(t("saves.success.backupRestoredToSlot", { kind: kindLabel(t,target.kind), i: target.slotIndex }))
    showRestoreToSlotDialog.value = false
    showBackupsDialog.value = false
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.restoreFailed") + ": " + String(e))
  }
}

async function deleteBackup(backup: SaveBackupEntry) {
  try {
    await invoke("delete_save_backup", { backupId: backup.id })
    if (!isActive.value) return
    message.success(t("saves.success.backupDeleted"))
    backups.value = backups.value.filter((b) => b.id !== backup.id)
    await loadBackupCounts()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.backupDeleteFailed") + ": " + String(e))
  }
}

// --- 旧版备份升级 ---
const showUpgradeDialog = ref(false)
const upgradingBackupId = ref<string | null>(null)
const upgradeDialogManual = ref(true)

function openUpgradeDialog(backupId: string) {
  upgradingBackupId.value = backupId
  upgradeDialogManual.value = true
  showUpgradeDialog.value = true
}

async function confirmUpgrade() {
  const bid = upgradingBackupId.value
  if (!bid) return
  try {
    await invoke("upgrade_backup_manual_flag", {
      backupId: bid,
      manual: upgradeDialogManual.value,
    })
    if (!isActive.value) return
    message.success(t("saves.success.backupUpgraded"))
    // 更新本地缓存
    const b = backups.value.find((x) => x.id === bid)
    if (b) b.manual = upgradeDialogManual.value
    await loadBackupCounts()
    showUpgradeDialog.value = false
    upgradingBackupId.value = null
  } catch (e: unknown) {
    message.error(t("saves.error.backupUpgradeFailed") + ": " + String(e))
  }
}

// --- 同步 ---
async function handleSync() {
  loading.value = true
  try {
    const result = await invoke<SaveSyncResult>("sync_saves")
    if (!isActive.value) return
    if (result.syncedCount > 0) {
      message.success(t("saves.success.syncPairsDone", { n: result.syncedCount }))
    } else {
      message.info(t("saves.info.noSyncNeeded"))
    }
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.syncFailed") + ": " + String(e))
  } finally {
    loading.value = false
  }
}

async function toggleAutoSync(val: boolean) {
  try {
    await invoke("toggle_save_auto_sync", { enabled: val })
    if (!isActive.value) return
    autoSync.value = val
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.toggleFailed") + ": " + String(e))
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
    if (!isActive.value) return
    message.success(t("saves.success.syncPairsSaved"))
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.pairSaveFailed") + ": " + String(e))
  }
}

// --- 云存档 ---
async function openCloudDialog() {
  loading.value = true
  try {
    cloudStatus.value = await invoke<CloudSaveStatus>("get_cloud_save_status")
    if (!isActive.value) return
    cloudDiffs.value = await invoke<CloudSaveDiffEntry[]>("list_cloud_save_diff_entries")
    if (!isActive.value) return
    showCloudDialog.value = true
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.cloudUnavailable") + ": " + String(e))
  } finally {
    loading.value = false
  }
}

async function copyCloudSide(relPath: string, side: string) {
  try {
    await invoke("copy_cloud_save_diff_side", { relativePath: relPath, side })
    if (!isActive.value) return
    message.success(t("saves.success.copied"))
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.copyFailed") + ": " + String(e))
  }
}

async function ascendFull() {
  loading.value = true
  try {
    await invoke("ascend_to_cloud_full")
    if (!isActive.value) return
    message.success(t("saves.success.ascended"))
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.ascendFailed") + ": " + String(e))
  } finally {
    loading.value = false
  }
}

async function descendFull() {
  loading.value = true
  try {
    await invoke("descend_from_cloud_full")
    if (!isActive.value) return
    message.success(t("saves.success.descended"))
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.descendFailed") + ": " + String(e))
  } finally {
    loading.value = false
  }
}

async function cleanupArtifacts() {
  try {
    await invoke("cleanup_backup_artifacts")
    if (!isActive.value) return
    message.success(t("saves.success.artifactsCleaned"))
  } catch (e: unknown) {
    if (!isActive.value) return
    message.error(t("saves.error.cleanupFailed") + ": " + String(e))
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

function diffKindType(k: string): "success" | "warning" | "error" | "info" | "default" {
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
  if (!isActive.value) return
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!isActive.value) return
    syncPairs.value = bootstrap.saveSyncPairs ?? []
    autoSync.value = bootstrap.saveAutoSync ?? false
  } catch { /* ignore */ }
  await loadBackupCounts()
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <PageHeader :title="t('saves.title')" :subtitle="t('saves.subtitle')">
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
    </PageHeader>

    <!-- 空状态 -->
    <EmptyState v-if="slots.length === 0 && !loading" :icon="HardDrive" :title="t('saves.empty.setGamePath')" bordered />

    <template v-else>
      <!-- 多用户切换 -->
      <div v-if="userIds.length > 1" class="flex items-center gap-3 mb-4">
        <NIcon :size="16" class="text-c-muted"><User /></NIcon>
        <span class="text-xs text-c-muted">{{ t("saves.steamUser") }}:</span>
        <div class="flex gap-1.5">
          <NButton
            v-for="uid in userIds"
            :key="uid"
            :size="'tiny'"
            :type="uid === safeActiveUserId ? 'primary' : 'default'"
            :secondary="uid !== safeActiveUserId"
            @click="activeUserId = uid"
          >
            <template #icon><NIcon :size="12"><User /></NIcon></template>
            {{ shortUserId(uid) }}
          </NButton>
        </div>
      </div>

      <!-- 双列布局 -->
      <div class="grid grid-cols-2 gap-6">
        <!-- 原版存档 -->
        <NCard size="small" class="save-card-glass">
          <template #header>
            <div class="flex items-center gap-2">
              <NTag type="info" size="small" :bordered="false">{{ t("saves.kind.vanilla") }}</NTag>
              <span class="text-xs text-c-muted">
                {{ t("saves.slotCount", { n: vanillaSlots.length }) }}
              </span>
            </div>
          </template>

          <NSpace v-if="vanillaSlots.length > 0" vertical :size="8">
            <SlotCard
              v-for="slot in vanillaSlots"
              :key="`v-${slot.steamUserId}-${slot.slotIndex}`"
              :slot="slot"
              @backup="createBackup"
              @migrate="migrateSlot"
              @delete="deleteSaveSlot"
            />
          </NSpace>
        </NCard>

        <!-- 模组版存档 -->
        <NCard size="small" class="save-card-glass">
          <template #header>
            <div class="flex items-center gap-2">
              <NTag type="warning" size="small" :bordered="false">{{ t("saves.kind.modded") }}</NTag>
              <span class="text-xs text-c-muted">
                {{ t("saves.slotCount", { n: moddedSlots.length }) }}
              </span>
            </div>
          </template>

          <NSpace v-if="moddedSlots.length > 0" vertical :size="8">
            <SlotCard
              v-for="slot in moddedSlots"
              :key="`m-${slot.steamUserId}-${slot.slotIndex}`"
              :slot="slot"
              @backup="createBackup"
              @migrate="migrateSlot"
              @delete="deleteSaveSlot"
            />
          </NSpace>
        </NCard>
      </div>

      <!-- 配对同步配置 -->
      <NCard class="mt-4">
        <template #header>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <NIcon :size="20" :color="'var(--primary-color)'"><ArrowRightLeft /></NIcon>
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

        <p class="text-sm mb-4 text-c-secondary">
          {{ t("saves.pairSync.description") }}
        </p>

        <div class="grid grid-cols-1 gap-3 mb-4">
          <div
            v-for="i in 3"
            :key="`pair-${i}`"
            class="pair-card rounded-xl p-4 border-2 transition-all"
            :class="getPairedModdedSlot(i) !== null
              ? 'border-primary-theme bg-primary-10-theme'
              : 'border-c-default bg-c-secondary'"
          >
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-3 flex-1">
                <div class="w-10 h-10 rounded-xl bg-primary-10-theme flex items-center justify-center flex-shrink-0">
                  <span class="text-sm font-bold text-primary-600-theme">V{{ i }}</span>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-c-muted">{{ t("saves.pairSync.vanillaSlot") }}</span>
                  <span class="text-sm font-medium text-c-primary">{{ t("saves.pairSync.slotWithNumber", { n: i }) }}</span>
                </div>
              </div>

              <div class="flex flex-col items-center flex-shrink-0">
                <NIcon :size="22" :color="getPairedModdedSlot(i) !== null ? 'var(--primary-color)' : 'var(--color-text-muted)'">
                  <ArrowRightLeft />
                </NIcon>
                <span
                  v-if="getPairedModdedSlot(i) !== null"
                  class="text-[10px] text-primary-theme mt-0.5"
                >{{ t("saves.pairSync.paired") }}</span>
                <span v-else class="text-[10px] text-c-muted mt-0.5">{{ t("saves.pairSync.unpaired") }}</span>
              </div>

              <div class="flex items-center gap-3 flex-1 justify-end">
                <div
                  class="w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0"
                  :class="getPairedModdedSlot(i) !== null ? 'bg-primary-20-theme' : 'bg-c-secondary'"
                >
                  <span
                    class="text-sm font-bold"
                    :class="getPairedModdedSlot(i) !== null ? 'text-primary-theme' : 'text-c-muted'"
                  >{{ getPairedModdedSlot(i) !== null ? `M${getPairedModdedSlot(i)}` : '?' }}</span>
                </div>
                <div class="flex flex-col items-end gap-1">
                  <span class="text-xs text-c-muted">{{ t("saves.pairSync.moddedSlot") }}</span>
                  <NSelect
                    :value="getPairedModdedSlot(i)"
                    :options="pairOptionsForSlot(i)"
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

        <div class="flex items-center justify-between pt-3 border-t border-c-default">
          <span class="text-xs text-c-muted">
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
            <NIcon :size="16" :color="'var(--primary-color)'"><Cloud /></NIcon>
            <span>{{ t("saves.cloud.title") }}</span>
          </div>
        </template>
        <p class="text-xs mb-3 text-c-muted">
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
    <AppDialog v-model:show="showBackupsDialog" width="680px">
      <template #header>
        <span class="text-lg font-semibold">{{ t("saves.allHistoryBackups") }}</span>
      </template>
      <div class="flex items-center gap-3 text-xs text-c-muted mb-4 px-1">
        <CountBadge :label="t('saves.backups.manualCountLabel')" :count="totalBackupStats.manual" dot-color="bg-blue-500" />
        <CountBadge :label="t('saves.backups.autoCountLabel')" :count="totalBackupStats.auto" />
        <span>· {{ t("saves.backups.limitLabel", { n: totalBackupStats.limit }) }}</span>
      </div>

      <EmptyState v-if="backups.length === 0" :icon="Database" :title="t('saves.backups.empty')" size="sm" />

      <div v-else class="max-h-[500px] overflow-auto space-y-3 pr-1">
        <div
          v-for="b in backups"
          :key="b.id"
          class="flex items-start justify-between p-4 rounded-lg border border-c-default bg-c-secondary"
        >
          <div class="flex-1 min-w-0 space-y-2">
            <!-- 第一行：日期 + 类型标签 -->
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-semibold text-c-primary whitespace-nowrap">
                {{ new Date(b.createdAt).toLocaleString(currentLocale) }}
              </span>
              <NTag
                v-if="b.manual === true"
                type="success"
                size="tiny"
                :bordered="false"
              >
                {{ t("saves.backups.manualReason") }}
              </NTag>
              <NTag
                v-else-if="b.manual === false"
                type="default"
                size="tiny"
                :bordered="false"
              >
                {{ t("saves.backups.autoBackup") }}
              </NTag>
              <template v-else>
                <NTag type="warning" size="tiny" :bordered="false">
                  {{ t("saves.backups.legacyBackup") }}
                </NTag>
                <FloatingTip :text="t('saves.backups.legacyBackupHint')" :width="280" />
              </template>
            </div>

            <!-- 第二行：Steam 用户 + 槽位 -->
            <div class="flex items-center gap-4 text-xs text-c-muted">
              <div class="flex items-center gap-1">
                <NIcon :size="12"><User /></NIcon>
                <span>{{ t("saves.steamUser") }}: {{ b.steamUserId }}</span>
              </div>
              <div class="flex items-center gap-1">
                <NIcon :size="12"><Database /></NIcon>
                <span>{{ kindLabel(t, b.kind) }} {{ t("saves.slotIndex", { i: b.slotIndex }) }}</span>
              </div>
            </div>

            <!-- 第三行：备份原因 -->
            <div class="text-xs text-c-muted">
              <span class="text-c-muted">{{ t("saves.backups.backupSource") }}:</span>
              <span class="ml-1">{{ b.reason }}</span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2 ml-4 flex-shrink-0">
            <NButton v-if="b.manual === null" size="tiny" secondary @click="openUpgradeDialog(b.id)">
              {{ t("saves.backups.upgrade") }}
            </NButton>
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
          </div>
        </div>
      </div>
    </AppDialog>

    <!-- 恢复到指定槽位对话框 -->
    <AppDialog v-if="restoreToSlotBackup" v-model:show="showRestoreToSlotDialog" width="400px">
      <template #header>
        <div class="flex items-center gap-2">
          <NIcon :size="18" :color="'var(--primary-color)'"><RotateCcw /></NIcon>
          <span class="text-lg font-semibold">{{ t("saves.backups.restoreToTitle") }}</span>
        </div>
      </template>

      <div class="text-sm text-c-secondary">
        <div class="mb-2">
          <span class="text-c-muted">{{ t("saves.backups.restoreFrom") }}:</span>
          <span class="ml-2 font-medium text-c-primary">
            {{ new Date(restoreToSlotBackup.createdAt).toLocaleString(currentLocale) }}
          </span>
        </div>
        <div class="text-xs mb-3 text-c-muted">
          {{ restoreToSlotBackup.reason }} · {{ kindLabel(t,restoreToSlotBackup.kind) }}
          {{ t("saves.slotIndex", { i: restoreToSlotBackup.slotIndex }) }}
        </div>

        <div class="p-3 rounded-lg border bg-c-secondary border-c-default">
          <div class="text-xs mb-2 text-c-muted">{{ t("saves.backups.restoreTarget") }}</div>
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

        <div class="mt-3 p-2 rounded bg-c-warning text-xs text-c-warning flex items-start gap-1.5">
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
    </AppDialog>

    <!-- 旧版备份升级对话框 -->
    <AppDialog v-model:show="showUpgradeDialog" width="400px">
      <template #header>
        <span class="text-lg font-semibold">{{ t("saves.backups.upgradeTitle") }}</span>
      </template>

      <div class="text-sm text-c-secondary space-y-3">
        <p>{{ t("saves.backups.upgradeDesc") }}</p>
        <NRadioGroup v-model:value="upgradeDialogManual">
          <NSpace vertical>
            <NRadio :value="true">{{ t("saves.backups.manualReason") }}</NRadio>
            <NRadio :value="false">{{ t("saves.backups.autoBackup") }}</NRadio>
          </NSpace>
        </NRadioGroup>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="upgradingBackupId = null">{{ t("common.cancel") }}</NButton>
          <NButton type="primary" @click="confirmUpgrade">{{ t("saves.backups.upgradeConfirm") }}</NButton>
        </NSpace>
      </template>
    </AppDialog>

    <!-- Steam 云存档对话框 -->
    <AppDialog v-model:show="showCloudDialog" width="640px">
      <template #header>
        <div class="flex items-center justify-between">
          <span class="text-lg font-semibold">{{ t("saves.cloud.title") }}</span>
          <NButton size="tiny" @click="cleanupArtifacts">{{ t("saves.cloud.cleanupArtifacts") }}</NButton>
        </div>
      </template>

        <div v-if="cloudStatus && cloudStatus.isAvailable" class="grid grid-cols-4 gap-2 mb-4">
          <div class="text-center p-2 rounded bg-c-secondary">
            <div class="text-lg font-bold text-c-primary">{{ cloudStatus.localFileCount }}</div>
            <div class="text-xs text-c-muted">{{ t("saves.cloud.localFiles") }}</div>
          </div>
          <div class="text-center p-2 rounded bg-c-secondary">
            <div class="text-lg font-bold text-c-primary">{{ cloudStatus.cloudFileCount }}</div>
            <div class="text-xs text-c-muted">{{ t("saves.cloud.cloudFiles") }}</div>
          </div>
          <div class="text-center p-2 rounded bg-c-secondary">
            <div class="text-lg font-bold text-c-warning">{{ cloudStatus.differentCount }}</div>
            <div class="text-xs text-c-muted">{{ t("saves.cloud.differences") }}</div>
          </div>
          <div
            class="text-center p-2 rounded"
            :class="cloudStatus.hasMismatch ? 'bg-c-warning' : 'bg-green-50'"
          >
            <NIcon :size="24" :color="cloudStatus.hasMismatch ? '#f0a020' : '#18a058'">
              <AlertTriangle v-if="cloudStatus.hasMismatch" />
              <Cloud v-else />
            </NIcon>
            <div class="text-xs mt-1" :class="cloudStatus.hasMismatch ? 'text-c-warning' : 'text-green-600'">
              {{ mismatchLabel(cloudStatus) }}
            </div>
          </div>
        </div>

        <div
          v-if="cloudStatus && !cloudStatus.isAvailable"
          class="p-4 rounded-lg bg-c-warning border border-c-warning mb-4"
        >
          <div class="flex items-start gap-2">
            <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
            <div>
              <div class="text-sm font-medium text-c-warning mb-1">{{ t("saves.cloud.unavailable") }}</div>
              <div class="text-xs text-c-warning">{{ cloudStatus.diagnostic }}</div>
              <div v-if="cloudStatus.cloudPath" class="text-xs mt-1 font-mono text-c-muted">
                {{ t("saves.cloud.cloudLabel") }}: {{ cloudStatus.cloudPath }}
              </div>
              <div v-if="cloudStatus.localPath" class="text-xs mt-0.5 font-mono text-c-muted">
                {{ t("saves.cloud.localLabel") }}: {{ cloudStatus.localPath }}
              </div>
            </div>
          </div>
        </div>

        <NSpace class="mb-4">
          <NButton size="small" secondary :disabled="!cloudStatus?.isAvailable" @click="ascendFull">{{ t("saves.cloud.ascend") }}</NButton>
          <NButton size="small" secondary :disabled="!cloudStatus?.isAvailable" @click="descendFull">{{ t("saves.cloud.descend") }}</NButton>
        </NSpace>

        <div v-if="cloudDiffs.length > 0" class="max-h-64 overflow-auto border rounded-lg border-c-default">
          <div
            v-for="d in cloudDiffs"
            :key="d.relativePath"
            class="flex items-center justify-between p-2 border-b last:border-b-0 text-sm border-c-default"
          >
            <div class="flex-1 min-w-0 flex items-center gap-2">
              <NTag :type="diffKindType(d.kind)" size="tiny" :bordered="false">
                {{ diffKindLabel(d.kind) }}
              </NTag>
              <span class="truncate font-mono text-xs text-c-primary">{{ d.relativePath }}</span>
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
        <div v-else class="text-center py-6 text-sm text-c-muted">
          {{ t("saves.cloud.inSync") }}
        </div>
    </AppDialog>
  </div>
</template>

<style scoped>
.save-card-glass {
  --n-border-color: color-mix(in srgb, var(--primary-color) 10%, var(--color-border)) !important;
  background-color: var(--glass-bg) !important;
  backdrop-filter: blur(var(--glass-blur));
}
</style>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { currentLocale } from "@/i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  Alert,
  Badge,
  Button,
  Card,
  EmptyState,
  PageHeader,
  Popconfirm,
  RadioGroup,
  Select,
  Switch,
  useToast,
} from "@xcmm1543/miuui-vue"
import SlotCard from "@/components/SlotCard.vue"
import {
  HardDrive,
  ArrowRightLeft,
  RefreshCw,
  Database,
  History,
  RotateCcw,
  Trash2,
  Upload,
  Download,
  Cloud,
  AlertTriangle,
  User,
  Check,
} from "@lucide/vue"
import type {
  SaveSlot,
  SaveBackupEntry,
  SaveSyncPair,
  SaveSyncResult,
  CloudSaveStatus,
  CloudSaveDiffEntry,
  AppBootstrap,
} from "../types"
import { useStorage } from "@/composables/useStorage"
import { useIsActive } from "@/composables/useIsActive"
import { kindLabel } from "@/utils/kindLabel"
import { yieldToMain } from "@/utils/scheduler"
import AppDialog from "@/components/AppDialog.vue"
import FloatingTip from "@/components/FloatingTip.vue"

const { t } = useI18n()
const { toast } = useToast()
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
const restoreToSlotTarget = ref<{ kind: string; slotIndex: number }>({
  kind: "vanilla",
  slotIndex: 1,
})

// 云存档
const cloudStatus = ref<CloudSaveStatus | null>(null)
const cloudDiffs = ref<CloudSaveDiffEntry[]>([])
const showCloudDialog = ref(false)
const cloudStatusLoading = ref(false)
const cloudActionLoading = ref(false)

// 备份计数
const slotBackupCounts = ref<Record<string, { manual: number; auto: number; keepCount: number }>>(
  {},
)
const autoBackupKeepCount = ref(5)

/** 所有槽位的备份汇总 */
const totalBackupStats = computed(() => {
  let manual = 0,
    auto = 0
  for (const c of Object.values(slotBackupCounts.value)) {
    manual += c.manual
    auto += c.auto
  }
  return { manual, auto, limit: autoBackupKeepCount.value }
})

let backupCountsReqSeq = 0
let slotsReqSeq = 0

async function loadBackupCounts() {
  const reqId = ++backupCountsReqSeq
  try {
    const [allBackups, bootstrap] = await Promise.all([
      invoke<SaveBackupEntry[]>("list_save_backups", {}),
      invoke<AppBootstrap>("get_app_bootstrap"),
    ])
    if (!isActive.value || reqId !== backupCountsReqSeq) return
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
  currentUserSlots.value
    .filter((s) => s.kind === "vanilla")
    .sort((a, b) => a.slotIndex - b.slotIndex),
)
const moddedSlots = computed(() =>
  currentUserSlots.value
    .filter((s) => s.kind === "modded")
    .sort((a, b) => a.slotIndex - b.slotIndex),
)

const pairOptions = computed(() => {
  const slots = [1, 2, 3]
  return slots.map((m) => ({ label: t("saves.pairSync.slotWithNumber", { n: m }), value: m }))
})

function pairOptionsForSlot(vanillaSlot: number): Array<{ label: string; value: number }> {
  // 已被其他原版槽位占用的模组版槽位
  const taken = syncPairs.value
    .filter((p) => p.vanillaSlot !== vanillaSlot)
    .map((p) => p.moddedSlot)
  return [
    { label: t("saves.pairSync.noPair"), value: -1 },
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
  const reqId = ++slotsReqSeq
  loading.value = true
  try {
    const data = await invoke<SaveSlot[]>("list_save_slots")
    if (!isActive.value || reqId !== slotsReqSeq) return
    slots.value = data
    if (slots.value.length > 0 && !activeUserId.value) {
      activeUserId.value = slots.value[0].steamUserId
    }
    void loadBackupCounts()
  } catch {
    if (isActive.value && reqId === slotsReqSeq) {
      slots.value = []
    }
  } finally {
    if (isActive.value && reqId === slotsReqSeq) {
      loading.value = false
    }
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
    toast({ title: t("saves.success.slotDeleted", { i: slot.slotIndex }), variant: "success" })
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.deleteSlotFailed") + ": " + String(e), variant: "error" })
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
    toast({ title: t("saves.error.loadBackupsFailed") + ": " + String(e), variant: "error" })
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
    toast({ title: t("saves.success.slotBackedUp", { i: slot.slotIndex }), variant: "success" })
    await loadBackupCounts()
    return entry
  } catch (e: unknown) {
    if (!isActive.value) return null
    toast({ title: t("saves.error.backupFailed") + ": " + String(e), variant: "error" })
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
    toast({
      title: t("saves.success.backupRestoredToSlot", {
        kind: kindLabel(t, target.kind),
        i: target.slotIndex,
      }),
      variant: "success",
    })
    showRestoreToSlotDialog.value = false
    showBackupsDialog.value = false
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.restoreFailed") + ": " + String(e), variant: "error" })
  }
}

async function deleteBackup(backup: SaveBackupEntry) {
  try {
    await invoke("delete_save_backup", { backupId: backup.id })
    if (!isActive.value) return
    toast({ title: t("saves.success.backupDeleted"), variant: "success" })
    backups.value = backups.value.filter((b) => b.id !== backup.id)
    await loadBackupCounts()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.backupDeleteFailed") + ": " + String(e), variant: "error" })
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
    toast({ title: t("saves.success.backupUpgraded"), variant: "success" })
    // 更新本地缓存
    const b = backups.value.find((x) => x.id === bid)
    if (b) b.manual = upgradeDialogManual.value
    await loadBackupCounts()
    showUpgradeDialog.value = false
    upgradingBackupId.value = null
  } catch (e: unknown) {
    toast({ title: t("saves.error.backupUpgradeFailed") + ": " + String(e), variant: "error" })
  }
}

// --- 同步 ---
async function handleSync() {
  loading.value = true
  try {
    const result = await invoke<SaveSyncResult>("sync_saves")
    if (!isActive.value) return
    if (result.syncedCount > 0) {
      toast({
        title: t("saves.success.syncPairsDone", { n: result.syncedCount }),
        variant: "success",
      })
    } else {
      toast({ title: t("saves.info.noSyncNeeded"), variant: "info" })
    }
    await loadSlots()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.syncFailed") + ": " + String(e), variant: "error" })
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
    toast({ title: t("saves.error.toggleFailed") + ": " + String(e), variant: "error" })
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
    toast({ title: t("saves.success.syncPairsSaved"), variant: "success" })
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.pairSaveFailed") + ": " + String(e), variant: "error" })
  }
}

// --- 云存档 ---
async function openCloudDialog() {
  if (cloudStatusLoading.value) return
  cloudStatusLoading.value = true
  try {
    const [status, diffs] = await Promise.all([
      invoke<CloudSaveStatus>("get_cloud_save_status"),
      invoke<CloudSaveDiffEntry[]>("list_cloud_save_diff_entries"),
    ])
    if (!isActive.value) return
    cloudStatus.value = status
    cloudDiffs.value = diffs
    showCloudDialog.value = true
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.cloudUnavailable") + ": " + String(e), variant: "error" })
  } finally {
    cloudStatusLoading.value = false
  }
}

async function copyCloudSide(relPath: string, side: string) {
  try {
    await invoke("copy_cloud_save_diff_side", { relativePath: relPath, side })
    if (!isActive.value) return
    toast({ title: t("saves.success.copied"), variant: "success" })
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.copyFailed") + ": " + String(e), variant: "error" })
  }
}

async function ascendFull() {
  if (cloudActionLoading.value) return
  cloudActionLoading.value = true
  try {
    await invoke("ascend_to_cloud_full")
    if (!isActive.value) return
    toast({ title: t("saves.success.ascended"), variant: "success" })
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.ascendFailed") + ": " + String(e), variant: "error" })
  } finally {
    cloudActionLoading.value = false
  }
}

async function descendFull() {
  if (cloudActionLoading.value) return
  cloudActionLoading.value = true
  try {
    await invoke("descend_from_cloud_full")
    if (!isActive.value) return
    toast({ title: t("saves.success.descended"), variant: "success" })
    await openCloudDialog()
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.descendFailed") + ": " + String(e), variant: "error" })
  } finally {
    cloudActionLoading.value = false
  }
}

async function cleanupArtifacts() {
  try {
    await invoke("cleanup_backup_artifacts")
    if (!isActive.value) return
    toast({ title: t("saves.success.artifactsCleaned"), variant: "success" })
  } catch (e: unknown) {
    if (!isActive.value) return
    toast({ title: t("saves.error.cleanupFailed") + ": " + String(e), variant: "error" })
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

function diffKindType(k: string): "success" | "warning" | "danger" | "info" | "neutral" {
  const m: Record<string, "success" | "warning" | "danger" | "info"> = {
    in_sync: "success",
    different: "warning",
    local_only: "info",
    cloud_only: "info",
  }
  return m[k] ?? "neutral"
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
  await yieldToMain()
  if (!isActive.value) return
  void loadSlots()
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    if (!isActive.value) return
    syncPairs.value = bootstrap.saveSyncPairs ?? []
    autoSync.value = bootstrap.saveAutoSync ?? false
  } catch {
    /* ignore */
  }
})
</script>

<template>
  <div>
    <!-- 头部 -->
    <PageHeader :title="t('saves.title')" :description="t('saves.subtitle')" class="mb-5">
      <template #actions>
        <Button variant="secondary" :loading="loading" @click="loadSlots">
          <template #icon><RefreshCw :size="16" /></template>
          {{ t("common.refresh") }}
        </Button>
        <Button variant="secondary" @click="handleSync">
          <template #icon><ArrowRightLeft :size="16" /></template>
          {{ t("saves.sync") }}
        </Button>
        <Button variant="secondary" @click="openAllBackups">
          <template #icon><History :size="16" /></template>
          {{ t("saves.allHistoryBackups") }}
        </Button>
      </template>
    </PageHeader>

    <!-- 空状态 -->
    <Card v-if="slots.length === 0 && !loading" material="glass" padding="sm">
      <EmptyState :title="t('saves.empty.setGamePath')">
        <template #icon><HardDrive :size="40" /></template>
      </EmptyState>
    </Card>

    <template v-else>
      <!-- 多用户切换 -->
      <div v-if="userIds.length > 1" class="flex items-center gap-3 mb-4">
        <User :size="16" class="text-c-muted" />
        <span class="text-xs text-c-muted">{{ t("saves.steamUser") }}:</span>
        <div class="flex gap-1.5">
          <Button
            v-for="uid in userIds"
            :key="uid"
            size="xs"
            :variant="uid === safeActiveUserId ? 'solid' : 'secondary'"
            @click="activeUserId = uid"
          >
            <template #icon><User :size="12" /></template>
            {{ shortUserId(uid) }}
          </Button>
        </div>
      </div>

      <!-- 双列布局 -->
      <div class="grid grid-cols-2 gap-6">
        <!-- 原版存档 -->
        <Card padding="sm" material="glass" class="save-card-glass">
          <template #header>
            <div class="flex items-center gap-2">
              <Badge variant="info" size="sm">{{ t("saves.kind.vanilla") }}</Badge>
              <span class="text-xs text-c-muted">
                {{ t("saves.slotCount", { n: vanillaSlots.length }) }}
              </span>
            </div>
          </template>

          <div v-if="vanillaSlots.length > 0" class="flex flex-col gap-2">
            <SlotCard
              v-for="slot in vanillaSlots"
              :key="`v-${slot.steamUserId}-${slot.slotIndex}`"
              :slot="slot"
              @backup="createBackup"
              @migrate="migrateSlot"
              @delete="deleteSaveSlot"
            />
          </div>
        </Card>

        <!-- 模组版存档 -->
        <Card padding="sm" material="glass" class="save-card-glass">
          <template #header>
            <div class="flex items-center gap-2">
              <Badge variant="warning" size="sm">{{ t("saves.kind.modded") }}</Badge>
              <span class="text-xs text-c-muted">
                {{ t("saves.slotCount", { n: moddedSlots.length }) }}
              </span>
            </div>
          </template>

          <div v-if="moddedSlots.length > 0" class="flex flex-col gap-2">
            <SlotCard
              v-for="slot in moddedSlots"
              :key="`m-${slot.steamUserId}-${slot.slotIndex}`"
              :slot="slot"
              @backup="createBackup"
              @migrate="migrateSlot"
              @delete="deleteSaveSlot"
            />
          </div>
        </Card>
      </div>

      <!-- 配对同步配置 -->
      <Card material="glass" class="save-card-glass mt-4">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center gap-2.5">
              <ArrowRightLeft :size="18" class="text-primary-theme" />
              <span class="text-base font-semibold text-c-primary">{{
                t("saves.pairSync.title")
              }}</span>
              <Badge :variant="autoSync ? 'success' : 'neutral'" size="sm">
                {{ autoSync ? t("saves.pairSync.autoSyncing") : t("saves.pairSync.manual") }}
              </Badge>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs text-c-muted">{{
                autoSync ? t("saves.pairSync.on") : t("saves.pairSync.off")
              }}</span>
              <Switch :model-value="autoSync" size="sm" @update:model-value="toggleAutoSync" />
            </div>
          </div>
        </template>
        <p class="text-sm mb-4 text-c-secondary">
          {{ t("saves.pairSync.description") }}
        </p>

        <div class="grid grid-cols-1 gap-3 mb-4">
          <div
            v-for="i in 3"
            :key="`pair-${i}`"
            class="pair-card rounded-xl p-4 border transition-all"
            :class="getPairedModdedSlot(i) !== null ? 'border-primary-theme' : 'border-c-default'"
            :style="{
              backgroundColor:
                getPairedModdedSlot(i) !== null
                  ? 'color-mix(in srgb, var(--primary-color) 12%, var(--glass-surface-bg))'
                  : 'var(--glass-surface-bg)',
              backdropFilter: 'blur(var(--glass-surface-blur))',
              WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
              boxShadow: 'var(--shadow-glass-surface)',
            }"
          >
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-3 flex-1">
                <div
                  class="w-10 h-10 rounded-xl bg-primary-10-theme flex items-center justify-center flex-shrink-0"
                >
                  <span class="text-sm font-bold text-primary-600-theme">V{{ i }}</span>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-c-muted">{{ t("saves.pairSync.vanillaSlot") }}</span>
                  <span class="text-sm font-medium text-c-primary">{{
                    t("saves.pairSync.slotWithNumber", { n: i })
                  }}</span>
                </div>
              </div>

              <div class="flex flex-col items-center flex-shrink-0">
                <ArrowRightLeft
                  :size="22"
                  :class="getPairedModdedSlot(i) !== null ? 'text-primary-theme' : 'text-c-muted'"
                />
                <span
                  v-if="getPairedModdedSlot(i) !== null"
                  class="text-[10px] text-primary-theme mt-0.5"
                  >{{ t("saves.pairSync.paired") }}</span
                >
                <span v-else class="text-[10px] text-c-muted mt-0.5">{{
                  t("saves.pairSync.unpaired")
                }}</span>
              </div>

              <div class="flex items-center gap-3 flex-1 justify-end">
                <div
                  class="w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0"
                  :class="
                    getPairedModdedSlot(i) !== null ? 'bg-primary-20-theme' : 'bg-c-secondary/50'
                  "
                >
                  <span
                    class="text-sm font-bold"
                    :class="getPairedModdedSlot(i) !== null ? 'text-primary-theme' : 'text-c-muted'"
                    >{{
                      getPairedModdedSlot(i) !== null ? `M${getPairedModdedSlot(i)}` : "?"
                    }}</span
                  >
                </div>
                <div class="flex flex-col items-end gap-1">
                  <span class="text-xs text-c-muted">{{ t("saves.pairSync.moddedSlot") }}</span>
                  <Select
                    :model-value="getPairedModdedSlot(i) ?? -1"
                    :options="pairOptionsForSlot(i)"
                    size="sm"
                    class="w-28"
                    :placeholder="t('saves.pairSync.chooseSlot')"
                    @update:model-value="
                      (v) => {
                        if (v != null) updatePair(i, Number(v) === -1 ? null : Number(v))
                      }
                    "
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
          <Button variant="solid" size="sm" @click="saveSyncPairs">
            <template #icon><ArrowRightLeft :size="14" /></template>
            {{ t("saves.pairSync.savePairs") }}
          </Button>
        </div>
      </Card>

      <!-- Steam 云存档 -->
      <Card material="glass" class="save-card-glass mt-4">
        <template #header>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <Cloud :size="18" class="text-primary-theme" />
              <span class="text-base font-semibold text-c-primary">{{
                t("saves.cloud.title")
              }}</span>
            </div>
          </div>
        </template>
        <p class="text-sm mb-4 text-c-secondary">
          {{ t("saves.cloud.description") }}
        </p>
        <div class="flex items-center gap-2 flex-wrap">
          <Button
            variant="secondary"
            size="sm"
            :loading="cloudStatusLoading"
            @click="openCloudDialog"
          >
            <template #icon><Cloud :size="14" /></template>
            {{ t("saves.cloud.viewStatus") }}
          </Button>
          <Button variant="secondary" size="sm" :loading="cloudActionLoading" @click="ascendFull">
            <template #icon><Upload :size="14" /></template>
            {{ t("saves.cloud.uploadAll") }}
          </Button>
          <Button variant="secondary" size="sm" :loading="cloudActionLoading" @click="descendFull">
            <template #icon><Download :size="14" /></template>
            {{ t("saves.cloud.downloadAll") }}
          </Button>
        </div>
      </Card>
    </template>

    <!-- 备份列表对话框 -->
    <AppDialog v-model:show="showBackupsDialog" width="680px">
      <template #header>
        <span class="text-lg font-semibold">{{ t("saves.allHistoryBackups") }}</span>
      </template>
      <div class="flex items-center gap-3 text-xs text-c-muted mb-4 px-1">
        <Badge dot variant="info" size="sm">
          {{ t("saves.backups.manualCountLabel") }} {{ totalBackupStats.manual }}
        </Badge>
        <Badge dot variant="neutral" size="sm">
          {{ t("saves.backups.autoCountLabel") }} {{ totalBackupStats.auto }}
        </Badge>
        <span>· {{ t("saves.backups.limitLabel", { n: totalBackupStats.limit }) }}</span>
      </div>

      <EmptyState v-if="backups.length === 0" :title="t('saves.backups.empty')">
        <template #icon><Database :size="32" /></template>
      </EmptyState>

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
              <Badge v-if="b.manual === true" variant="success" size="xs">
                {{ t("saves.backups.manualReason") }}
              </Badge>
              <Badge v-else-if="b.manual === false" variant="neutral" size="xs">
                {{ t("saves.backups.autoBackup") }}
              </Badge>
              <template v-else>
                <Badge variant="warning" size="xs">
                  {{ t("saves.backups.legacyBackup") }}
                </Badge>
                <FloatingTip :text="t('saves.backups.legacyBackupHint')" :width="280" />
              </template>
            </div>

            <!-- 第二行：Steam 用户 + 槽位 -->
            <div class="flex items-center gap-4 text-xs text-c-muted">
              <div class="flex items-center gap-1">
                <User :size="12" />
                <span>{{ t("saves.steamUser") }}: {{ b.steamUserId }}</span>
              </div>
              <div class="flex items-center gap-1">
                <Database :size="12" />
                <span
                  >{{ kindLabel(t, b.kind) }} {{ t("saves.slotIndex", { i: b.slotIndex }) }}</span
                >
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
            <Button
              v-if="b.manual === null"
              size="xs"
              variant="secondary"
              @click="openUpgradeDialog(b.id)"
            >
              {{ t("saves.backups.upgrade") }}
            </Button>
            <Button size="xs" variant="secondary" @click="openRestoreToSlot(b)">
              <template #icon><RotateCcw :size="12" /></template>
              {{ t("saves.backups.restoreTo") }}
            </Button>
            <Popconfirm
              :title="t('saves.backups.confirmDelete')"
              danger
              material="glass"
              @confirm="deleteBackup(b)"
            >
              <template #trigger>
                <Button
                  size="xs"
                  variant="ghost"
                  icon-only
                  :aria-label="t('common.delete')"
                  class="text-red-500"
                >
                  <Trash2 :size="12" />
                </Button>
              </template>
            </Popconfirm>
          </div>
        </div>
      </div>
    </AppDialog>

    <!-- 恢复到指定槽位对话框 -->
    <AppDialog v-if="restoreToSlotBackup" v-model:show="showRestoreToSlotDialog" width="400px">
      <template #header>
        <div class="flex items-center gap-2">
          <RotateCcw :size="18" class="text-primary-theme" />
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
          {{ restoreToSlotBackup.reason }} · {{ kindLabel(t, restoreToSlotBackup.kind) }}
          {{ t("saves.slotIndex", { i: restoreToSlotBackup.slotIndex }) }}
        </div>

        <div class="p-3 rounded-lg border bg-c-secondary border-c-default">
          <div class="text-xs mb-2 text-c-muted">{{ t("saves.backups.restoreTarget") }}</div>
          <div class="flex flex-col gap-2">
            <RadioGroup
              :model-value="restoreToSlotTarget.kind"
              :options="[
                { value: 'vanilla', label: t('saves.kind.vanilla') },
                { value: 'modded', label: t('saves.kind.modded') },
              ]"
              orientation="horizontal"
              size="sm"
              @update:model-value="(v: string | number) => setRestoreTargetKind(String(v))"
            />
            <Select
              :model-value="restoreToSlotTarget.slotIndex"
              :options="[1, 2, 3].map((i) => ({ label: t('saves.slotIndex', { i }), value: i }))"
              size="sm"
              class="w-32"
              :placeholder="t('saves.backups.chooseSlot')"
              @update:model-value="
                (v) => {
                  if (v != null) setRestoreTargetSlot(Number(v))
                }
              "
            />
          </div>
        </div>

        <Alert variant="warning" material="glass" class="mt-3">
          <template #icon><AlertTriangle :size="14" /></template>
          <span class="text-xs">{{ t("saves.backups.restoreConfirmHint") }}</span>
        </Alert>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <Button variant="ghost" size="sm" @click="showRestoreToSlotDialog = false">{{
            t("common.cancel")
          }}</Button>
          <Button variant="solid" size="sm" @click="doRestoreToSlot">
            {{ t("saves.backups.confirmRestore") }}
          </Button>
        </div>
      </template>
    </AppDialog>

    <!-- 旧版备份升级对话框 -->
    <AppDialog v-model:show="showUpgradeDialog" width="400px">
      <template #header>
        <span class="text-lg font-semibold">{{ t("saves.backups.upgradeTitle") }}</span>
      </template>

      <div class="text-sm text-c-secondary space-y-3">
        <p>{{ t("saves.backups.upgradeDesc") }}</p>
        <RadioGroup
          :model-value="upgradeDialogManual ? 'manual' : 'auto'"
          :options="[
            { value: 'manual', label: t('saves.backups.manualReason') },
            { value: 'auto', label: t('saves.backups.autoBackup') },
          ]"
          orientation="vertical"
          size="sm"
          @update:model-value="(v: string | number) => (upgradeDialogManual = v === 'manual')"
        />
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <Button
            variant="ghost"
            size="sm"
            @click="((showUpgradeDialog = false), (upgradingBackupId = null))"
            >{{ t("common.cancel") }}</Button
          >
          <Button variant="solid" size="sm" @click="confirmUpgrade">{{
            t("saves.backups.upgradeConfirm")
          }}</Button>
        </div>
      </template>
    </AppDialog>

    <!-- Steam 云存档对话框 -->
    <AppDialog v-model:show="showCloudDialog" width="640px">
      <template #header>
        <div class="flex items-center justify-between w-full gap-3 pr-2">
          <span class="text-lg font-semibold">{{ t("saves.cloud.title") }}</span>
          <Button size="xs" variant="secondary" @click="cleanupArtifacts">
            {{ t("saves.cloud.cleanupArtifacts") }}
            <FloatingTip
              :text="t('saves.cloud.cleanupArtifactsTip')"
              :width="240"
              placement="bottom-end"
            />
          </Button>
        </div>
      </template>

      <div v-if="cloudStatus && cloudStatus.isAvailable" class="grid grid-cols-4 gap-3 mb-5">
        <!-- 本地文件 -->
        <div
          class="p-3.5 rounded-xl flex flex-col items-center justify-center transition-all duration-200 hover:-translate-y-0.5 border border-c-default"
          :style="{
            backgroundColor: 'var(--glass-surface-bg)',
            backdropFilter: 'blur(var(--glass-surface-blur))',
            WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
            boxShadow: 'var(--shadow-glass-surface)',
          }"
        >
          <div class="text-xl font-bold font-mono text-c-primary leading-tight">
            {{ cloudStatus.localFileCount }}
          </div>
          <div class="text-xs text-c-muted mt-1 font-medium">
            {{ t("saves.cloud.localFiles") }}
          </div>
        </div>

        <!-- 云端文件 -->
        <div
          class="p-3.5 rounded-xl flex flex-col items-center justify-center transition-all duration-200 hover:-translate-y-0.5 border border-c-default"
          :style="{
            backgroundColor: 'var(--glass-surface-bg)',
            backdropFilter: 'blur(var(--glass-surface-blur))',
            WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
            boxShadow: 'var(--shadow-glass-surface)',
          }"
        >
          <div class="text-xl font-bold font-mono text-c-primary leading-tight">
            {{ cloudStatus.cloudFileCount }}
          </div>
          <div class="text-xs text-c-muted mt-1 font-medium">
            {{ t("saves.cloud.cloudFiles") }}
          </div>
        </div>

        <!-- 差异数量 -->
        <div
          class="p-3.5 rounded-xl flex flex-col items-center justify-center transition-all duration-200 hover:-translate-y-0.5 border"
          :class="cloudStatus.differentCount > 0 ? 'border-amber-500/40' : 'border-c-default'"
          :style="{
            backgroundColor:
              cloudStatus.differentCount > 0
                ? 'color-mix(in srgb, #f59e0b 10%, var(--glass-surface-bg))'
                : 'var(--glass-surface-bg)',
            backdropFilter: 'blur(var(--glass-surface-blur))',
            WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
            boxShadow: 'var(--shadow-glass-surface)',
          }"
        >
          <div
            class="text-xl font-bold font-mono leading-tight"
            :class="cloudStatus.differentCount > 0 ? 'text-amber-500' : 'text-c-primary'"
          >
            {{ cloudStatus.differentCount }}
          </div>
          <div class="text-xs text-c-muted mt-1 font-medium">
            {{ t("saves.cloud.differences") }}
          </div>
        </div>

        <!-- 同步状态 -->
        <div
          class="p-3.5 rounded-xl flex flex-col items-center justify-center transition-all duration-200 hover:-translate-y-0.5 border"
          :class="cloudStatus.hasMismatch ? 'border-amber-500/40' : 'border-emerald-500/40'"
          :style="{
            backgroundColor: cloudStatus.hasMismatch
              ? 'color-mix(in srgb, #f59e0b 10%, var(--glass-surface-bg))'
              : 'color-mix(in srgb, #10b981 10%, var(--glass-surface-bg))',
            backdropFilter: 'blur(var(--glass-surface-blur))',
            WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
            boxShadow: 'var(--shadow-glass-surface)',
          }"
        >
          <div class="flex items-center gap-1.5 leading-tight">
            <component
              :is="cloudStatus.hasMismatch ? AlertTriangle : Check"
              :size="18"
              :class="cloudStatus.hasMismatch ? 'text-amber-500' : 'text-emerald-500'"
            />
            <span
              class="text-sm font-bold truncate max-w-[80px]"
              :class="cloudStatus.hasMismatch ? 'text-amber-500' : 'text-emerald-500'"
            >
              {{ mismatchLabel(cloudStatus) }}
            </span>
          </div>
          <div class="text-xs text-c-muted mt-1 font-medium">
            {{ t("saves.cloud.title") }}
          </div>
        </div>
      </div>

      <Alert
        v-if="cloudStatus && !cloudStatus.isAvailable"
        variant="warning"
        material="glass"
        :title="t('saves.cloud.unavailable')"
        class="mb-4"
      >
        <template #icon><AlertTriangle :size="18" class="text-amber-500" /></template>
        <div class="text-xs space-y-1">
          <div>{{ cloudStatus.diagnostic }}</div>
          <div v-if="cloudStatus.cloudPath" class="font-mono text-c-muted">
            {{ t("saves.cloud.cloudLabel") }}: {{ cloudStatus.cloudPath }}
          </div>
          <div v-if="cloudStatus.localPath" class="font-mono text-c-muted">
            {{ t("saves.cloud.localLabel") }}: {{ cloudStatus.localPath }}
          </div>
        </div>
      </Alert>

      <div class="flex items-center gap-2 mb-4">
        <Button
          size="sm"
          variant="secondary"
          :disabled="!cloudStatus?.isAvailable"
          :loading="cloudActionLoading"
          @click="ascendFull"
        >
          <template #icon><Upload :size="14" /></template>
          {{ t("saves.cloud.ascend") }}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          :disabled="!cloudStatus?.isAvailable"
          :loading="cloudActionLoading"
          @click="descendFull"
        >
          <template #icon><Download :size="14" /></template>
          {{ t("saves.cloud.descend") }}
        </Button>
      </div>

      <div
        v-if="cloudDiffs.length > 0"
        class="rounded-xl border border-c-default overflow-hidden"
        :style="{
          backgroundColor: 'var(--glass-surface-bg)',
          backdropFilter: 'blur(var(--glass-surface-blur))',
          WebkitBackdropFilter: 'blur(var(--glass-surface-blur))',
        }"
      >
        <div class="max-h-64 overflow-y-auto divide-y divide-c-default">
          <div
            v-for="d in cloudDiffs"
            :key="d.relativePath"
            class="flex items-center justify-between p-2.5 text-sm transition-colors hover:bg-c-secondary/40"
          >
            <div class="flex-1 min-w-0 flex items-center gap-2">
              <Badge :variant="diffKindType(d.kind)" size="xs">
                {{ diffKindLabel(d.kind) }}
              </Badge>
              <span class="truncate font-mono text-xs text-c-primary">{{ d.relativePath }}</span>
            </div>
            <div class="flex items-center gap-1 flex-shrink-0 ml-3">
              <Button
                v-if="d.kind === 'local_only'"
                size="xs"
                variant="secondary"
                @click="copyCloudSide(d.relativePath, 'local_to_cloud')"
              >
                {{ t("saves.cloud.toCloud") }}
              </Button>
              <Button
                v-if="d.kind === 'cloud_only'"
                size="xs"
                variant="secondary"
                @click="copyCloudSide(d.relativePath, 'cloud_to_local')"
              >
                {{ t("saves.cloud.toLocal") }}
              </Button>
            </div>
          </div>
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
  background-color: var(--glass-surface-bg);
  -webkit-backdrop-filter: blur(var(--glass-surface-blur));
  backdrop-filter: blur(var(--glass-surface-blur));
  box-shadow: var(--shadow-glass-surface) !important;
}
</style>

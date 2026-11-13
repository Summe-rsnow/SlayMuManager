<script setup lang="ts">
import { ref, computed } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { Alert, Button, Badge, Tooltip, useToast, useConfirm } from "@miuui/vue"
import { Trash2, FolderOpen, ShieldCheck, ArrowRightLeft, AlertTriangle, Check } from "@lucide/vue"
import AppDialog from "./AppDialog.vue"
import ConfirmBtn from "./ConfirmBtn.vue"
import type { InstalledMod } from "../types"
import { useModCacheStore } from "@/stores/useModCacheStore"

const props = defineProps<{
  show: boolean
  modId: string | null
}>()

const emit = defineEmits<{
  (e: "update:show", value: boolean): void
}>()

const { t } = useI18n()
const { toast } = useToast()
const { confirm } = useConfirm()
const modCacheStore = useModCacheStore()

const busy = ref(false)

const group = computed(() => {
  if (!props.modId) return null
  return modCacheStore.getConflictGroup(props.modId) ?? null
})

/** 打开实例所在物理文件夹 */
async function handleOpenFolder(mod: InstalledMod) {
  try {
    if (mod.installDir) {
      await invoke("open_mod_folder_by_path", { installDir: mod.installDir })
    }
  } catch (e: unknown) {
    toast.error(String(e))
  }
}

/** 单独删除某个版本实例 */
async function handleDeleteInstance(mod: InstalledMod) {
  if (busy.value) return
  busy.value = true
  try {
    await invoke("uninstall_mod_instance", {
      installDir: mod.installDir,
      modId: mod.id,
    })
    toast.success(
      t("library.success.uninstalled", {
        name: `${mod.name} (v${mod.version ?? mod.folderName})`,
      }),
    )
    await modCacheStore.fetchMods()
    if (!modCacheStore.hasConflict(mod.id)) {
      emit("update:show", false)
    }
  } catch (e: unknown) {
    toast.error(String(e))
  } finally {
    busy.value = false
  }
}

/** 切换激活版本：将当前版本设为启用，并把同模组其他已启用版本设为禁用 */
async function handleSwitchActive(mod: InstalledMod) {
  if (busy.value) return
  busy.value = true
  try {
    await invoke("switch_active_mod_version", {
      installDir: mod.installDir,
      modId: mod.id,
    })
    toast.success(`已切换生效版本为: v${mod.version ?? mod.folderName}`)
    await modCacheStore.fetchMods()
  } catch (e: unknown) {
    toast.error(`版本切换失败: ${String(e)}`)
  } finally {
    busy.value = false
  }
}

/** 保留此版本并清理其他所有旧版本 */
async function confirmKeepThisAndCleanOthers(targetMod: InstalledMod) {
  if (!group.value || busy.value) return
  const otherCount = group.value.instances.length - 1
  const ok = await confirm({
    title: "保留此版本并清理旧版本",
    description: `将保留版本「v${targetMod.version ?? targetMod.folderName}」，并彻底删除其余 ${otherCount} 个多余物理版本。此操作不可撤销，确认继续？`,
    confirmLabel: "确认清理",
    cancelLabel: t("common.cancel"),
    danger: true,
  })
  if (!ok) return
  busy.value = true
  try {
    const cleanedCount = await invoke<number>("keep_mod_version_and_clean_others", {
      installDir: targetMod.installDir,
      modId: targetMod.id,
    })
    toast.success(
      `已保留版本 v${targetMod.version ?? targetMod.folderName}，并成功清理 ${cleanedCount} 个旧版本！`,
    )
    await modCacheStore.fetchMods()
    emit("update:show", false)
  } catch (e: unknown) {
    toast.error(`清理失败: ${String(e)}`)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <AppDialog
    :show="show"
    :title="`模组版本共存管理 - ${group?.name ?? ''}`"
    width="680px"
    @update:show="(v) => emit('update:show', v)"
  >
    <div v-if="group" class="space-y-4">
      <!-- 提示横条 -->
      <Alert variant="warning" material="glass">
        <template #icon><AlertTriangle :size="18" class="text-amber-500" /></template>
        检测到系统中共存
        <strong>{{ group.instances.length }}</strong>
        个不同版本的安装实例。请选择需要的版本生效，或清理冗余文件避免冲突。
      </Alert>

      <!-- 版本实例列表 -->
      <div class="space-y-2.5 max-h-[380px] overflow-y-auto pr-1">
        <div
          v-for="item in group.instances"
          :key="item.installDir"
          class="p-3 rounded-xl border flex items-center justify-between gap-3 transition-all duration-200"
          :style="{
            backgroundColor:
              item.state === 'enabled'
                ? 'color-mix(in srgb, var(--primary-color) 6%, var(--glass-surface-bg))'
                : 'var(--glass-surface-bg)',
            borderColor:
              item.state === 'enabled'
                ? 'color-mix(in srgb, var(--primary-color) 40%, var(--glass-surface-border))'
                : 'var(--glass-surface-border)',
          }"
        >
          <!-- 左侧信息 -->
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="font-bold text-[15px] font-mono text-c-primary">
                v{{ item.version ?? "未知版本" }}
              </span>
              <Badge v-if="item.state === 'enabled'" variant="success" size="xs">
                <Check :size="12" />
                当前生效
              </Badge>
              <Badge v-else variant="neutral" size="xs"> 已禁用 </Badge>
              <Badge v-if="item.source === 'workshop'" variant="info" size="xs"> 创意工坊 </Badge>
            </div>

            <!-- 文件夹路径 -->
            <div class="text-xs text-c-muted font-mono mt-1 truncate" :title="item.installDir">
              📁 {{ item.folderName }}
              <span class="opacity-60 text-[11px] ml-1">({{ item.installDir }})</span>
            </div>
          </div>

          <!-- 右侧操作 -->
          <div class="flex items-center gap-1.5 shrink-0">
            <!-- 切换为此版本生效 -->
            <Button
              v-if="item.state !== 'enabled' && item.source !== 'workshop'"
              size="sm"
              variant="secondary"
              :disabled="busy"
              @click="handleSwitchActive(item)"
            >
              <template #icon><ArrowRightLeft :size="14" /></template>
              设为当前版本
            </Button>

            <!-- 保留此版本并清理其他版本 -->
            <Tooltip side="top" material="glass">
              <template #trigger>
                <Button
                  size="sm"
                  variant="solid"
                  :disabled="busy"
                  @click="confirmKeepThisAndCleanOthers(item)"
                >
                  <template #icon><ShieldCheck :size="14" /></template>
                  保留并清理旧版
                </Button>
              </template>
              将保留此版本，并彻底删除其余 {{ group.instances.length - 1 }} 个多余版本
            </Tooltip>

            <!-- 打开文件夹 -->
            <Tooltip side="top" material="glass">
              <template #trigger>
                <Button
                  size="sm"
                  variant="ghost"
                  icon-only
                  aria-label="打开所在文件夹"
                  @click="handleOpenFolder(item)"
                >
                  <FolderOpen :size="14" />
                </Button>
              </template>
              打开所在文件夹
            </Tooltip>

            <!-- 单独删除当前版本 -->
            <ConfirmBtn
              v-if="item.source !== 'workshop'"
              :icon="Trash2"
              tip="彻底删除此版本"
              :confirmText="`确认删除版本「v${item.version ?? item.folderName}」？此操作不会影响其他版本。`"
              @confirm="handleDeleteInstance(item)"
            />
          </div>
        </div>
      </div>
    </div>
  </AppDialog>
</template>

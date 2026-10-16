<script setup lang="ts">
import { ref, computed } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NButton, NTag, NIcon, NTooltip, useMessage, useDialog } from "naive-ui"
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
const message = useMessage()
const dialog = useDialog()
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
    message.error(String(e))
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
    message.success(
      t("library.success.uninstalled", { name: `${mod.name} (v${mod.version ?? mod.folderName})` }),
    )
    await modCacheStore.fetchMods()
    if (!modCacheStore.hasConflict(mod.id)) {
      emit("update:show", false)
    }
  } catch (e: unknown) {
    message.error(String(e))
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
    message.success(`已切换生效版本为: v${mod.version ?? mod.folderName}`)
    await modCacheStore.fetchMods()
  } catch (e: unknown) {
    message.error(`版本切换失败: ${String(e)}`)
  } finally {
    busy.value = false
  }
}

/** 保留此版本并清理其他所有旧版本 */
function confirmKeepThisAndCleanOthers(targetMod: InstalledMod) {
  if (!group.value || busy.value) return
  const otherCount = group.value.instances.length - 1
  dialog.warning({
    title: "保留此版本并清理旧版本",
    content: `将保留版本「v${targetMod.version ?? targetMod.folderName}」，并彻底删除其余 ${otherCount} 个多余物理版本。此操作不可撤销，确认继续？`,
    positiveText: "确认清理",
    negativeText: t("common.cancel"),
    onPositiveClick: async () => {
      busy.value = true
      try {
        const cleanedCount = await invoke<number>("keep_mod_version_and_clean_others", {
          installDir: targetMod.installDir,
          modId: targetMod.id,
        })
        message.success(
          `已保留版本 v${targetMod.version ?? targetMod.folderName}，并成功清理 ${cleanedCount} 个旧版本！`,
        )
        await modCacheStore.fetchMods()
        emit("update:show", false)
      } catch (e: unknown) {
        message.error(`清理失败: ${String(e)}`)
      } finally {
        busy.value = false
      }
    },
  })
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
      <div
        class="flex items-center gap-2.5 p-3 rounded-xl text-sm"
        :style="{
          backgroundColor: 'color-mix(in srgb, var(--warning-color, #f59e0b) 12%, transparent)',
          border: '1px solid color-mix(in srgb, var(--warning-color, #f59e0b) 25%, transparent)',
          color: 'var(--color-text-secondary)',
        }"
      >
        <NIcon :size="18" class="text-amber-500 shrink-0"><AlertTriangle /></NIcon>
        <div class="leading-snug">
          检测到系统中共存
          <strong>{{ group.instances.length }}</strong>
          个不同版本的安装实例。请选择需要的版本生效，或清理冗余文件避免冲突。
        </div>
      </div>

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
              <NTag v-if="item.state === 'enabled'" type="success" size="tiny" :bordered="false">
                <template #icon
                  ><NIcon :size="12"><Check /></NIcon
                ></template>
                当前生效
              </NTag>
              <NTag v-else type="default" size="tiny" :bordered="false"> 已禁用 </NTag>
              <NTag v-if="item.source === 'workshop'" type="info" size="tiny" :bordered="false">
                创意工坊
              </NTag>
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
            <NButton
              v-if="item.state !== 'enabled' && item.source !== 'workshop'"
              size="small"
              secondary
              :disabled="busy"
              @click="handleSwitchActive(item)"
            >
              <template #icon
                ><NIcon :size="14"><ArrowRightLeft /></NIcon
              ></template>
              设为当前版本
            </NButton>

            <!-- 保留此版本并清理其他版本 -->
            <NTooltip trigger="hover" placement="top">
              <template #trigger>
                <NButton
                  size="small"
                  type="primary"
                  secondary
                  :disabled="busy"
                  @click="confirmKeepThisAndCleanOthers(item)"
                >
                  <template #icon
                    ><NIcon :size="14"><ShieldCheck /></NIcon
                  ></template>
                  保留并清理旧版
                </NButton>
              </template>
              将保留此版本，并彻底删除其余 {{ group.instances.length - 1 }} 个多余版本
            </NTooltip>

            <!-- 打开文件夹 -->
            <NTooltip trigger="hover" placement="top">
              <template #trigger>
                <NButton size="small" quaternary @click="handleOpenFolder(item)">
                  <template #icon
                    ><NIcon :size="14"><FolderOpen /></NIcon
                  ></template>
                </NButton>
              </template>
              打开所在文件夹
            </NTooltip>

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

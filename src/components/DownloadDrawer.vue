<script setup lang="ts">
import { ref } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import {
  NDrawer, NDrawerContent, NTabs, NTabPane, NButton,
  NIcon, NProgress, NTag, NSpace, useMessage,
} from "naive-ui"
import {
  Download, Pause, Play, X, Trash2, FolderOpen,
  CheckCircle2, AlertCircle, PackageCheck, Clock,
} from "@lucide/vue"
import { useDownloadStore } from "@/stores/useDownloadStore"
import EmptyState from "@/components/EmptyState.vue"

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

const { t } = useI18n()
const message = useMessage()
const downloadStore = useDownloadStore()
const { activeDownloads, completedDownloads } = storeToRefs(downloadStore)
const activeTab = ref<"active" | "completed">("active")

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B"
  const k = 1024
  const sizes = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

function calcPercent(downloaded: number, total: number): number {
  if (total <= 0) return 0
  const pct = Math.floor((downloaded / total) * 100)
  return Math.min(100, Math.max(0, pct))
}

async function handlePause(taskId: string) {
  try {
    await downloadStore.pauseDownload(taskId)
  } catch (e: unknown) {
    message.error(`${t("download.error.pauseFailed")}: ${String(e)}`)
  }
}

async function handleResume(taskId: string) {
  try {
    await downloadStore.resumeDownload(taskId)
  } catch (e: unknown) {
    message.error(`${t("download.error.resumeFailed")}: ${String(e)}`)
  }
}

async function handleCancel(taskId: string) {
  try {
    await downloadStore.cancelDownload(taskId)
    message.info(t("download.cancelled"))
  } catch (e: unknown) {
    message.error(`${t("download.error.cancelFailed")}: ${String(e)}`)
  }
}

async function handleClearCompleted() {
  try {
    await downloadStore.clearCompleted()
    message.success(t("download.clearedCompleted"))
  } catch (e: unknown) {
    message.error(String(e))
  }
}

async function handleOpenFolder() {
  try {
    await downloadStore.openDownloadsFolder()
  } catch (e: unknown) {
    message.error(String(e))
  }
}
</script>

<template>
  <NDrawer
    :show="show"
    :width="460"
    placement="right"
    @update:show="(v: boolean) => emit('update:show', v)"
  >
    <NDrawerContent class="glass-drawer" closable>
      <template #header>
        <div class="flex items-center justify-between w-full pr-4">
          <div class="flex items-center gap-2">
            <NIcon :size="20" class="text-primary-theme"><Download /></NIcon>
            <span class="font-bold text-base text-c-primary">{{ t("download.title") }}</span>
          </div>
          <NSpace :size="6">
            <NButton
              size="tiny"
              secondary
              :title="t('download.openFolder')"
              @click="handleOpenFolder"
            >
              <template #icon><NIcon :size="13"><FolderOpen /></NIcon></template>
              {{ t("download.openFolder") }}
            </NButton>
            <NButton
              v-if="completedDownloads.length > 0"
              size="tiny"
              secondary
              :title="t('download.clearCompleted')"
              @click="handleClearCompleted"
            >
              <template #icon><NIcon :size="13"><Trash2 /></NIcon></template>
              {{ t("download.clearCompleted") }}
            </NButton>
          </NSpace>
        </div>
      </template>

      <NTabs v-model:value="activeTab" type="line" animated>
        <!-- 正在下载 -->
        <NTabPane name="active">
          <template #tab>
            <div class="flex items-center gap-1.5">
              <span>{{ t("download.tabs.active") }}</span>
              <NTag v-if="activeDownloads.length > 0" type="primary" size="tiny" round :bordered="false">
                {{ activeDownloads.length }}
              </NTag>
            </div>
          </template>

          <div v-if="activeDownloads.length > 0" class="flex flex-col gap-3 pt-2">
            <div
              v-for="task in activeDownloads"
              :key="task.id"
              class="download-task-card p-3 rounded-xl border border-c-default transition-all"
            >
              <div class="flex items-start justify-between gap-2 mb-1.5">
                <div class="min-w-0 flex-1">
                  <div class="font-semibold text-sm text-c-primary truncate">{{ task.modName }}</div>
                  <div class="text-xs text-c-muted truncate font-mono">{{ task.fileName }}</div>
                </div>
                <NSpace :size="4" class="flex-shrink-0">
                  <NButton
                    v-if="task.status === 'downloading'"
                    size="tiny"
                    secondary
                    @click="handlePause(task.id)"
                  >
                    <template #icon><NIcon :size="12"><Pause /></NIcon></template>
                  </NButton>
                  <NButton
                    v-else-if="task.status === 'paused'"
                    size="tiny"
                    secondary
                    type="primary"
                    @click="handleResume(task.id)"
                  >
                    <template #icon><NIcon :size="12"><Play /></NIcon></template>
                  </NButton>
                  <NButton
                    size="tiny"
                    secondary
                    @click="handleCancel(task.id)"
                  >
                    <template #icon><NIcon :size="12"><X /></NIcon></template>
                  </NButton>
                </NSpace>
              </div>

              <!-- 进度条 -->
              <NProgress
                type="line"
                :percentage="calcPercent(task.downloadedBytes, task.totalBytes)"
                :status="task.status === 'paused' ? 'warning' : 'default'"
                :show-indicator="false"
                :height="6"
                class="mb-1.5"
              />

              <div class="flex items-center justify-between text-xs text-c-muted font-mono">
                <span>
                  {{ formatBytes(task.downloadedBytes) }} / {{ task.totalBytes > 0 ? formatBytes(task.totalBytes) : '...' }}
                </span>
                <span v-if="task.status === 'downloading'" class="text-primary-theme font-medium">
                  {{ formatBytes(task.speedBytesPerSec) }}/s
                </span>
                <NTag v-else-if="task.status === 'paused'" type="warning" size="tiny" :bordered="false">
                  {{ t("download.status.paused") }}
                </NTag>
              </div>
            </div>
          </div>

          <EmptyState
            v-else
            :icon="Download"
            :title="t('download.empty.noActive')"
            :description="t('download.empty.noActiveHint')"
            bordered
            class="mt-6"
          />
        </NTabPane>

        <!-- 已完成与历史 -->
        <NTabPane name="completed">
          <template #tab>
            <div class="flex items-center gap-1.5">
              <span>{{ t("download.tabs.completed") }}</span>
              <NTag v-if="completedDownloads.length > 0" size="tiny" round :bordered="false">
                {{ completedDownloads.length }}
              </NTag>
            </div>
          </template>

          <div v-if="completedDownloads.length > 0" class="flex flex-col gap-3 pt-2">
            <div
              v-for="task in completedDownloads"
              :key="task.id"
              class="download-task-card p-3 rounded-xl border border-c-default transition-all"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0 flex-1">
                  <div class="font-semibold text-sm text-c-primary truncate">{{ task.modName }}</div>
                  <div class="text-xs text-c-muted truncate font-mono mt-0.5">{{ task.fileName }}</div>
                </div>
                <NTag v-if="task.installed" type="success" size="tiny" :bordered="false" class="flex-shrink-0">
                  <template #icon><NIcon :size="10"><PackageCheck /></NIcon></template>
                  {{ t("download.status.installed") }}
                </NTag>
                <NTag v-else-if="task.installError" type="warning" size="tiny" :bordered="false" class="flex-shrink-0">
                  <template #icon><NIcon :size="10"><AlertCircle /></NIcon></template>
                  {{ t("download.status.notInstalled") }}
                </NTag>
                <NTag v-else type="info" size="tiny" :bordered="false" class="flex-shrink-0">
                  <template #icon><NIcon :size="10"><CheckCircle2 /></NIcon></template>
                  {{ t("download.status.completed") }}
                </NTag>
              </div>

              <div class="flex items-center justify-between text-xs text-c-muted mt-2 pt-1 border-t border-c-subtle">
                <span class="font-mono">{{ formatBytes(task.totalBytes || task.downloadedBytes) }}</span>
                <span v-if="task.completedAt" class="flex items-center gap-1">
                  <NIcon :size="11"><Clock /></NIcon>
                  {{ new Date(task.completedAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
                </span>
              </div>
            </div>
          </div>

          <EmptyState
            v-else
            :icon="PackageCheck"
            :title="t('download.empty.noCompleted')"
            bordered
            class="mt-6"
          />
        </NTabPane>
      </NTabs>
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped>
.glass-drawer {
  background-color: var(--glass-surface-bg) !important;
  backdrop-filter: blur(var(--glass-surface-blur));
  -webkit-backdrop-filter: blur(var(--glass-surface-blur));
}
.download-task-card {
  background-color: var(--glass-surface-bg) !important;
  border-color: var(--glass-surface-border) !important;
}
.download-task-card:hover {
  background-color: var(--glass-surface-bg-hover) !important;
  box-shadow: var(--shadow-glow) !important;
}
</style>

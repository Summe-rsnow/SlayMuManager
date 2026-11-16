<script setup lang="ts">
import { computed, ref } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { Badge, Button, Card, Drawer, EmptyState, Progress, Tabs, useToast } from "@xcmm1543/miuui-vue"
import {
  Download,
  Pause,
  Play,
  X,
  Trash2,
  FolderOpen,
  CheckCircle2,
  AlertCircle,
  PackageCheck,
  Clock,
} from "@lucide/vue"
import { useDownloadStore } from "@/stores/useDownloadStore"

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

const { t } = useI18n()
const { toast } = useToast()
const downloadStore = useDownloadStore()
const { activeDownloads, completedDownloads } = storeToRefs(downloadStore)
const activeTab = ref<"active" | "completed">("active")
const tabItems = computed(() => [
  {
    value: "active",
    label:
      activeDownloads.value.length > 0
        ? `${t("download.tabs.active")} (${activeDownloads.value.length})`
        : t("download.tabs.active"),
  },
  {
    value: "completed",
    label:
      completedDownloads.value.length > 0
        ? `${t("download.tabs.completed")} (${completedDownloads.value.length})`
        : t("download.tabs.completed"),
  },
])
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
    toast.error(`${t("download.error.pauseFailed")}: ${String(e)}`)
  }
}

async function handleResume(taskId: string) {
  try {
    await downloadStore.resumeDownload(taskId)
  } catch (e: unknown) {
    toast.error(`${t("download.error.resumeFailed")}: ${String(e)}`)
  }
}

async function handleCancel(taskId: string) {
  try {
    await downloadStore.cancelDownload(taskId)
    toast.info(t("download.cancelled"))
  } catch (e: unknown) {
    toast.error(`${t("download.error.cancelFailed")}: ${String(e)}`)
  }
}

async function handleClearCompleted() {
  try {
    await downloadStore.clearCompleted()
    toast.success(t("download.clearedCompleted"))
  } catch (e: unknown) {
    toast.error(String(e))
  }
}

async function handleOpenFolder() {
  try {
    await downloadStore.openDownloadsFolder()
  } catch (e: unknown) {
    toast.error(String(e))
  }
}
</script>

<template>
  <Drawer
    :open="show"
    width="460px"
    side="end"
    material="glass"
    :aria-label="t('download.title')"
    @update:open="(v: boolean) => emit('update:show', v)"
  >
    <template #title>
      <div class="flex items-center justify-between w-full pr-4">
        <span class="font-bold text-base text-c-primary">{{ t("download.title") }}</span>
        <div class="flex items-center gap-1.5">
          <Button
            size="xs"
            variant="secondary"
            :title="t('download.openFolder')"
            @click="handleOpenFolder"
          >
            <template #icon><FolderOpen :size="13" /></template>
            {{ t("download.openFolder") }}
          </Button>
          <Button
            v-if="completedDownloads.length > 0"
            size="xs"
            variant="secondary"
            :title="t('download.clearCompleted')"
            @click="handleClearCompleted"
          >
            <template #icon><Trash2 :size="13" /></template>
            {{ t("download.clearCompleted") }}
          </Button>
        </div>
      </div>
    </template>

    <Tabs
      :model-value="activeTab"
      :items="tabItems"
      variant="underline"
      @update:model-value="(v: string) => (activeTab = v as 'active' | 'completed')"
    >
      <template #panel="{ value }">
        <template v-if="value === 'active'">
          <div v-if="activeDownloads.length > 0" class="flex flex-col gap-3 p-3.5">
            <Card
              v-for="task in activeDownloads"
              :key="task.id"
              material="glass"
              interactive
              padding="sm"
            >
              <div class="flex items-start justify-between gap-2 mb-1.5">
                <div class="min-w-0 flex-1">
                  <div class="font-semibold text-sm text-c-primary truncate">
                    {{ task.modName }}
                  </div>
                  <div class="text-xs text-c-muted truncate font-mono">{{ task.fileName }}</div>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <Button
                    v-if="task.status === 'downloading'"
                    size="xs"
                    variant="secondary"
                    icon-only
                    :aria-label="t('download.status.downloading')"
                    @click="handlePause(task.id)"
                  >
                    <Pause :size="12" />
                  </Button>
                  <Button
                    v-else-if="task.status === 'paused'"
                    size="xs"
                    variant="solid"
                    icon-only
                    :aria-label="t('download.status.paused')"
                    @click="handleResume(task.id)"
                  >
                    <Play :size="12" />
                  </Button>
                  <Button
                    size="xs"
                    variant="secondary"
                    icon-only
                    :aria-label="t('common.cancel')"
                    @click="handleCancel(task.id)"
                  >
                    <X :size="12" />
                  </Button>
                </div>
              </div>

              <!-- 进度条 -->
              <Progress
                :model-value="calcPercent(task.downloadedBytes, task.totalBytes)"
                :variant="task.status === 'paused' ? 'warning' : 'primary'"
                size="sm"
                :show-value="false"
                class="mb-1.5"
              />
              <div class="flex items-center justify-between text-xs text-c-muted font-mono">
                <span>
                  {{ formatBytes(task.downloadedBytes) }} /
                  {{ task.totalBytes > 0 ? formatBytes(task.totalBytes) : "..." }}
                </span>
                <span v-if="task.status === 'downloading'" class="text-primary-theme font-medium">
                  {{ formatBytes(task.speedBytesPerSec) }}/s
                </span>
                <Badge v-else-if="task.status === 'paused'" variant="warning" size="xs">
                  {{ t("download.status.paused") }}
                </Badge>
              </div>
            </Card>
          </div>

          <Card v-else material="glass" padding="sm" class="mt-6">
            <EmptyState
              :title="t('download.empty.noActive')"
              :description="t('download.empty.noActiveHint')"
            >
              <template #icon><Download :size="36" /></template>
            </EmptyState>
          </Card>
        </template>

        <template v-else-if="value === 'completed'">
          <div v-if="completedDownloads.length > 0" class="flex flex-col gap-3 p-3.5">
            <Card
              v-for="task in completedDownloads"
              :key="task.id"
              material="glass"
              interactive
              padding="sm"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0 flex-1">
                  <div class="font-semibold text-sm text-c-primary truncate">
                    {{ task.modName }}
                  </div>
                  <div class="text-xs text-c-muted truncate font-mono mt-0.5">
                    {{ task.fileName }}
                  </div>
                </div>
                <Badge v-if="task.installed" variant="success" size="xs" class="flex-shrink-0">
                  <PackageCheck :size="10" />
                  {{ t("download.status.installed") }}
                </Badge>
                <Badge
                  v-else-if="task.installError"
                  variant="warning"
                  size="xs"
                  class="flex-shrink-0"
                >
                  <AlertCircle :size="10" />
                  {{ t("download.status.notInstalled") }}
                </Badge>
                <Badge v-else variant="info" size="xs" class="flex-shrink-0">
                  <CheckCircle2 :size="10" />
                  {{ t("download.status.completed") }}
                </Badge>
              </div>

              <div
                class="flex items-center justify-between text-xs text-c-muted mt-2 pt-1 border-t border-c-subtle"
              >
                <span class="font-mono">{{
                  formatBytes(task.totalBytes || task.downloadedBytes)
                }}</span>
                <span v-if="task.completedAt" class="flex items-center gap-1">
                  <Clock :size="11" />
                  new Date(task.completedAt).toLocaleTimeString([], { hour: "2-digit", minute:
                  "2-digit", }) }}
                </span>
              </div>
            </Card>
          </div>

          <Card v-else material="glass" padding="sm" class="mt-6">
            <EmptyState :title="t('download.empty.noCompleted')">
              <template #icon><PackageCheck :size="36" /></template>
            </EmptyState>
          </Card>
        </template>
      </template>
    </Tabs>
  </Drawer>
</template>

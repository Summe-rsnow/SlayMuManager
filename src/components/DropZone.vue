<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { useI18n } from "vue-i18n"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import { NIcon } from "naive-ui"
import { Upload } from "lucide-vue-next"

const { t } = useI18n()

const props = defineProps<{
  busy?: boolean
}>()

const emit = defineEmits<{
  (e: "filesDropped", paths: string[]): void
}>()

// --- Tauri 原生拖放（获取真实文件系统路径）---
let tauriUnlisten: (() => void) | null = null

// 防抖：400ms 内连续拖放合并为一次导入（对齐 SlaySP2Manager）
let debounceTimer: ReturnType<typeof setTimeout> | null = null
const accumulatedPaths: string[] = []
const DROP_DEBOUNCE_MS = 400

/// 过滤支持的导入格式：.zip / .7z / 文件夹（无扩展名）
function isSupportedImport(path: string): boolean {
  const lower = path.toLowerCase()
  // 文件：仅接受 .zip / .7z
  if (lower.endsWith(".zip") || lower.endsWith(".7z")) return true
  // 文件夹：无扩展名视为目录
  const base = path.split(/[\\/]/).pop() ?? ""
  if (!base.includes(".")) return true
  return false
}

async function setupTauriDragDrop() {
  const webview = getCurrentWebview()
  tauriUnlisten = await webview.onDragDropEvent((event) => {
    if (event.payload.type === "drop") {
      // isBusy 防护
      if (props.busy) return

      const paths = event.payload.paths
      if (paths.length === 0) return

      // 过滤支持格式
      const importable = paths.filter(isSupportedImport)
      if (importable.length === 0) return

      // 防抖合并
      accumulatedPaths.push(...importable)
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        const unique = [...new Set(accumulatedPaths)]
        accumulatedPaths.length = 0
        emit("filesDropped", unique)
      }, DROP_DEBOUNCE_MS)
    }
  })
}

onMounted(() => {
  setupTauriDragDrop()
})

onUnmounted(() => {
  tauriUnlisten?.()
  if (debounceTimer) clearTimeout(debounceTimer)
})

// --- HTML5 事件（仅用于 DropZone 区域的视觉反馈）---
const isDragging = ref(false)
let dragCounter = 0

function onDragEnter(e: DragEvent) {
  e.preventDefault()
  dragCounter++
  isDragging.value = true
}

function onDragLeave(e: DragEvent) {
  e.preventDefault()
  dragCounter--
  if (dragCounter === 0) {
    isDragging.value = false
  }
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  // 不在此提取路径 —— Tauri onDragDropEvent 会提供真实文件系统路径
  isDragging.value = false
  dragCounter = 0
}
</script>

<template>
  <div
    class="relative border-2 border-dashed rounded-lg transition-all duration-200"
    :class="{
      'border-green-400 bg-green-50 scale-[1.02] cursor-copy': isDragging && !busy,
      'border-c-default bg-c-secondary cursor-pointer': !isDragging && !busy,
      'border-c-default bg-c-secondary cursor-not-allowed opacity-60': busy,
    }"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div
      class="flex flex-col items-center justify-center py-6 gap-2 text-c-muted"
    >
      <NIcon :size="28" :color="isDragging && !busy ? '#18a058' : undefined">
        <Upload />
      </NIcon>
      <p class="text-sm" :class="{ 'text-green-600 font-medium': isDragging && !busy }">
        {{ isDragging && !busy ? t("import.dropZoneDropHint") : t("import.dropZoneHint") }}
      </p>
      <p class="text-xs text-c-muted">
        {{ t("import.dropZoneOrUseButton") }}
      </p>
    </div>
  </div>
</template>

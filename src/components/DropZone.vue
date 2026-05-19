<script setup lang="ts">
import { ref } from "vue"
import { useI18n } from "vue-i18n"
import { NIcon } from "naive-ui"
import { Upload } from "lucide-vue-next"

const { t } = useI18n()

const props = defineProps<{
  busy?: boolean
}>()

const emit = defineEmits<{
  (e: "filesDropped", paths: string[]): void
}>()

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

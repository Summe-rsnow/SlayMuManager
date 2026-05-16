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
  filesDropped: [paths: string[]]
}>()

const isDragging = ref(false)
let dragCounter = 0

// 防抖：400ms 内连续拖放合并为一次导入（对齐 SlaySP2Manager）
let debounceTimer: ReturnType<typeof setTimeout> | null = null
const accumulatedPaths: string[] = []
const DROP_DEBOUNCE_MS = 400

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
  isDragging.value = false
  dragCounter = 0

  // isBusy 防护：安装/扫描中忽略新的拖放
  if (props.busy) return

  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return

  const paths: string[] = []
  for (let i = 0; i < files.length; i++) {
    const file = files[i]
    // @ts-expect-error Tauri drag event provides path
    const path: string = file.path ?? (file as any).webkitRelativePath ?? file.name
    if (path) paths.push(path)
  }

  if (paths.length > 0) {
    // 防抖合并：快速连续拖放合并累积
    accumulatedPaths.push(...paths)
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      const unique = [...new Set(accumulatedPaths)]
      accumulatedPaths.length = 0
      emit("filesDropped", unique)
    }, DROP_DEBOUNCE_MS)
  }
}
</script>

<template>
  <div
    class="relative border-2 border-dashed rounded-lg transition-all duration-200"
    :class="{
      'border-green-400 bg-green-50 scale-[1.02] cursor-copy': isDragging && !busy,
      'border-gray-200 bg-gray-50/50 hover:border-gray-300 hover:bg-gray-50 cursor-pointer': !isDragging && !busy,
      'border-gray-100 bg-gray-50/30 cursor-not-allowed opacity-60': busy,
    }"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div class="flex flex-col items-center justify-center py-6 gap-2" :class="{ 'text-gray-300': busy, 'text-gray-400': !busy }">
      <NIcon :size="28" :color="isDragging ? '#18a058' : undefined">
        <Upload />
      </NIcon>
      <p class="text-sm" :class="{ 'text-green-600 font-medium': isDragging && !busy }">
        {{ isDragging ? t("import.dropZoneDropHint") : t("import.dropZoneHint") }}
      </p>
      <p class="text-xs text-gray-300">
        {{ t("import.dropZoneOrUseButton") }}
      </p>
    </div>
  </div>
</template>

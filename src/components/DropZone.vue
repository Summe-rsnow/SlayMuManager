<script setup lang="ts">
import { ref } from "vue"
import { useI18n } from "vue-i18n"
import { NIcon } from "naive-ui"
import { Upload } from "lucide-vue-next"

const { t } = useI18n()

const emit = defineEmits<{
  filesDropped: [paths: string[]]
}>()

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
  isDragging.value = false
  dragCounter = 0

  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return

  const paths: string[] = []
  for (let i = 0; i < files.length; i++) {
    // Tauri 2.0 中，拖放文件可通过 webkitGetAsEntry 或直接读 path 属性
    const file = files[i]
    // @ts-expect-error Tauri drag event provides path
    const path: string = file.path ?? (file as any).webkitRelativePath ?? file.name
    if (path) paths.push(path)
  }

  if (paths.length > 0) {
    emit("filesDropped", paths)
  }
}
</script>

<template>
  <div
    class="relative border-2 border-dashed rounded-lg transition-all duration-200 cursor-pointer"
    :class="
      isDragging
        ? 'border-green-400 bg-green-50 scale-[1.02]'
        : 'border-gray-200 bg-gray-50/50 hover:border-gray-300 hover:bg-gray-50'
    "
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div class="flex flex-col items-center justify-center py-6 gap-2 text-gray-400">
      <NIcon :size="28" :color="isDragging ? '#18a058' : undefined">
        <Upload />
      </NIcon>
      <p class="text-sm" :class="{ 'text-green-600 font-medium': isDragging }">
        {{ isDragging ? t("import.dropZoneDropHint") : t("import.dropZoneHint") }}
      </p>
      <p class="text-xs text-gray-300">
        {{ t("import.dropZoneOrUseButton") }}
      </p>
    </div>
  </div>
</template>

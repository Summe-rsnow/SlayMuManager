<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { useI18n } from "vue-i18n"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import { NIcon } from "naive-ui"
import { Upload } from "@lucide/vue"

const { t } = useI18n()

const props = defineProps<{
  /** 拖放提示主标题（默认：松开以导入文件） */
  title?: string
  /** 拖放提示副标题（默认：拖放 .zip / .7z / .rar 文件或文件夹到此处） */
  subtitle?: string
  /** 接受的文件扩展名（小写，不含点），如 ['zip', '7z', 'rar']。默认接受所有文件。 */
  acceptExt?: string[]
  /** 是否接受文件夹（无扩展名路径），默认 true */
  acceptFolders?: boolean
}>()

const showOverlay = ref(false)
let tauriUnlisten: (() => void) | null = null

function isAcceptable(path: string): boolean {
  const lower = path.toLowerCase()
  // 扩展名过滤
  if (props.acceptExt && props.acceptExt.length > 0) {
    if (props.acceptExt.some(ext => lower.endsWith(`.${ext}`))) return true
    // 文件夹
    if (props.acceptFolders !== false) {
      const base = path.split(/[\\/]/).pop() ?? ""
      if (!base.includes(".")) return true
    }
    return false
  }
  return true
}

async function setupTauriDrag() {
  const webview = getCurrentWebview()
  tauriUnlisten = await webview.onDragDropEvent((event) => {
    const { type } = event.payload
    if (type === "enter") {
      const paths = event.payload.paths
      if (paths.length === 0 || !paths.some(isAcceptable)) return
      showOverlay.value = true
    } else if (type === "leave" || type === "drop") {
      showOverlay.value = false
    }
  })
}

onMounted(() => {
  setupTauriDrag()
})

onUnmounted(() => {
  tauriUnlisten?.()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="drag-fade">
      <div
        v-if="showOverlay"
        class="fixed inset-0 z-[9999] flex items-center justify-center select-none"
        :style="{
          background: 'radial-gradient(ellipse 80% 70% at 50% 45%, color-mix(in srgb, var(--primary-color) 10%, transparent) 0%, transparent 70%)',
          backdropFilter: 'blur(var(--blur-backdrop)) saturate(var(--blur-saturate))',
        }"
      >
        <div class="flex flex-col items-center gap-8">
          <div
            class="w-36 h-36 rounded-full flex items-center justify-center"
            :style="{ backgroundColor: 'color-mix(in srgb, var(--primary-color) 15%, transparent)' }"
          >
            <NIcon :size="60" :color="'var(--primary-color)'">
              <Upload />
            </NIcon>
          </div>
          <p class="text-3xl font-semibold text-c-primary">
            {{ title ?? t("import.dropZoneDropHint") }}
          </p>
          <p class="text-lg text-c-secondary">
            {{ subtitle ?? t("import.dropZoneHint") }}
          </p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.drag-fade-enter-active {
  transition: opacity 0.18s ease-out;
}
.drag-fade-leave-active {
  transition: opacity 0.15s ease-in;
}
.drag-fade-enter-from,
.drag-fade-leave-to {
  opacity: 0;
}
</style>

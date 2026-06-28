<script setup lang="ts">
import { ref } from "vue"
import { NIcon, NModal } from "naive-ui"
import { PackageOpen } from "@lucide/vue"

const props = defineProps<{
  src: string | null | undefined
  alt: string
}>()

const loadFailed = ref(false)
const showPreview = ref(false)

function onError() {
  loadFailed.value = true
}

function openPreview() {
  if (props.src) showPreview.value = true
}
</script>

<template>
  <div
    class="w-28 h-28 rounded-lg flex-shrink-0 overflow-hidden bg-c-secondary"
    :class="{ 'cursor-pointer': !!src }"
  >
    <img
      v-if="src && !loadFailed"
      :src="src"
      :alt="alt"
      class="w-full h-full object-cover"
      referrerpolicy="no-referrer"
      @error="onError"
      @click="openPreview"
    />
    <div v-else class="w-full h-full flex items-center justify-center">
      <NIcon :size="32" :color="'var(--color-text-muted)'"><PackageOpen /></NIcon>
    </div>
  </div>

  <NModal
    :show="showPreview"
    :style="{
      backdropFilter: 'blur(var(--blur-backdrop)) saturate(1.3)',
      WebkitBackdropFilter: 'blur(var(--blur-backdrop)) saturate(1.3)',
      background: 'var(--blur-backdrop-bg)',
    }"
    @update:show="(v: boolean) => !v && (showPreview = false)"
  >
    <div
      class="flex items-center justify-center"
      style="max-width: 90vw; max-height: 90vh;"
      @click="showPreview = false"
    >
      <img
        v-if="src"
        :src="src"
        class="max-w-full max-h-[85vh] rounded-lg shadow-2xl object-contain"
        style="max-width: 85vw;"
      />
    </div>
  </NModal>
</template>

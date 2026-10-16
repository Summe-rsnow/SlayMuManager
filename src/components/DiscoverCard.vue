<script setup lang="ts">
import { computed, type Component } from "vue"
import { NIcon } from "naive-ui"
import ModPreviewImage from "./ModPreviewImage.vue"
import FloatingTip from "./FloatingTip.vue"
import ModTranslateBlock from "./ModTranslateBlock.vue"

defineProps<{
  name: string
  imageUrl: string | null | undefined
  description: string | null | undefined
  author: string
  version?: string | null
  stats: Array<{ icon: Component; value: string | number }>
}>()

const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  borderColor: "var(--glass-surface-border)",
  boxShadow: "var(--shadow-glass-surface)",
  transform: "translateZ(0)",
}))
</script>

<template>
  <div
    class="discover-card group rounded-xl border transition-all duration-200 hover:-translate-y-0.5 p-4 flex flex-col min-w-0"
    :style="cardStyle"
  >
    <div class="flex gap-4 flex-1 min-w-0">
      <ModPreviewImage :src="imageUrl" :alt="name" class="flex-shrink-0" />
      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="font-semibold text-base text-c-primary truncate">{{ name }}</span>
            <span v-if="version" class="text-xs font-mono flex-shrink-0 text-c-muted"
              >v{{ version }}</span
            >
          </div>
          <slot name="actions" />
        </div>
        <div class="min-h-0 mt-2">
          <FloatingTip :text="description ?? ''" truncated :max-lines="6" />
          <ModTranslateBlock :text="description" />
        </div>
        <div class="flex items-center gap-3 text-xs pt-2 mt-auto text-c-muted">
          <span class="truncate max-w-[120px]">{{ author }}</span>
          <span v-for="(stat, i) in stats" :key="i" class="flex items-center gap-1 flex-shrink-0">
            <NIcon :size="13"><component :is="stat.icon" /></NIcon>
            {{ stat.value }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.discover-card {
  transform: translateZ(0);
}
.discover-card:hover {
  background-color: var(--glass-surface-bg-hover) !important;
  box-shadow: var(--shadow-card-hover), var(--shadow-glow) !important;
  border-color: color-mix(
    in srgb,
    var(--primary-color) 35%,
    var(--glass-surface-border)
  ) !important;
}
</style>

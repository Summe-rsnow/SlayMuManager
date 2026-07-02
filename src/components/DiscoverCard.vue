<script setup lang="ts">
import type { Component } from "vue"
import { NCard, NIcon } from "naive-ui"
import ModPreviewImage from "./ModPreviewImage.vue"
import TruncatedText from "./TruncatedText.vue"
import ModTranslateBlock from "./ModTranslateBlock.vue"

defineProps<{
  name: string
  imageUrl: string | null | undefined
  description: string | null | undefined
  author: string
  version?: string | null
  stats: Array<{ icon: Component; value: string | number }>
}>()
</script>

<template>
  <NCard class="discover-card break-inside-avoid mb-4 transition-all duration-200 hover:-translate-y-0.5" :style="{ minHeight: '150px' }">
    <div class="flex gap-4 h-full">
      <ModPreviewImage :src="imageUrl" :alt="name" />
      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="font-semibold text-base text-c-primary truncate">{{ name }}</span>
            <span v-if="version" class="text-xs font-mono flex-shrink-0 text-c-muted">v{{ version }}</span>
          </div>
          <slot name="actions" />
        </div>
        <div class="min-h-0 mt-2">
          <TruncatedText :text="description" />
          <ModTranslateBlock :text="description" />
        </div>
        <div class="flex items-center gap-3 text-xs pt-2 mt-auto text-c-muted">
          <span>{{ author }}</span>
          <span v-for="(stat, i) in stats" :key="i" class="flex items-center gap-1">
            <NIcon :size="13"><component :is="stat.icon" /></NIcon>
            {{ stat.value }}
          </span>
        </div>
      </div>
    </div>
  </NCard>
</template>

<style scoped>
.discover-card {
  --n-border-color: color-mix(in srgb, var(--color-border), var(--color-text-muted) 50%);
}
.discover-card:hover {
  box-shadow: var(--shadow-glow) !important;
  border-color: color-mix(in srgb, var(--primary-color) 25%, var(--color-border)) !important;
}
</style>

<script setup lang="ts">
import { NCard, NSpace } from "naive-ui"

defineProps<{
  title: string
  id?: string
  highlighted?: boolean
}>()
</script>

<template>
  <NCard :id="id" :title="title" size="small" :class="{ 'setting-highlight': highlighted }">
    <NSpace vertical>
      <slot />
    </NSpace>
  </NCard>
</template>

<style scoped>
.setting-highlight {
  animation: highlight-flash 2.5s ease-in-out;
}
@keyframes highlight-flash {
  0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 50%, transparent); }
  50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--primary-color) 30%, transparent); }
  100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 0%, transparent); }
}
</style>

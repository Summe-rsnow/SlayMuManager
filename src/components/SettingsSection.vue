<script setup lang="ts">
import { Card } from "@miuui/vue"
defineProps<{
  title: string
  id?: string
  highlighted?: boolean
}>()
</script>

<template>
  <Card
    :id="id"
    :title="title"
    padding="sm"
    material="glass"
    class="settings-section glass-surface"
    :class="{ 'setting-highlight': highlighted }"
  >
    <div class="flex flex-col gap-3">
      <slot />
    </div>
  </Card>
</template>

<style scoped>
.settings-section {
  background-color: var(--glass-surface-bg);
}
.setting-highlight {
  animation: highlight-flash 2.5s ease-in-out;
}
@keyframes highlight-flash {
  0% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 50%, transparent);
  }
  50% {
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--primary-color) 30%, transparent);
  }
  100% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 0%, transparent);
  }
}
</style>

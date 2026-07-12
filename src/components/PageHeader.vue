<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-center justify-between pb-3 mb-5 gap-4 border-b"
    :style="{ borderColor: 'var(--color-border)' }">
    <div class="flex-shrink-0">
      <h1 class="text-3xl font-bold tracking-tight text-c-primary">{{ title }}</h1>
      <p v-if="subtitle" class="text-xs text-c-muted mt-0.5">{{ subtitle }}</p>
    </div>
    <div v-if="$slots.default"
      class="flex items-center gap-2.5 min-w-0 overflow-auto flex-wrap justify-end">
      <slot />
    </div>
  </div>
</template>

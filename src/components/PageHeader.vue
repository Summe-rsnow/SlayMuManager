<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-center justify-between mb-6 gap-4">
    <div class="flex-shrink-0">
      <h1 class="text-2xl font-bold text-c-primary whitespace-nowrap">{{ title }}</h1>
      <p v-if="subtitle" class="text-sm mt-1 text-c-secondary whitespace-nowrap">{{ subtitle }}</p>
    </div>
    <div v-if="$slots.default" class="flex items-center gap-4 min-w-0 overflow-auto">
      <slot />
    </div>
  </div>
</template>

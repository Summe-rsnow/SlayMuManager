<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-2xl font-bold text-c-primary">{{ title }}</h1>
      <p v-if="subtitle" class="text-sm mt-1 text-c-secondary">{{ subtitle }}</p>
    </div>
    <div v-if="$slots.default" class="flex items-center gap-4">
      <slot />
    </div>
  </div>
</template>

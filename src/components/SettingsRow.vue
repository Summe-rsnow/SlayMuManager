<script setup lang="ts">
defineProps<{
  label?: string
}>()
</script>

<template>
  <div class="flex items-center justify-between min-h-[32px]">
    <slot name="label">
      <span class="text-sm text-c-primary">{{ label }}</span>
    </slot>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { NTooltip } from "naive-ui"

defineProps<{
  text: string | null | undefined
}>()
</script>

<template>
  <div v-if="text" class="truncated-wrapper">
    <NTooltip trigger="hover" placement="top">
      <template #trigger>
        <p class="text-xs text-c-muted truncated-content">{{ text }}</p>
      </template>
      {{ text }}
    </NTooltip>
  </div>
  <div v-else style="min-height: 1.125rem" />
</template>

<style scoped>
.truncated-content {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
  max-height: 7.5rem;
  line-height: 1.25;
}
</style>

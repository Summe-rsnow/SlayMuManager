<script setup lang="ts">
import { NPopover } from "naive-ui"

defineProps<{
  text: string | null | undefined
}>()
</script>

<template>
  <NPopover v-if="text" trigger="hover" placement="top" :width="320">
    <template #trigger>
      <p class="text-xs text-gray-400 line-clamp-2 cursor-help">
        {{ text }}
      </p>
    </template>
    <div class="text-xs leading-relaxed">{{ text }}</div>
  </NPopover>
  <div v-else style="min-height: 1.125rem" />
</template>

<script setup lang="ts">
import { NIcon, NPopover } from "naive-ui"
import { HelpCircle } from "@lucide/vue"

withDefaults(defineProps<{
  text: string
  width?: number
  label?: string
  placement?: string
  iconSize?: number
}>(), {
  placement: "right",
  iconSize: 13,
})
</script>

<template>
  <NPopover trigger="hover" :placement="placement as any" :width="width ?? 260">
    <template #trigger>
      <span v-if="label" class="flex items-center gap-1 cursor-help text-sm text-c-primary" :style="{ lineHeight: '1' }">
        <span>{{ label }}</span>
        <NIcon :size="iconSize" class="text-c-muted"><HelpCircle /></NIcon>
      </span>
      <span v-else class="inline-flex items-center ml-1">
        <NIcon :size="iconSize" class="text-c-muted cursor-help"><HelpCircle /></NIcon>
      </span>
    </template>
    <span class="text-xs">{{ text }}</span>
  </NPopover>
</template>

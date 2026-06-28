<script setup lang="ts">
import { NIcon, NPopover } from "naive-ui"
import { HelpCircle } from "@lucide/vue"

defineProps<{
  text: string
  width?: number
}>()
</script>

<template>
  <NPopover trigger="hover" placement="bottom" :width="width ?? 260">
    <template #trigger>
      <span class="inline-flex items-center ml-1">
        <NIcon :size="13" class="text-c-muted cursor-help"><HelpCircle /></NIcon>
      </span>
    </template>
    <span class="text-xs">{{ text }}</span>
  </NPopover>
</template>

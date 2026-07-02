<script setup lang="ts">
import { NButton, NIcon, NTooltip } from "naive-ui"

defineProps<{
  icon: any
  tip: string
  disabled?: boolean
  type?: "default" | "primary" | "info" | "success" | "warning" | "error"
}>()
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <NTooltip trigger="hover" placement="top">
    <template #trigger>
      <slot name="trigger" :icon="icon" :disabled="disabled" :type="type">
        <NButton text size="tiny" :disabled="disabled" :type="type ?? 'default'" class="transition-transform duration-150 active:scale-90" @click="emit('click')">
          <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
        </NButton>
      </slot>
    </template>
    {{ tip }}
  </NTooltip>
</template>

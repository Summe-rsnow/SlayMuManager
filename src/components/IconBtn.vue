<script setup lang="ts">
import { NButton, NIcon } from "naive-ui"
import FloatingTip from "./FloatingTip.vue"

defineProps<{
  icon: any
  tip: string
  disabled?: boolean
  type?: "default" | "primary" | "info" | "success" | "warning" | "error"
}>()
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <FloatingTip :text="tip" mode="tooltip">
    <slot name="trigger" :icon="icon" :disabled="disabled" :type="type">
      <NButton text size="tiny" :disabled="disabled" :type="type ?? 'default'" class="transition-transform duration-150 active:scale-90" @click="emit('click')">
        <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
      </NButton>
    </slot>
  </FloatingTip>
</template>

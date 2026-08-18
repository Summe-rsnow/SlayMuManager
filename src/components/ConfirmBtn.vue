<script setup lang="ts">
import type { Component } from "vue"
import { NButton, NIcon, NPopconfirm } from "naive-ui"
import FloatingTip from "./FloatingTip.vue"

defineProps<{
  icon: Component
  tip?: string
  confirmText: string
  disabled?: boolean
  type?: string
  size?: string
}>()

const emit = defineEmits<{
  (e: "confirm"): void
}>()
</script>

<template>
  <FloatingTip v-if="tip" :text="tip" mode="tooltip">
    <NPopconfirm @positive-click="emit('confirm')">
      <template #trigger>
        <NButton text :size="(size ?? 'tiny') as any" :type="(type as any) ?? 'error'" :disabled="disabled" class="transition-transform duration-150 active:scale-90">
          <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
        </NButton>
      </template>
      <slot name="confirm-content">{{ confirmText }}</slot>
    </NPopconfirm>
  </FloatingTip>
  <NPopconfirm v-else @positive-click="emit('confirm')">
    <template #trigger>
      <NButton text :size="(size ?? 'tiny') as any" :type="(type as any) ?? 'error'" :disabled="disabled" class="transition-transform duration-150 active:scale-90">
        <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
      </NButton>
    </template>
    <slot name="confirm-content">{{ confirmText }}</slot>
  </NPopconfirm>
</template>

<script setup lang="ts">
import type { Component } from "vue"
import { NButton, NIcon, NPopconfirm, NTooltip } from "naive-ui"

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
  <NTooltip v-if="tip" trigger="hover" placement="top">
    <template #trigger>
      <NPopconfirm @positive-click="emit('confirm')">
        <template #trigger>
          <NButton text :size="(size ?? 'tiny') as any" :type="(type as any) ?? 'error'" :disabled="disabled" class="transition-transform duration-150 active:scale-90">
            <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
          </NButton>
        </template>
        <slot name="confirm-content">{{ confirmText }}</slot>
      </NPopconfirm>
    </template>
    {{ tip }}
  </NTooltip>
  <NPopconfirm v-else @positive-click="emit('confirm')">
    <template #trigger>
      <NButton text :size="(size ?? 'tiny') as any" :type="(type as any) ?? 'error'" :disabled="disabled" class="transition-transform duration-150 active:scale-90">
        <template #icon><NIcon :size="14"><component :is="icon" /></NIcon></template>
      </NButton>
    </template>
    <slot name="confirm-content">{{ confirmText }}</slot>
  </NPopconfirm>
</template>

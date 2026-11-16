<script setup lang="ts">
import type { Component } from "vue"
import { Button, Popconfirm } from "@xcmm1543/miuui-vue"
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
  <FloatingTip v-if="tip" :text="tip">
    <Popconfirm
      :title="confirmText"
      :danger="!type || type === 'error'"
      material="glass"
      @confirm="emit('confirm')"
    >
      <template #trigger>
        <Button
          variant="ghost"
          size="xs"
          icon-only
          :aria-label="tip"
          :disabled="disabled"
          class="text-red-500 hover:text-red-600 transition-transform duration-150 active:scale-90"
        >
          <component :is="icon" :size="14" />
        </Button>
      </template>
    </Popconfirm>
  </FloatingTip>
  <Popconfirm
    v-else
    :title="confirmText"
    :danger="!type || type === 'error'"
    material="glass"
    @confirm="emit('confirm')"
  >
    <template #trigger>
      <Button
        variant="ghost"
        size="xs"
        icon-only
        :aria-label="confirmText"
        :disabled="disabled"
        class="text-red-500 hover:text-red-600 transition-transform duration-150 active:scale-90"
      >
        <component :is="icon" :size="14" />
      </Button>
    </template>
  </Popconfirm>
</template>

<script setup lang="ts">
import type { Component } from "vue"
import { Button } from "@miuui/vue"
import FloatingTip from "./FloatingTip.vue"

defineProps<{
  icon: Component
  tip: string
  disabled?: boolean
  type?: "default" | "primary" | "info" | "success" | "warning" | "error"
}>()
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <FloatingTip :text="tip">
    <slot name="trigger" :icon="icon" :disabled="disabled" :type="type">
      <Button
        variant="ghost"
        size="xs"
        icon-only
        :aria-label="tip"
        :disabled="disabled"
        :class="[
          'transition-transform duration-150 active:scale-90',
          type === 'error'
            ? 'text-red-500 hover:text-red-600'
            : type === 'primary'
              ? 'text-primary-theme'
              : '',
        ]"
        @click="emit('click')"
      >
        <component :is="icon" :size="14" />
      </Button>
    </slot>
  </FloatingTip>
</template>

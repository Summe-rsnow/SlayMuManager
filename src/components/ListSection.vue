<script setup lang="ts">
import { NCard, NButton } from "naive-ui"

defineProps<{
  title: string
  actionLabel?: string
  actionBusy?: boolean
}>()

const emit = defineEmits<{
  (e: "action"): void
}>()
</script>

<template>
  <NCard size="small" class="mb-4">
    <template #header>
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span>{{ title }}</span>
        </div>
        <div class="flex items-center gap-2">
          <slot name="header-extra" />
          <NButton v-if="actionLabel" size="small" secondary :loading="actionBusy" @click="emit('action')">{{ actionLabel }}</NButton>
        </div>
      </div>
    </template>
    <slot />
    <template v-if="!$slots.default || !$slots.default()">
      <slot name="empty" />
    </template>
  </NCard>
</template>

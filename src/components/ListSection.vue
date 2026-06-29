<script setup lang="ts">
import { NCard, NTag, NButton } from "naive-ui"

defineProps<{
  title: string
  count: number
  countType?: string
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
          <NTag :type="(count > 0 ? (countType as any) : 'default') ?? 'default'" size="small" round>{{ count }}</NTag>
        </div>
        <div class="flex items-center gap-2">
          <slot name="header-extra" />
          <NButton v-if="actionLabel" size="small" secondary :loading="actionBusy" @click="emit('action')">{{ actionLabel }}</NButton>
        </div>
      </div>
    </template>
    <slot v-if="count > 0" />
    <slot v-else name="empty" />
  </NCard>
</template>

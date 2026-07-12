<script setup lang="ts">
defineProps<{
  title: string
  actionLabel?: string
  actionBusy?: boolean
  count?: number
}>()
const emit = defineEmits<{ (e: "action"): void }>()
</script>

<template>
  <div class="mb-6">
    <div class="flex items-center justify-between pb-2 mb-3">
      <div class="flex items-center gap-2.5">
        <h2 class="text-base font-semibold text-c-primary">{{ title }}</h2>
        <span
          v-if="count !== undefined"
          class="text-xs text-c-muted bg-c-secondary px-2 py-0.5 rounded-full"
        >
          {{ count }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <slot name="header-extra" />
        <button
          v-if="actionLabel"
          class="text-xs font-medium px-3 py-1.5 rounded-lg transition-all duration-200 cursor-pointer outline-none border-0"
          :style="{
            color: 'var(--primary-color)',
            backgroundColor: 'color-mix(in srgb, var(--primary-color) 8%, transparent)',
          }"
          :disabled="actionBusy"
          @click="emit('action')"
        >
          {{ actionLabel }}
        </button>
      </div>
    </div>
    <slot />
  </div>
</template>

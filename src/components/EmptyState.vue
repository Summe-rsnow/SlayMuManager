<script setup lang="ts">
import type { Component } from "vue"
import { NCard, NIcon, NButton } from "naive-ui"

defineProps<{
  icon: Component
  title: string
  description?: string
  bordered?: boolean
  size?: "sm" | "md" | "lg"
  actionText?: string
  actionType?: string
}>()

const emit = defineEmits<{
  (e: "action"): void
}>()
</script>

<template>
  <NCard v-if="bordered" size="small">
    <div class="text-center" :class="size === 'sm' ? 'py-8' : size === 'lg' ? 'py-16' : 'py-12'">
      <NIcon :size="size === 'sm' ? 32 : 48" class="mb-3" :color="'var(--color-text-muted)'">
        <component :is="icon" />
      </NIcon>
      <p class="text-c-muted">{{ title }}</p>
      <p v-if="description" class="text-sm mt-1 text-c-muted">{{ description }}</p>
      <NButton v-if="actionText" size="tiny" :type="(actionType as any) ?? 'primary'" class="mt-3" @click="emit('action')">{{ actionText }}</NButton>
    </div>
  </NCard>
  <div v-else class="text-center" :class="size === 'sm' ? 'py-8' : size === 'lg' ? 'py-16' : 'py-12'">
    <NIcon :size="size === 'sm' ? 32 : 48" class="mb-3" :color="'var(--color-text-muted)'">
      <component :is="icon" />
    </NIcon>
    <p class="text-c-muted">{{ title }}</p>
    <p v-if="description" class="text-sm mt-1 text-c-muted">{{ description }}</p>
    <NButton v-if="actionText" size="tiny" :type="(actionType as any) ?? 'primary'" class="mt-3" @click="emit('action')">{{ actionText }}</NButton>
  </div>
</template>

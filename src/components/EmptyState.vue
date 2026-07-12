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
  <NCard v-if="bordered" size="small" :style="{ boxShadow: 'var(--shadow-card)' }">
    <div class="text-center mx-auto max-w-sm" :class="size === 'sm' ? 'py-8' : size === 'lg' ? 'py-16' : 'py-12'">
      <div class="animate-float">
        <NIcon :size="size === 'sm' ? 32 : 48" class="mb-3" :color="'var(--primary-color)'" :style="{ opacity: 0.5 }">
          <component :is="icon" />
        </NIcon>
      </div>
      <p class="text-c-muted">{{ title }}</p>
      <p v-if="description" class="text-sm mt-1 text-c-muted">{{ description }}</p>
      <NButton v-if="actionText" size="tiny" :type="(actionType as any) ?? 'primary'" class="mt-3 transition-all duration-200 hover:-translate-y-0.5" @click="emit('action')">{{ actionText }}</NButton>
    </div>
  </NCard>
  <div v-else class="text-center mx-auto max-w-sm" :class="size === 'sm' ? 'py-8' : size === 'lg' ? 'py-16' : 'py-12'">
    <div class="animate-float">
      <NIcon :size="size === 'sm' ? 32 : 48" class="mb-3" :color="'var(--primary-color)'" :style="{ opacity: 0.5 }">
        <component :is="icon" />
      </NIcon>
    </div>
    <p class="text-c-muted">{{ title }}</p>
    <p v-if="description" class="text-sm mt-1 text-c-muted">{{ description }}</p>
    <NButton v-if="actionText" size="tiny" :type="(actionType as any) ?? 'primary'" class="mt-3 transition-all duration-200 hover:-translate-y-0.5" @click="emit('action')">{{ actionText }}</NButton>
  </div>
</template>

<style scoped>
@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}
.animate-float {
  animation: float 3s ease-in-out infinite;
}
</style>

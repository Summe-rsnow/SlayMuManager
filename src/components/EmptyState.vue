<script setup lang="ts">
import type { Component } from "vue"
import { NCard, NIcon } from "naive-ui"

defineProps<{
  /** Lucide 图标组件 */
  icon: Component
  /** 主标题 */
  title: string
  /** 副标题（可选） */
  description?: string
  /** 是否包含 NCard 边框 */
  bordered?: boolean
  /** 间距大小：sm=py-8, md=py-12(默认), lg=py-16 */
  size?: "sm" | "md" | "lg"
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
    </div>
  </NCard>
  <div v-else class="text-center" :class="size === 'sm' ? 'py-8' : size === 'lg' ? 'py-16' : 'py-12'">
    <NIcon :size="size === 'sm' ? 32 : 48" class="mb-3" :color="'var(--color-text-muted)'">
      <component :is="icon" />
    </NIcon>
    <p class="text-c-muted">{{ title }}</p>
    <p v-if="description" class="text-sm mt-1 text-c-muted">{{ description }}</p>
  </div>
</template>

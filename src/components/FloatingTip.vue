<script setup lang="ts">
import { useSlots } from "vue"
import { NIcon, NTooltip } from "naive-ui"
import { HelpCircle } from "@lucide/vue"

const props = withDefaults(defineProps<{
  text: string
  width?: number
  label?: string
  placement?: string
  iconSize?: number
  truncated?: boolean
  maxLines?: number
}>(), {
  placement: "top",
  iconSize: 13,
  truncated: false,
  maxLines: 6,
})

const slots = useSlots()
</script>

<template>
  <!-- 1. 自定义触发元素模式（包裹任意按钮、开关等） -->
  <NTooltip
    v-if="slots.default"
    trigger="hover"
    :placement="placement as any"
    :style="width ? { maxWidth: `${width}px` } : undefined"
  >
    <template #trigger>
      <span class="inline-flex items-center">
        <slot />
      </span>
    </template>
    <span>{{ text }}</span>
  </NTooltip>

  <!-- 2. 截断文本模式（用于发现页/预设描述文本 hover 查看全文） -->
  <div v-else-if="truncated" class="truncated-wrapper">
    <NTooltip
      trigger="hover"
      :placement="placement as any"
      :style="width ? { maxWidth: `${width}px` } : undefined"
    >
      <template #trigger>
        <p class="text-xs text-c-muted truncated-content cursor-help" :style="{ '-webkit-line-clamp': maxLines }">{{ text }}</p>
      </template>
      <span>{{ text }}</span>
    </NTooltip>
  </div>

  <!-- 3. 带标签或独立问号提示模式 -->
  <NTooltip
    v-else
    trigger="hover"
    :placement="placement as any"
    :style="width ? { maxWidth: `${width}px` } : undefined"
  >
    <template #trigger>
      <span v-if="label" class="floating-tip-label">
        <span>{{ label }}</span>
        <NIcon :size="iconSize" class="text-c-muted"><HelpCircle /></NIcon>
      </span>
      <span v-else class="floating-tip-icon">
        <NIcon :size="iconSize" class="text-c-muted cursor-help"><HelpCircle /></NIcon>
      </span>
    </template>
    <span>{{ text }}</span>
  </NTooltip>
</template>

<style scoped>
.truncated-content {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  overflow: hidden;
  max-height: 7.5rem;
  line-height: 1.25;
}

.floating-tip-label {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: help;
  line-height: 1;
}

.floating-tip-icon {
  display: inline-flex;
  align-items: center;
  margin-left: 4px;
}
</style>

<script setup lang="ts">
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
  placement: "right",
  iconSize: 13,
  truncated: false,
  maxLines: 6,
})
</script>

<template>
  <!-- 截断文本模式（用于描述文本） -->
  <div v-if="truncated" class="truncated-wrapper">
    <NTooltip trigger="hover" placement="top">
      <template #trigger>
        <p class="text-xs text-c-muted truncated-content" :style="{ '-webkit-line-clamp': maxLines }">{{ text }}</p>
      </template>
      {{ text }}
    </NTooltip>
  </div>

  <!-- 带标签的提示模式 -->
  <NTooltip v-else trigger="hover" :placement="placement as any">
    <template #trigger>
      <span v-if="label" class="floating-tip-label">
        <span>{{ label }}</span>
        <NIcon :size="iconSize" class="text-c-muted"><HelpCircle /></NIcon>
      </span>
      <span v-else class="floating-tip-icon">
        <NIcon :size="iconSize" class="text-c-muted cursor-help"><HelpCircle /></NIcon>
      </span>
    </template>
    <span class="text-xs">{{ text }}</span>
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

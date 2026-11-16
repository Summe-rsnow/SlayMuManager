<script setup lang="ts">
import { computed, useSlots } from "vue"
import { Tooltip, TruncateText } from "@xcmm1543/miuui-vue"
import { HelpCircle } from "@lucide/vue"

const props = withDefaults(
  defineProps<{
    text: string
    width?: number
    label?: string
    placement?: string
    iconSize?: number
    truncated?: boolean
    maxLines?: number
    to?: string | boolean | HTMLElement
  }>(),
  {
    placement: "top",
    iconSize: 13,
    truncated: false,
    maxLines: 6,
    to: "body",
  },
)

const slots = useSlots()
const side = computed(() => {
  const base = props.placement?.split("-")[0]
  if (base === "top" || base === "right" || base === "bottom" || base === "left") return base
  return "top"
})
</script>

<template>
  <!-- 1. 自定义触发元素模式（包裹任意按钮、开关等） -->
  <Tooltip
    v-if="slots.default"
    :side="side"
    material="glass"
    :max-width="width ? `${width}px` : '420px'"
    max-height="280px"
  >
    <template #trigger>
      <span class="inline-flex items-center">
        <slot />
      </span>
    </template>
    <span class="whitespace-pre-wrap break-words">{{ text }}</span>
  </Tooltip>

  <!-- 2. 截断文本模式（用于发现页/预设描述文本 hover 查看全文） -->
  <div v-else-if="truncated" class="truncated-wrapper">
    <Tooltip
      :side="side"
      material="glass"
      :max-width="width ? `${width}px` : '420px'"
      max-height="280px"
    >
      <template #trigger>
        <TruncateText
          as="p"
          :text="text"
          :lines="maxLines"
          :title="false"
          class="text-xs text-c-muted cursor-help"
        />
      </template>
      <span class="whitespace-pre-wrap break-words">{{ text }}</span>
    </Tooltip>
  </div>

  <!-- 3. 带标签或独立问号提示模式 -->
  <Tooltip
    v-else
    :side="side"
    material="glass"
    :max-width="width ? `${width}px` : '420px'"
    max-height="280px"
  >
    <template #trigger>
      <span v-if="label" class="floating-tip-label">
        <span>{{ label }}</span>
        <HelpCircle :size="iconSize" class="text-c-muted" />
      </span>
      <span v-else class="floating-tip-icon" @click.stop>
        <HelpCircle :size="iconSize" class="text-c-muted cursor-help" />
      </span>
    </template>
    <span class="whitespace-pre-wrap break-words">{{ text }}</span>
  </Tooltip>
</template>

<style scoped>
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

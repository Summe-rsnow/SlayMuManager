<script setup lang="ts">
import { NCard, NModal } from "naive-ui"

defineProps<{
  show: boolean
  /** 标题（提供 header slot 时忽略） */
  title?: string
  /** 对话框宽度, 默认 520px */
  width?: string | number
  /** 是否允许点击遮罩关闭, 默认 true */
  maskClosable?: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

function onUpdateShow(v: boolean) {
  if (!v) emit("update:show", false)
}
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="maskClosable ?? true"
    @update:show="onUpdateShow"
  >
    <NCard
      :style="{
        width: typeof width === 'number' ? width + 'px' : width || '520px',
        maxHeight: '85vh',
      }"
      :bordered="false"
      role="dialog"
    >
      <template #header>
        <slot name="header">
          <span class="text-lg font-semibold">{{ title }}</span>
        </slot>
      </template>
      <slot />
      <template v-if="$slots.footer" #footer>
        <slot name="footer" />
      </template>
    </NCard>
  </NModal>
</template>

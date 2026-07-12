<script setup lang="ts">
import { NCard, NModal, NButton, NSpace } from "naive-ui"
import { useI18n } from "vue-i18n"

const { t } = useI18n()

const props = withDefaults(defineProps<{
  show: boolean
  title?: string
  width?: string | number
  maskClosable?: boolean
  confirmText?: string
  cancelText?: string
  confirmLoading?: boolean
  confirmType?: string
  showFooter?: boolean
}>(), {
  maskClosable: true,
})

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
  (e: "confirm"): void
  (e: "cancel"): void
}>()

function onUpdateShow(v: boolean) {
  if (!v) emit("update:show", false)
}

function onCancel() {
  emit("cancel")
  emit("update:show", false)
}
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="maskClosable"
    @update:show="onUpdateShow"
  >
    <NCard
      :style="{
        width: typeof width === 'number' ? width + 'px' : width || '520px',
        maxHeight: '85vh',
        boxShadow: 'var(--shadow-modal)',
      }"
      :bordered="false"
      role="dialog"
    >
      <template #header>
        <slot name="header">
          <span class="text-lg font-semibold">{{ title }}</span>
        </slot>
      </template>
      <div class="dialog-content">
        <div class="dialog-glow" />
        <slot />
      </div>
      <template v-if="$slots.footer" #footer>
        <slot name="footer" />
      </template>
      <template v-else-if="showFooter" #footer>
        <NSpace justify="end">
          <NButton @click="onCancel">{{ cancelText || t("common.cancel") }}</NButton>
          <NButton :type="(confirmType as any) ?? 'primary'" :loading="confirmLoading" @click="emit('confirm')">{{ confirmText }}</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style>
.n-modal-mask {
  backdrop-filter: blur(var(--blur-backdrop)) saturate(var(--blur-saturate));
  background: transparent;
  transition: backdrop-filter 0.25s ease;
}

.n-modal-mask.fade-in-transition-leave-active {
  backdrop-filter: blur(0px) saturate(var(--blur-saturate));
}
</style>

<style scoped>
.dialog-content {
  position: relative;
}
.dialog-content :deep(.n-card__content) {
  position: relative;
  z-index: 1;
}
.dialog-glow {
  display: none;
}
:root.dark .dialog-glow {
  display: block;
  position: absolute;
  top: -12px;
  left: 50%;
  transform: translateX(-50%);
  width: 160px;
  height: 80px;
  background: radial-gradient(ellipse at center, var(--primary-color) 0%, transparent 70%);
  opacity: 0.08;
  pointer-events: none;
  z-index: 0;
}
:root.dark .dialog-content {
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border-radius: inherit;
}
</style>

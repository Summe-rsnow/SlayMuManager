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

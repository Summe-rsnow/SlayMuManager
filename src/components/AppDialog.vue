<script setup lang="ts">
import { computed } from "vue"
import { Dialog, Button } from "@miuui/vue"
import { useI18n } from "vue-i18n"

const { t } = useI18n()

const props = withDefaults(
  defineProps<{
    show: boolean
    title?: string
    width?: string | number
    maskClosable?: boolean
    confirmText?: string
    cancelText?: string
    confirmLoading?: boolean
    confirmType?: string
    showFooter?: boolean
  }>(),
  {
    maskClosable: true,
  },
)

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
  (e: "confirm"): void
  (e: "cancel"): void
}>()

const resolvedWidth = computed(() =>
  typeof props.width === "number" ? `${props.width}px` : props.width || "520px",
)

function onUpdateOpen(v: boolean) {
  if (!v) {
    if (!props.maskClosable) return
    emit("update:show", false)
  }
}

function onCancel() {
  emit("cancel")
  emit("update:show", false)
}
</script>

<template>
  <Dialog
    :open="show"
    :width="resolvedWidth"
    :hide-header="!title && !$slots.header"
    material="glass"
    :aria-label="title || 'Dialog'"
    :styles="{ content: { maxHeight: '85vh', overflowY: 'auto' } }"
    @update:open="onUpdateOpen"
  >
    <template v-if="title || $slots.header" #title>
      <slot name="header">
        <span class="text-lg font-semibold">{{ title }}</span>
      </slot>
    </template>
    <slot />
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
    <template v-else-if="showFooter" #footer>
      <div class="flex items-center justify-end gap-2">
        <Button variant="ghost" size="sm" @click="onCancel">{{
          cancelText || t("common.cancel")
        }}</Button>
        <Button
          :variant="confirmType === 'error' ? 'danger' : 'solid'"
          size="sm"
          :loading="confirmLoading"
          @click="emit('confirm')"
        >
          {{ confirmText || t("common.confirm") }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>

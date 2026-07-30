<script setup lang="ts">
import { computed } from "vue"
import { NIcon, NSelect, NPagination, NInputNumber } from "naive-ui"
import { List } from "@lucide/vue"

const props = withDefaults(defineProps<{
  page: number
  pageSize: number
  totalCount: number
  showPageSize?: boolean
  pageSizeOptions?: Array<{ label: string; value: number }>
}>(), {
  showPageSize: true,
})

const emit = defineEmits<{
  "update:page": [value: number]
  "update:pageSize": [value: number]
}>()

const totalPages = computed(() => Math.ceil(props.totalCount / props.pageSize))

function onPageInput(val: number | null) {
  if (val == null) return
  const clamped = Math.max(1, Math.min(Math.round(val), totalPages.value))
  if (clamped !== props.page) {
    emit("update:page", clamped)
  }
}
</script>

<template>
  <div class="discover-pagination-bar flex items-center gap-3 px-5 py-2.5 rounded-full"
    :style="{
      backgroundColor: 'var(--glass-bg)',
      backdropFilter: 'blur(var(--glass-blur))',
      WebkitBackdropFilter: 'blur(var(--glass-blur))',
      border: 'var(--glass-border)',
      boxShadow: 'var(--shadow-glass)',
    }"
  >
    <template v-if="showPageSize && pageSizeOptions?.length">
      <div class="flex items-center gap-1.5">
        <NIcon :size="14" class="text-c-muted"><List /></NIcon>
        <NSelect
          :value="pageSize"
          :options="pageSizeOptions"
          style="width: 80px"
          size="tiny"
          @update:value="emit('update:pageSize', $event)"
        />
      </div>
    </template>
    <div class="discover-pagination">
      <NPagination
        :page="page"
        :page-size="pageSize"
        :item-count="totalCount"
        @update:page="emit('update:page', $event)"
        size="small"
      />
    </div>
    <NInputNumber
      :value="page"
      size="tiny"
      :min="1"
      :max="totalPages"
      style="width: 70px"
      @update:value="onPageInput"
    />
  </div>
</template>

<style scoped>
.discover-pagination :deep(.n-pagination-item) {
  border-radius: 9999px !important;
}
.discover-pagination :deep(.n-pagination-item.n-pagination-item--active) {
  border-radius: 9999px !important;
}
</style>

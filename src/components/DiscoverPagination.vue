<script setup lang="ts">
import { ref, computed } from "vue"
import { useI18n } from "vue-i18n"
import { NIcon, NSelect, NPagination, NInputNumber, NButton } from "naive-ui"
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

const { t } = useI18n()

const totalPages = computed(() => Math.ceil(props.totalCount / props.pageSize))

const jumpPage = ref<number | null>(null)

function jumpToPage() {
  const p = jumpPage.value
  if (p == null || p < 1 || p > totalPages.value) return
  emit("update:page", p)
  jumpPage.value = null
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
    <span class="text-xs text-c-muted">{{ t("discover.jumpTo") }}</span>
    <NInputNumber
      v-model:value="jumpPage"
      size="tiny"
      :min="1"
      :max="totalPages"
      :placeholder="String(page)"
      style="width: 70px"
      @keyup.enter="jumpToPage"
    />
    <NButton size="tiny" secondary @click="jumpToPage">{{ t("discover.jumpToBtn") }}</NButton>
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

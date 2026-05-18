<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { NTag, NButton, NIcon, NSpace, NPopconfirm } from "naive-ui"
import { Clock, Upload, Database, Trash2, ShieldAlert } from "lucide-vue-next"
import { currentLocale } from "../i18n"
import type { SaveSlot } from "../types"
import { kindLabel } from "../utils/kindLabel"

const props = defineProps<{
  slot: SaveSlot
}>()

const emit = defineEmits<{
  (e: "backup", slot: SaveSlot): void
  (e: "migrate", slot: SaveSlot): void
  (e: "delete", slot: SaveSlot): void
}>()

const { t } = useI18n()

</script>

<template>
  <div
    :class="slot.hasData
      ? 'p-3 rounded-xl border border-c-default bg-c-secondary'
      : 'p-2 rounded-xl border border-dashed border-c-default bg-c-secondary'"
  >
    <div :class="slot.hasData ? 'flex items-center justify-between mb-2' : 'flex items-center justify-between'">
      <span class="font-medium text-sm text-c-primary">
        {{ t("saves.slotIndex", { i: slot.slotIndex }) }}
      </span>
      <NSpace :size="4">
        <NTag v-if="slot.hasCurrentRun" type="warning" size="tiny" :bordered="false">
          {{ t("saves.currentRun") }}
        </NTag>
        <NTag
          :type="slot.hasData ? 'success' : 'default'"
          size="tiny"
          :bordered="false"
        >
          {{ slot.hasData ? t("saves.fileCount", { n: slot.fileCount }) : t("saves.empty.empty") }}
        </NTag>
      </NSpace>
    </div>

    <div v-if="slot.hasData" class="text-xs text-c-muted mb-2">
      <NIcon :size="12"><Clock /></NIcon>
      {{ slot.lastModifiedAt ? new Date(slot.lastModifiedAt).toLocaleString(currentLocale) : t("common.unknown") }}
    </div>

    <!-- 操作按钮行 -->
    <div v-if="slot.hasData" class="flex items-center gap-1 flex-wrap">
      <NButton size="tiny" secondary @click="emit('backup', slot)">
        <template #icon><NIcon :size="12"><Database /></NIcon></template>
        {{ t("saves.backup") }}
      </NButton>
      <NButton size="tiny" secondary @click="emit('migrate', slot)">
        <template #icon><NIcon :size="12"><Upload /></NIcon></template>
        {{ t("saves.migrate") }}
      </NButton>
      <NPopconfirm
        @positive-click="() => emit('delete', slot)"
      >
        <template #trigger>
          <NButton size="tiny" type="error" secondary>
            <template #icon><NIcon :size="12"><Trash2 /></NIcon></template>
            {{ t("saves.deleteSlot") }}
          </NButton>
        </template>
        <div class="max-w-64">
          <div class="flex items-center gap-2 mb-1">
            <NIcon :size="16" color="#d03050"><ShieldAlert /></NIcon>
            <span class="font-medium">{{ t("saves.confirmDeleteSlotTitle") }}</span>
          </div>
          <p class="text-xs text-c-secondary">
            {{ t("saves.confirmDeleteSlotDesc", { kind: kindLabel(t, slot.kind), i: slot.slotIndex }) }}
          </p>
          <p class="text-xs text-c-warning mt-1">
            {{ t("saves.confirmDeleteSlotNote") }}
          </p>
        </div>
      </NPopconfirm>
    </div>
  </div>
</template>

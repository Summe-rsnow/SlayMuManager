<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { Badge, Button, Card, Popconfirm } from "@miuui/vue"
import { Clock, Upload, Database, Trash2 } from "@lucide/vue"
import { currentLocale } from "@/i18n"
import type { SaveSlot } from "../types"
import { kindLabel } from "@/utils/kindLabel"

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
  <Card
    :variant="slot.hasData ? 'surface' : 'outline'"
    :material="slot.hasData ? 'glass' : undefined"
    :interactive="slot.hasData"
    padding="sm"
    :class="slot.hasData ? '' : 'border-dashed opacity-85'"
  >
    <div
      :class="
        slot.hasData
          ? 'flex items-center justify-between mb-2'
          : 'flex items-center justify-between'
      "
    >
      <span class="font-medium text-sm text-c-primary">
        {{ t("saves.slotIndex", { i: slot.slotIndex }) }}
      </span>
      <div class="flex items-center gap-1">
        <Badge v-if="slot.hasCurrentRun" variant="warning" size="xs">
          {{ t("saves.currentRun") }}
        </Badge>
        <Badge :variant="slot.hasData ? 'success' : 'neutral'" size="xs">
          {{ slot.hasData ? t("saves.fileCount", { n: slot.fileCount }) : t("saves.empty.empty") }}
        </Badge>
      </div>
    </div>

    <div v-if="slot.hasData" class="flex items-center gap-1 text-xs text-c-muted mb-2">
      <Clock :size="12" />
      {{
        slot.lastModifiedAt
          ? new Date(slot.lastModifiedAt).toLocaleString(currentLocale)
          : t("common.unknown")
      }}
    </div>

    <!-- 操作按钮行 -->
    <div v-if="slot.hasData" class="flex items-center gap-1 flex-wrap">
      <Button size="xs" variant="secondary" @click="emit('backup', slot)">
        <template #icon><Database :size="12" /></template>
        {{ t("saves.backup") }}
      </Button>
      <Button size="xs" variant="secondary" @click="emit('migrate', slot)">
        <template #icon><Upload :size="12" /></template>
        {{ t("saves.migrate") }}
      </Button>
      <Popconfirm
        :title="t('saves.confirmDeleteSlotTitle')"
        :description="`${t('saves.confirmDeleteSlotDesc', { kind: kindLabel(t, slot.kind), i: slot.slotIndex })} ${t('saves.confirmDeleteSlotNote')}`"
        danger
        material="glass"
        @confirm="emit('delete', slot)"
      >
        <template #trigger>
          <Button size="xs" variant="danger">
            <template #icon><Trash2 :size="12" /></template>
            {{ t("saves.deleteSlot") }}
          </Button>
        </template>
      </Popconfirm>
    </div>
  </Card>
</template>

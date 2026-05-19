<script setup lang="ts">
import { ref, computed } from "vue"
import { currentLocale } from "../i18n"
import { useI18n } from "vue-i18n"
import {
  NTag, NButton, NIcon, NSwitch, NPopconfirm, NPopover, NCheckbox, NSpace, NInput,
} from "naive-ui"
import { FolderOpen, Trash2, Plus, StickyNote } from "lucide-vue-next"
import { useModTags, PRESET_TAGS } from "../composables/useModTags"
import { useModNotes } from "../composables/useModNotes"
import type { InstalledMod } from "../types"

import type { ModUpdateInfo } from "../types"

const props = defineProps<{
  mod: InstalledMod
  enabled: boolean
  busy: boolean
  toggleDisabled: boolean
  hasUpdate?: boolean
  updateInfo?: ModUpdateInfo | null
}>()

const emit = defineEmits<{
  (e: "toggle", mod: InstalledMod): void
  (e: "openFolder", mod: InstalledMod): void
  (e: "uninstall", mod: InstalledMod): void
}>()

const { t } = useI18n()
const { getTags, toggleTag, getTagLabel, isPresetTag } = useModTags()
const { getNote, setNote, hasNote } = useModNotes()

const noteDraft = ref("")
function openNotePopover(modId: string) {
  noteDraft.value = getNote(modId)
}
function saveNote(modId: string) {
  setNote(modId, noteDraft.value)
}

// --- 动态卡片样式（暗色模式适配）---
const cardStyle = computed(() => ({
  backgroundColor: "var(--color-bg-secondary)",
  borderColor: "var(--color-border)",
  borderLeftColor: props.enabled ? "var(--primary-color)" : "var(--color-text-muted)",
}))
</script>

<template>
  <div
    class="group flex items-center justify-between p-3 rounded-lg border border-l-3 transition-shadow hover:shadow-sm"
    :style="cardStyle"
    :class="[{ 'pointer-events-none opacity-60': busy }]"
  >
    <div class="flex-1 min-w-0" style="position:relative;z-index:2">
      <!-- 名称 + 版本 + 标签 -->
      <div class="flex items-center gap-1.5 flex-wrap">
        <span class="font-medium truncate max-w-[180px] text-c-primary">
          {{ mod.name }}
        </span>
        <span class="text-xs text-c-muted font-mono truncate">{{ mod.version ?? "—" }}</span>
        <NTag v-if="hasUpdate && updateInfo?.remoteVersion" type="warning" size="tiny" :bordered="false" :title="'v' + updateInfo!.remoteVersion + ' 可用'">
          ↑ {{ updateInfo!.remoteVersion }}
        </NTag>
        <NTag v-if="mod.affectsGameplay" type="warning" size="tiny" :bordered="false">
          {{ t("library.mod.affectsGameplay") }}
        </NTag>
        <NTag
          v-for="tagId in getTags(mod.id)"
          :key="tagId"
          size="tiny"
          :bordered="false"
          :type="isPresetTag(tagId) ? 'info' : 'default'"
          closable
          @close="() => toggleTag(mod.id, tagId)"
        >
          {{ getTagLabel(tagId, currentLocale) }}
        </NTag>
      </div>

      <!-- 作者 + 文件夹 -->
      <div class="text-xs text-c-muted mt-0.5">
        {{ mod.author ?? t("library.mod.unknownAuthor") }} · {{ mod.folderName }}
      </div>

      <!-- 备注内容（有备注时显示为淡色一行） -->
      <div v-if="hasNote(mod.id)" class="mt-0.5 text-xs text-c-muted truncate">
        <NIcon :size="12" class="inline-block mr-0.5 align-middle"><StickyNote /></NIcon>
        <span class="align-middle">{{ getNote(mod.id) }}</span>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4">
      <!-- 添加标签 -->
      <NPopover trigger="click" placement="bottom-end">
        <template #trigger>
          <NButton text size="tiny" :disabled="busy" :aria-label="t('library.mod.selectTag')">
            <template #icon><NIcon :size="14"><Plus /></NIcon></template>
          </NButton>
        </template>
        <div class="w-52">
          <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.selectTag") }}</div>
          <NSpace vertical :size="4">
            <NCheckbox
              v-for="t in PRESET_TAGS"
              :key="t.id"
              size="small"
              :checked="getTags(mod.id).includes(t.id)"
              @update:checked="() => toggleTag(mod.id, t.id)"
            >
              <span class="text-xs">{{ getTagLabel(t.id, currentLocale) }}</span>
            </NCheckbox>
          </NSpace>
        </div>
      </NPopover>

      <!-- 备注 -->
      <NPopover trigger="click" placement="bottom" @update:show="(v: boolean) => v && openNotePopover(mod.id)">
        <template #trigger>
          <NButton text size="tiny" :disabled="busy">
            <template #icon><NIcon :size="14"><StickyNote /></NIcon></template>
          </NButton>
        </template>
        <div class="w-56">
          <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.note") }}</div>
          <NInput
            :value="noteDraft"
            type="textarea"
            size="small"
            :placeholder="t('library.mod.notePlaceholder')"
            :autosize="{ minRows: 2, maxRows: 6 }"
            @update:value="(v: string) => noteDraft = v"
            @blur="saveNote(mod.id)"
          />
        </div>
      </NPopover>

      <NButton text size="tiny" :disabled="busy" :aria-label="t('library.mod.openFolder')" @click="emit('openFolder', mod)">
        <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
      </NButton>
      <NPopconfirm @positive-click="() => emit('uninstall', mod)">
        <template #trigger>
          <NButton text size="tiny" type="error" :disabled="busy" :aria-label="t('library.mod.uninstall')">
            <template #icon><NIcon :size="14"><Trash2 /></NIcon></template>
          </NButton>
        </template>
        {{ t("library.mod.confirmUninstall", { name: mod.name }) }}
      </NPopconfirm>
      <NSwitch
        :value="enabled"
        :disabled="busy || toggleDisabled"
        @update:value="() => emit('toggle', mod)"
      />
    </div>
  </div>
</template>

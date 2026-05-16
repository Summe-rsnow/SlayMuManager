<script setup lang="ts">
import { ref } from "vue"
import { useI18n } from "vue-i18n"
import {
  NTag, NButton, NIcon, NSwitch, NPopconfirm, NPopover, NCheckbox, NSpace, NInput,
} from "naive-ui"
import { FolderOpen, Trash2, Plus, StickyNote } from "lucide-vue-next"
import { useModTags, PRESET_TAGS } from "../composables/useModTags"
import { useModNotes } from "../composables/useModNotes"
import type { InstalledMod } from "../types"
import "../assets/library-effects.css"

const props = defineProps<{
  mod: InstalledMod
  enabled: boolean
  busy: boolean
  toggleDisabled: boolean
}>()

const emit = defineEmits<{
  toggle: [mod: InstalledMod]
  openFolder: [mod: InstalledMod]
  uninstall: [mod: InstalledMod]
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
</script>

<template>
  <div
    class="mod-card group flex items-center justify-between p-3 rounded-lg border transition-colors"
    :class="[
      enabled
        ? 'mod-card--enabled border-gray-100 bg-white'
        : 'mod-card--disabled border-gray-200 bg-gray-50/60',
      { 'pointer-events-none opacity-60': busy },
    ]"
  >
    <div class="flex-1 min-w-0" style="position:relative;z-index:2">
      <!-- 名称 + 版本 + 标签 -->
      <div class="flex items-center gap-2">
        <span class="font-medium truncate" :class="enabled ? 'text-gray-800' : 'text-gray-600'">
          {{ mod.name }}
        </span>
        <span class="text-xs text-gray-400 font-mono truncate">{{ mod.version ?? "—" }}</span>
        <NTag v-if="mod.affectsGameplay" type="warning" size="tiny" :bordered="false">
          {{ t("library.mod.affectsGameplay") }}
        </NTag>
      </div>

      <!-- 作者 + 文件夹 -->
      <div class="text-xs text-gray-400 mt-0.5">
        {{ mod.author ?? t("library.mod.unknownAuthor") }} · {{ mod.folderName }}
      </div>

      <!-- 标签行 -->
      <div class="flex items-center gap-1 mt-1 flex-wrap">
        <NTag
          v-for="tagId in getTags(mod.id)"
          :key="tagId"
          size="tiny"
          :bordered="false"
          :type="isPresetTag(tagId) ? 'info' : 'default'"
          closable
          @close="() => toggleTag(mod.id, tagId)"
        >
          {{ getTagLabel(tagId) }}
        </NTag>

        <!-- 添加标签 -->
        <NPopover trigger="click" placement="bottom-start">
          <template #trigger>
            <NButton text size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity">
              <template #icon><NIcon :size="12"><Plus /></NIcon></template>
            </NButton>
          </template>
          <div class="w-52">
            <div class="text-xs text-gray-500 mb-2">{{ t("library.mod.selectTag") }}</div>
            <NSpace vertical :size="4">
              <NCheckbox
                v-for="t in PRESET_TAGS"
                :key="t.id"
                size="small"
                :checked="getTags(mod.id).includes(t.id)"
                @update:checked="() => toggleTag(mod.id, t.id)"
              >
                <span class="text-xs">{{ getTagLabel(t.id) }}</span>
              </NCheckbox>
            </NSpace>
          </div>
        </NPopover>

        <!-- 备注 -->
        <NPopover trigger="click" placement="bottom" @update:show="(v: boolean) => v && openNotePopover(mod.id)">
          <template #trigger>
            <NButton text size="tiny" :type="hasNote(mod.id) ? 'warning' : 'default'">
              <template #icon><NIcon :size="12"><StickyNote /></NIcon></template>
            </NButton>
          </template>
          <div class="w-56">
            <div class="text-xs text-gray-500 mb-2">{{ t("library.mod.note") }}</div>
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
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4">
      <NButton text size="tiny" :disabled="busy" @click="emit('openFolder', mod)">
        <template #icon><NIcon :size="14"><FolderOpen /></NIcon></template>
      </NButton>
      <NPopconfirm @positive-click="() => emit('uninstall', mod)">
        <template #trigger>
          <NButton text size="tiny" type="error" :disabled="busy">
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

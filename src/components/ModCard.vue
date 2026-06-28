<script setup lang="ts">
import { ref, computed } from "vue"
import { currentLocale } from "@/i18n"
import { useI18n } from "vue-i18n"
import {
  NTag, NButton, NIcon, NSwitch, NPopconfirm, NPopover, NCheckbox, NSpace, NInput,
} from "naive-ui"
import { FolderOpen, Trash2, Plus, StickyNote, Copy, Check } from "@lucide/vue"
import IconBtn from "./IconBtn.vue"
import { useModTags, PRESET_TAGS } from "@/composables/useModTags"
import { useModNotes } from "@/composables/useModNotes"
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
  (e: "openUpdateUrl", mod: InstalledMod): void
  (e: "unsubscribe", workshopId: string): void
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

const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null
async function copyModId() {
  try {
    await navigator.clipboard.writeText(props.mod.id)
    copied.value = true
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copied.value = false }, 1500)
  } catch { /* ignore */ }
}
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
        <span class="font-medium truncate max-w-[320px] text-c-primary" :title="mod.name">
          {{ mod.name }}
        </span>
        <span class="text-xs text-c-muted font-mono truncate">{{ mod.version ?? "—" }}</span>
        <span
          v-if="hasUpdate && updateInfo?.remoteVersion"
          class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-xs font-bold leading-none cursor-pointer transition-transform hover:scale-105 active:scale-95"
          :style="{
            backgroundColor: 'color-mix(in srgb, var(--primary-color) 18%, transparent)',
            color: 'var(--primary-color)',
            border: '1px solid color-mix(in srgb, var(--primary-color) 35%, transparent)',
          }"
          :title="'v' + updateInfo!.remoteVersion + ' 可用'"
          @click="emit('openUpdateUrl', mod)"
        >
          ↑ {{ updateInfo!.remoteVersion }}
        </span>
        <NTag v-if="mod.source === 'workshop'" type="info" size="tiny" :bordered="false">
          {{ t("library.mod.workshop") }}
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
      <IconBtn :icon="Plus" :tip="t('library.mod.selectTag')">
        <template #trigger="{ disabled: d }">
          <NPopover trigger="click" placement="bottom-end">
            <template #trigger>
              <NButton text size="tiny" :disabled="d">
                <template #icon><NIcon :size="14"><Plus /></NIcon></template>
              </NButton>
            </template>
            <div class="w-52">
              <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.selectTag") }}</div>
              <NSpace vertical :size="4">
                <NCheckbox v-for="t in PRESET_TAGS" :key="t.id" size="small"
                  :checked="getTags(mod.id).includes(t.id)"
                  @update:checked="() => toggleTag(mod.id, t.id)">
                  <span class="text-xs">{{ getTagLabel(t.id, currentLocale) }}</span>
                </NCheckbox>
              </NSpace>
            </div>
          </NPopover>
        </template>
      </IconBtn>

      <IconBtn :icon="StickyNote" :tip="t('library.mod.note')">
        <template #trigger="{ disabled: d }">
          <NPopover trigger="click" placement="bottom" @update:show="(v: boolean) => v && openNotePopover(mod.id)">
            <template #trigger>
              <NButton text size="tiny" :disabled="d">
                <template #icon><NIcon :size="14"><StickyNote /></NIcon></template>
              </NButton>
            </template>
            <div class="w-56">
              <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.note") }}</div>
              <NInput :value="noteDraft" type="textarea" size="small"
                :placeholder="t('library.mod.notePlaceholder')"
                :autosize="{ minRows: 2, maxRows: 6 }"
                @update:value="(v: string) => noteDraft = v"
                @blur="saveNote(mod.id)" />
            </div>
          </NPopover>
        </template>
      </IconBtn>

      <IconBtn :icon="FolderOpen" :tip="t('library.mod.openFolder')" @click="emit('openFolder', mod)" />

      <IconBtn v-if="mod.source !== 'workshop'" :icon="Trash2" :tip="t('library.mod.uninstall')" type="error">
        <template #trigger="{ icon: ic, disabled: d, type: tty }">
          <NPopconfirm @positive-click="() => emit('uninstall', mod)">
            <template #trigger>
              <NButton text size="tiny" :disabled="d" :type="tty">
                <template #icon><NIcon :size="14"><component :is="ic" /></NIcon></template>
              </NButton>
            </template>
            {{ t("library.mod.confirmUninstall", { name: mod.name }) }}
          </NPopconfirm>
        </template>
      </IconBtn>

      <IconBtn v-if="mod.source === 'workshop' && mod.workshopId" :icon="Trash2" :tip="t('library.mod.unsubscribe')" type="error">
        <template #trigger="{ icon: ic, disabled: d, type: tty }">
          <NPopconfirm @positive-click="() => emit('unsubscribe', mod.workshopId!)">
            <template #trigger>
              <NButton text size="tiny" :disabled="d" :type="tty">
                <template #icon><NIcon :size="14"><component :is="ic" /></NIcon></template>
              </NButton>
            </template>
            {{ t("library.mod.confirmUnsubscribe", { name: mod.name }) }}
          </NPopconfirm>
        </template>
      </IconBtn>
      <IconBtn :icon="copied ? Check : Copy" :tip="copied ? t('library.mod.copied') : t('library.mod.copyId')"
        @click="copyModId" />
      <NPopover v-if="mod.source === 'workshop'" trigger="hover" placement="left" :width="200">
        <template #trigger>
          <span>
            <NSwitch
              :value="enabled"
              :disabled="true"
            />
          </span>
        </template>
        <span class="text-xs">{{ t("library.mod.workshopHint") }}</span>
      </NPopover>
      <NSwitch
        v-else
        :value="enabled"
        :disabled="busy || toggleDisabled"
        @update:value="() => emit('toggle', mod)"
      />
    </div>
  </div>
</template>

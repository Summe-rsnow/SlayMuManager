<script setup lang="ts">
import { ref, computed } from "vue"
import { useI18n } from "vue-i18n"
import { currentLocale } from "@/i18n"
import { Badge, Button, Card, Pill, Switch, Popover, Checkbox, Textarea, Tooltip } from "@xcmm1543/miuui-vue"
import { FolderOpen, Trash2, Plus, StickyNote, Copy, Check, Layers } from "@lucide/vue"
import IconBtn from "./IconBtn.vue"
import ConfirmBtn from "./ConfirmBtn.vue"
import FloatingTip from "./FloatingTip.vue"
import { useTagStore, PRESET_TAGS } from "@/stores/useTagStore"
import { useNoteStore } from "@/stores/useNoteStore"
import { useModCacheStore } from "@/stores/useModCacheStore"
import type { InstalledMod, ModUpdateInfo } from "../types"

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
  (e: "manageVersions", mod: InstalledMod): void
}>()

const { t } = useI18n()
const modCacheStore = useModCacheStore()
const conflictGroup = computed(() => modCacheStore.getConflictGroup(props.mod.id))
const hasConflict = computed(() => !!conflictGroup.value)
const conflictCount = computed(() => conflictGroup.value?.instances.length ?? 0)
const { getTags, toggleTag, getTagLabel, isPresetTag } = useTagStore()
const { getNote, setNote, hasNote } = useNoteStore()

const noteDraft = ref("")
function openNotePopover(modId: string) {
  noteDraft.value = getNote(modId)
}
function saveNote(modId: string) {
  setNote(modId, noteDraft.value)
}

// --- 动态卡片样式（Level 1: Surface 玻璃质感）---
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null
async function copyModId() {
  try {
    await navigator.clipboard.writeText(props.mod.id)
    copied.value = true
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <Card
    material="glass"
    interactive
    padding="sm"
    class="group"
    :class="[{ 'pointer-events-none opacity-60': busy }]"
  >
    <div class="flex items-center justify-between">
      <div class="flex-1 min-w-0">
        <!-- 名称 + 版本 + 标签 -->
        <div class="flex items-center gap-1.5 flex-wrap">
          <Tooltip side="top" material="glass">
            <template #trigger>
              <span class="font-semibold text-[15px] truncate max-w-[320px] text-c-primary">
                {{ mod.name }}
              </span>
            </template>
            {{ mod.name }}
          </Tooltip>
          <span
            class="bg-c-secondary/50 px-1.5 py-0.5 rounded-md text-xs text-c-muted font-mono truncate"
            >{{ mod.version ?? "—" }}</span
          >
          <span
            v-if="hasUpdate && updateInfo?.remoteVersion"
            class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-full text-[11px] font-bold leading-none cursor-pointer transition-transform hover:scale-105 active:scale-95 animate-pulse"
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
          <Badge v-if="mod.source === 'workshop'" variant="info" size="xs">
            {{ t("library.mod.workshop") }}
          </Badge>
          <Badge v-if="mod.affectsGameplay" variant="warning" size="xs">
            {{ t("library.mod.affectsGameplay") }}
          </Badge>
          <!-- 多版本共存警告标签 -->
          <Tooltip v-if="hasConflict" side="top" material="glass">
            <template #trigger>
              <span
                class="inline-flex items-center cursor-pointer"
                @click.stop="emit('manageVersions', mod)"
              >
                <Badge variant="warning" size="xs" class="font-bold animate-pulse hover:opacity-80">
                  ⚠️ 多版本共存 ({{ conflictCount }})
                </Badge>
              </span>
            </template>
            检测到此模组安装了 {{ conflictCount }} 个不同版本，点击打开版本保留管理面板
          </Tooltip>
          <Pill
            v-for="tagId in getTags(mod.id)"
            :key="tagId"
            size="sm"
            variant="tag"
            :active="isPresetTag(tagId)"
            :label="getTagLabel(tagId, currentLocale)"
            removable
            @remove="toggleTag(mod.id, tagId)"
          />
        </div>

        <!-- 作者 + 文件夹 -->
        <div class="text-xs text-c-muted mt-0.5">
          {{ mod.author ?? t("library.mod.unknownAuthor") }} · {{ mod.folderName }}
        </div>

        <!-- 备注内容（有备注时显示为淡色一行） -->
        <div v-if="hasNote(mod.id)" class="mt-0.5 text-xs text-c-muted truncate">
          <StickyNote :size="12" class="inline-block mr-0.5 align-middle" />
          <span class="align-middle">{{ getNote(mod.id) }}</span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div
        class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4 opacity-60 group-hover:opacity-100 transition-opacity duration-200"
      >
        <IconBtn :icon="Plus" :tip="t('library.mod.selectTag')">
          <template #trigger="{ disabled: d }">
            <Popover side="bottom" align="end" material="glass">
              <template #trigger>
                <Button
                  variant="ghost"
                  size="xs"
                  icon-only
                  :aria-label="t('library.mod.selectTag')"
                  :disabled="d"
                >
                  <Plus :size="14" />
                </Button>
              </template>
              <div class="w-52">
                <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.selectTag") }}</div>
                <div class="flex flex-col gap-1.5">
                  <Checkbox
                    v-for="tItem in PRESET_TAGS"
                    :key="tItem.id"
                    size="sm"
                    :model-value="getTags(mod.id).includes(tItem.id)"
                    :label="getTagLabel(tItem.id, currentLocale)"
                    @update:model-value="() => toggleTag(mod.id, tItem.id)"
                  />
                </div>
              </div>
            </Popover>
          </template>
        </IconBtn>

        <IconBtn :icon="StickyNote" :tip="t('library.mod.note')">
          <template #trigger="{ disabled: d }">
            <Popover
              side="bottom"
              align="center"
              material="glass"
              @update:open="(v: boolean) => v && openNotePopover(mod.id)"
            >
              <template #trigger>
                <Button
                  variant="ghost"
                  size="xs"
                  icon-only
                  :aria-label="t('library.mod.note')"
                  :disabled="d"
                >
                  <StickyNote :size="14" />
                </Button>
              </template>
              <div class="w-56">
                <div class="text-xs text-c-secondary mb-2">{{ t("library.mod.note") }}</div>
                <Textarea
                  :model-value="noteDraft"
                  size="sm"
                  :placeholder="t('library.mod.notePlaceholder')"
                  :rows="2"
                  auto-resize
                  :auto-resize-max-height="140"
                  @update:model-value="(v: string) => (noteDraft = v)"
                  @blur="saveNote(mod.id)"
                />
              </div>
            </Popover>
          </template>
        </IconBtn>

        <IconBtn
          v-if="hasConflict"
          :icon="Layers"
          tip="多版本管理与保留"
          @click="emit('manageVersions', mod)"
        />

        <IconBtn
          :icon="FolderOpen"
          :tip="t('library.mod.openFolder')"
          @click="emit('openFolder', mod)"
        />

        <ConfirmBtn
          v-if="mod.source !== 'workshop'"
          :icon="Trash2"
          :tip="t('library.mod.uninstall')"
          :confirmText="t('library.mod.confirmUninstall', { name: mod.name })"
          @confirm="emit('uninstall', mod)"
        />

        <ConfirmBtn
          v-if="mod.source === 'workshop' && mod.workshopId"
          :icon="Trash2"
          :tip="t('library.mod.unsubscribe')"
          :confirmText="t('library.mod.confirmUnsubscribe', { name: mod.name })"
          @confirm="emit('unsubscribe', mod.workshopId!)"
        />
        <IconBtn
          :icon="copied ? Check : Copy"
          :tip="copied ? t('library.mod.copied') : t('library.mod.copyId')"
          @click="copyModId"
        />
        <FloatingTip
          v-if="mod.source === 'workshop'"
          :text="t('library.mod.workshopHint')"
          placement="left"
          :width="200"
        >
          <template #default>
            <Switch :model-value="enabled" :disabled="true" size="sm" :aria-label="mod.name" />
          </template>
        </FloatingTip>
        <Switch
          v-else
          :model-value="enabled"
          :disabled="busy || toggleDisabled"
          size="sm"
          :aria-label="mod.name"
          @update:model-value="() => emit('toggle', mod)"
        />
      </div>
    </div>
  </Card>
</template>

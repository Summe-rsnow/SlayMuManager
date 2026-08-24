<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { NModal, NCard, NTag, NButton, NIcon } from "naive-ui"
import { Sparkles, Calendar, PartyPopper, CheckCircle2, Wrench, Zap, Palette } from "@lucide/vue"
import { releaseNotesConfig } from "@/data/releaseNotes"

const { t } = useI18n()

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: "update:show", v: boolean): void
}>()

function getTypeIcon(type: string) {
  switch (type) {
    case "feat": return Sparkles
    case "perf": return Zap
    case "ui": return Palette
    case "fix": return Wrench
    default: return CheckCircle2
  }
}

function getTypeTag(type: string) {
  switch (type) {
    case "feat": return { label: t("releaseNotes.tags.feat"), type: "primary" as const }
    case "perf": return { label: t("releaseNotes.tags.perf"), type: "success" as const }
    case "ui": return { label: t("releaseNotes.tags.ui"), type: "info" as const }
    case "fix": return { label: t("releaseNotes.tags.fix"), type: "warning" as const }
    default: return { label: t("releaseNotes.tags.opt"), type: "default" as const }
  }
}
</script>

<template>
  <NModal
    :show="show"
    mask-closable
    @update:show="(v) => emit('update:show', v)"
  >
    <NCard
      class="glass-modal"
      :style="{
        width: '560px',
        maxHeight: '85vh',
        boxShadow: 'var(--shadow-glass-modal)',
      }"
      :bordered="false"
      role="dialog"
    >
      <template #header>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-xl bg-primary-theme/15 text-primary-theme flex items-center justify-center flex-shrink-0">
            <NIcon :size="18"><PartyPopper /></NIcon>
          </div>
          <div>
            <span class="text-base font-semibold text-c-primary block">{{ t("releaseNotes.dialogTitle") }}</span>
            <span class="text-xs text-c-muted font-normal block mt-0.5">{{ t("releaseNotes.dialogSubtitle") }}</span>
          </div>
        </div>
      </template>

      <div class="flex flex-col gap-4 py-1 max-h-[58vh] overflow-y-auto pr-1">
        <div
          v-for="(item, index) in releaseNotesConfig"
          :key="item.version"
          class="flex flex-col gap-2.5 rounded-xl p-3.5 transition-all"
          :class="index === 0 ? 'bg-primary-theme/5 border border-primary-theme/25' : 'bg-white/3 border border-white/6'"
        >
          <!-- Version header -->
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <span class="font-bold font-mono text-sm text-c-primary">v{{ item.version }}</span>
              <NTag v-if="item.tagKey" type="primary" size="tiny" round>{{ t(item.tagKey) }}</NTag>
            </div>
            <div class="flex items-center gap-1.5 text-xs text-c-muted font-mono">
              <NIcon :size="12"><Calendar /></NIcon>
              <span>{{ item.date }}</span>
            </div>
          </div>

          <div class="text-xs font-semibold text-c-primary">
            {{ t(item.titleKey) }}
          </div>

          <!-- Highlights (for latest version) -->
          <div
            v-if="item.highlightsKeys && item.highlightsKeys.length > 0"
            class="flex flex-col gap-1.5 p-2.5 rounded-lg bg-primary-theme/8 border border-primary-theme/15"
          >
            <div class="text-xs font-semibold text-primary-theme flex items-center gap-1.5 mb-0.5">
              <NIcon :size="12"><Sparkles /></NIcon>
              <span>{{ t("releaseNotes.highlights") }}</span>
            </div>
            <div
              v-for="(hlKey, hIdx) in item.highlightsKeys"
              :key="hIdx"
              class="flex items-start gap-2 text-xs text-c-primary leading-relaxed"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-primary-theme flex-shrink-0 mt-1.5" />
              <span>{{ t(hlKey) }}</span>
            </div>
          </div>

          <!-- Changes list -->
          <div class="flex flex-col gap-1.5 mt-0.5">
            <div
              v-for="(ch, cIdx) in item.changes"
              :key="cIdx"
              class="flex items-start gap-2 text-xs text-c-secondary leading-relaxed"
            >
              <NTag :type="getTypeTag(ch.type).type" size="tiny" class="flex-shrink-0 mt-0.5">
                <template #icon>
                  <NIcon :size="10"><component :is="getTypeIcon(ch.type)" /></NIcon>
                </template>
                {{ getTypeTag(ch.type).label }}
              </NTag>
              <span class="flex-1">{{ t(ch.textKey) }}</span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end">
          <NButton type="primary" size="small" @click="emit('update:show', false)">
            {{ t("common.ok") }}
          </NButton>
        </div>
      </template>
    </NCard>
  </NModal>
</template>

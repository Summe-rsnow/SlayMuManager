<script setup lang="ts">
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { NTag, NTooltip, NSwitch, NIcon } from "naive-ui"
import { Download, Edit3, Trash2, Sparkles } from "@lucide/vue"
import IconBtn from "./IconBtn.vue"
import ConfirmBtn from "./ConfirmBtn.vue"
import FloatingTip from "./FloatingTip.vue"
import type { ModProfile } from "@/types"

const props = defineProps<{
  profile: ModProfile
  isActive: boolean
  loading?: boolean
  isExporting?: boolean
}>()

const emit = defineEmits<{
  (e: "apply", profile: ModProfile): void
  (e: "export", profile: ModProfile): void
  (e: "edit", profile: ModProfile): void
  (e: "delete", profile: ModProfile): void
}>()

const { t } = useI18n()

// --- 动态卡片样式（Level 1: Surface 玻璃质感，与 ModCard 保持完全一致）---
const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  borderColor: props.isActive
    ? "color-mix(in srgb, var(--primary-color) 45%, var(--glass-surface-border))"
    : "var(--glass-surface-border)",
  boxShadow: "var(--shadow-glass-surface)",
  transform: "translateZ(0)",
}))
</script>

<template>
  <div
    class="group mod-card flex items-center justify-between p-3.5 rounded-xl border transition-[border-color,box-shadow,transform] duration-200 hover:-translate-y-1"
    :style="cardStyle"
    :class="[{ 'pointer-events-none opacity-60': loading }]"
  >
    <div class="flex-1 min-w-0" style="position: relative; z-index: 2">
      <!-- 预设名称 + 数量徽章 + 标签 -->
      <div class="flex items-center gap-1.5 flex-wrap">
        <NTooltip trigger="hover" placement="top">
          <template #trigger>
            <span class="font-semibold text-[15px] truncate max-w-[320px] text-c-primary">
              {{ profile.name }}
            </span>
          </template>
          {{ profile.name }}
        </NTooltip>

        <span
          class="bg-c-secondary/50 px-1.5 py-0.5 rounded-md text-xs text-c-muted font-mono truncate"
        >
          {{ t("profiles.modCount", { n: profile.modIds.length }) }}
        </span>

        <NTag v-if="profile.builtin" type="success" size="tiny" :bordered="false">
          {{ t("profiles.builtin") }}
        </NTag>

        <NTag v-if="isActive" type="info" size="tiny" :bordered="false" class="font-medium">
          <template #icon>
            <NIcon :size="10"><Sparkles /></NIcon>
          </template>
          {{ t("profiles.active") }}
        </NTag>
      </div>

      <!-- 描述 / 备注内容（与 ModCard 结构一致） -->
      <div class="text-xs text-c-muted mt-0.5 truncate">
        <FloatingTip
          v-if="profile.description && profile.description.trim().length > 0"
          :text="profile.description"
          truncated
          :max-lines="1"
        />
        <span v-else>{{ t("profiles.noDescription") }}</span>
      </div>
    </div>

    <!-- 操作按钮区（完全对齐 ModCard: 悬停显现 + 导出 + 编辑 + 删除 + 启用开关） -->
    <div
      class="mod-actions flex items-center gap-2 flex-shrink-0 ml-4 opacity-60 group-hover:opacity-100 transition-opacity duration-200"
    >
      <IconBtn
        v-if="!profile.builtin"
        :icon="Download"
        :tip="t('profiles.exportBundle')"
        :disabled="isExporting"
        @click="emit('export', profile)"
      />

      <IconBtn
        v-if="!profile.builtin"
        :icon="Edit3"
        :tip="t('profiles.form.editTitle')"
        @click="emit('edit', profile)"
      />

      <ConfirmBtn
        v-if="!profile.builtin"
        :icon="Trash2"
        :confirmText="t('profiles.confirmDelete', { name: profile.name })"
        :tip="t('common.delete')"
        @confirm="emit('delete', profile)"
      />

      <NTooltip trigger="hover" placement="top">
        <template #trigger>
          <NSwitch
            :value="isActive"
            :disabled="loading || isActive"
            @update:value="() => emit('apply', profile)"
          />
        </template>
        {{ isActive ? t("profiles.active") : t("profiles.applyBtn") }}
      </NTooltip>
    </div>
  </div>
</template>

<style scoped>
.mod-card {
  transform: translateZ(0);
  transition:
    box-shadow 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.mod-card:hover {
  background-color: var(--glass-surface-bg-hover) !important;
  box-shadow: var(--shadow-card-hover), var(--shadow-glow) !important;
  border-color: color-mix(
    in srgb,
    var(--primary-color) 35%,
    var(--glass-surface-border)
  ) !important;
}
</style>

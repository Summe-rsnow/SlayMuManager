<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { Badge, Card, Switch, Tooltip } from "@xcmm1543/miuui-vue"
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
</script>

<template>
  <Card
    material="glass"
    interactive
    padding="sm"
    class="group"
    :class="[{ 'pointer-events-none opacity-60': loading }]"
    :styles="
      isActive
        ? { root: { borderColor: 'color-mix(in srgb, var(--primary-color) 45%, transparent)' } }
        : undefined
    "
  >
    <div class="flex items-center justify-between">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5 flex-wrap">
          <Tooltip side="top" material="glass">
            <template #trigger>
              <span class="font-semibold text-[15px] truncate max-w-[320px] text-c-primary">
                {{ profile.name }}
              </span>
            </template>
            {{ profile.name }}
          </Tooltip>

          <span
            class="bg-c-secondary/50 px-1.5 py-0.5 rounded-md text-xs text-c-muted font-mono truncate"
          >
            {{ t("profiles.modCount", { n: profile.modIds.length }) }}
          </span>

          <Badge v-if="profile.builtin" variant="success" size="xs">
            {{ t("profiles.builtin") }}
          </Badge>

          <Badge v-if="isActive" variant="info" size="xs" class="font-medium">
            <Sparkles :size="10" />
            {{ t("profiles.active") }}
          </Badge>
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

      <!-- 操作按钮区 -->
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

        <Tooltip side="top" material="glass">
          <template #trigger>
            <span class="inline-flex items-center">
              <Switch
                :model-value="isActive"
                :disabled="loading || isActive"
                size="sm"
                :aria-label="profile.name"
                @update:model-value="() => emit('apply', profile)"
              />
            </span>
          </template>
          {{ isActive ? t("profiles.active") : t("profiles.applyBtn") }}
        </Tooltip>
      </div>
    </div>
  </Card>
</template>

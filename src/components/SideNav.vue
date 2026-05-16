<script setup lang="ts">
import { h, computed, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { NMenu, NIcon, NSelect, NButton, NModal, NCard, NSpace, type MenuOption } from "naive-ui"
import { Library, Compass, FolderHeart, Save, Settings, Play, AlertTriangle } from "lucide-vue-next"
import { useSidebarActions } from "../composables/useSidebarActions"

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const {
  launchingGame,
  showLaunchMismatchDialog,
  launchMismatchStatus,
  handleLaunchGame,
  handleGoToSaves,
  handleLaunchAnyway,
  quickPresetId,
  quickPresetOptions,
  loadQuickPresets,
  handleQuickPreset,
} = useSidebarActions()

const menuOptions = computed<MenuOption[]>(() => [
  {
    label: t("nav.library"),
    key: "/",
    icon: () => h(NIcon, null, { default: () => h(Library, { size: 18 }) }),
  },
  {
    label: t("nav.discover"),
    key: "/discover",
    icon: () => h(NIcon, null, { default: () => h(Compass, { size: 18 }) }),
  },
  {
    label: t("nav.profiles"),
    key: "/profiles",
    icon: () => h(NIcon, null, { default: () => h(FolderHeart, { size: 18 }) }),
  },
  {
    label: t("nav.saves"),
    key: "/saves",
    icon: () => h(NIcon, null, { default: () => h(Save, { size: 18 }) }),
  },
  {
    label: t("nav.settings"),
    key: "/settings",
    icon: () => h(NIcon, null, { default: () => h(Settings, { size: 18 }) }),
  },
])

const activeKey = computed(() => {
  if (route.path === "/") return "/"
  const prefix = menuOptions.value.find(
    (o) => typeof o.key === "string" && route.path.startsWith(o.key) && o.key !== "/",
  )
  return prefix ? (prefix.key as string) : null
})

function handleUpdateValue(key: string) {
  router.push(key)
}

onMounted(() => {
  loadQuickPresets()
})
</script>

<template>
  <nav
    class="w-52 flex-shrink-0 flex flex-col p-3"
    :style="{
      borderRight: '1px solid var(--color-border)',
      backgroundColor: 'var(--color-bg-sidebar)',
    }"
  >
    <NMenu
      :value="activeKey"
      :options="menuOptions"
      :indent="16"
      :default-expanded-keys="[]"
      @update:value="handleUpdateValue"
    />

    <!-- 底部操作区 -->
    <div class="mt-auto space-y-2 pt-4" :style="{ borderTop: '1px solid var(--color-border)' }">
      <!-- 预设快速切换 -->
      <NSelect
        v-model:value="quickPresetId"
        :options="quickPresetOptions"
        :placeholder="t('library.quickPresetPlaceholder')"
        size="small"
        :disabled="quickPresetOptions.length === 0"
        @update:value="handleQuickPreset"
      />

      <!-- 启动游戏 -->
      <NButton
        block
        size="large"
        strong
        :loading="launchingGame"
        @click="handleLaunchGame"
        class="launch-btn"
      >
        <template #icon>
          <NIcon :size="20"><Play /></NIcon>
        </template>
        {{ t("library.launchGame") }}
      </NButton>
    </div>

    <!-- 云存档差异确认弹窗 -->
    <NModal
      :show="showLaunchMismatchDialog"
      @update:show="(v: boolean) => !v && (showLaunchMismatchDialog = false)"
    >
      <NCard style="width: 440px" :bordered="false" role="dialog">
        <template #header>
          <div class="flex items-center gap-2">
            <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
            <span class="font-semibold">{{ t("library.launchMismatch.title") }}</span>
          </div>
        </template>
        <NSpace v-if="launchMismatchStatus" vertical :size="8">
          <p class="text-sm text-c-secondary">{{ t("library.launchMismatch.warning") }}</p>
          <div class="text-xs text-c-secondary bg-amber-50 rounded p-2 space-y-1">
            <div class="flex justify-between" v-if="launchMismatchStatus.differentCount > 0">
              <span>{{ t("saves.cloud.mismatch.different", { n: launchMismatchStatus.differentCount }) }}</span>
            </div>
            <div class="flex justify-between" v-if="launchMismatchStatus.localOnlyCount > 0">
              <span>{{ t("saves.cloud.mismatch.localOnly", { n: launchMismatchStatus.localOnlyCount }) }}</span>
            </div>
            <div class="flex justify-between" v-if="launchMismatchStatus.cloudOnlyCount > 0">
              <span>{{ t("saves.cloud.mismatch.cloudOnly", { n: launchMismatchStatus.cloudOnlyCount }) }}</span>
            </div>
          </div>
          <div class="flex justify-between mt-2 gap-2">
            <NButton secondary size="small" @click="handleGoToSaves">
              {{ t("library.launchMismatch.goToSaves") }}
            </NButton>
            <NButton type="warning" size="small" @click="handleLaunchAnyway">
              {{ t("library.launchMismatch.forceLaunch") }}
            </NButton>
          </div>
        </NSpace>
      </NCard>
    </NModal>
  </nav>
</template>

<style scoped>
.launch-btn {
  --n-color: var(--primary-color) !important;
  --n-color-hover: var(--primary-color-hover) !important;
  --n-color-pressed: var(--primary-color-pressed) !important;
  --n-color-active: var(--primary-color) !important;
  --n-height: 44px;
  font-size: 15px;
  border-radius: 10px;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--primary-color) 40%, transparent);
  transition: all 0.2s ease;
}
.launch-btn:hover {
  box-shadow: 0 4px 14px color-mix(in srgb, var(--primary-color) 50%, transparent);
  transform: translateY(-1px);
}
</style>

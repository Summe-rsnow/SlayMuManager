<script setup lang="ts">
import { h, computed, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { NMenu, NIcon, NSelect, NButton, NModal, NCard, NSpace, type MenuOption } from "naive-ui"
import { Library, Compass, FolderHeart, Save, Settings, Play, AlertTriangle, Menu } from "lucide-vue-next"
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
  sidebarCollapsed,
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
  <!-- ============ 左下浮动按钮组 ============ -->
  <div class="absolute left-8 bottom-8 z-30 flex flex-col gap-3">
    <!-- 菜单面板（展开时在按钮上方淡入） -->
    <Transition name="menu">
      <div
        v-if="!sidebarCollapsed"
        class="absolute left-0 z-20 flex flex-col w-52 rounded-2xl border shadow-lg backdrop-blur-xl overflow-hidden"
        :style="{
          backgroundColor: 'color-mix(in srgb, var(--color-bg-sidebar) 60%, transparent)',
          borderColor: 'var(--color-border)',
          bottom: 'calc(100% + 8px)',
          maxHeight: 'calc(100vh - 140px)',
        }"
      >
        <div class="flex-1 overflow-y-auto p-3 pb-0">
          <NMenu
            :value="activeKey"
            :options="menuOptions"
            :indent="16"
            :default-expanded-keys="[]"
            @update:value="handleUpdateValue"
          />
        </div>
        <div class="flex-shrink-0 px-3 pb-3">
          <NSelect
            v-model:value="quickPresetId"
            :options="quickPresetOptions"
            :placeholder="t('library.quickPresetPlaceholder')"
            size="small"
            :disabled="quickPresetOptions.length === 0"
            @update:value="handleQuickPreset"
          />
        </div>
      </div>
    </Transition>

    <!-- 「菜单」按钮（主色调） -->
    <button
      class="flex items-center gap-2 rounded-xl shadow-lg px-6 py-3 cursor-pointer select-none outline-none border-0 transition-all duration-200 hover:scale-105 active:scale-95 text-sm font-medium whitespace-nowrap"
      :style="{
        backgroundColor: 'var(--primary-color)',
        color: '#fff',
      }"
      @click="sidebarCollapsed = !sidebarCollapsed"
    >
      <NIcon :size="20" color="#fff"><Menu /></NIcon>
      <span>{{ t("nav.menu") }}</span>
    </button>

    <!-- 「启动游戏」按钮（高斯模糊玻璃药丸） -->
    <button
      class="flex items-center gap-2 rounded-xl shadow-lg backdrop-blur-xl px-6 py-3 cursor-pointer select-none outline-none border-0 transition-all duration-200 hover:scale-105 active:scale-95 text-sm font-medium whitespace-nowrap"
      :style="{
        backgroundColor: 'color-mix(in srgb, var(--color-bg-sidebar) 65%, transparent)',
        color: 'var(--color-text-primary)',
      }"
      :disabled="launchingGame"
      @click="handleLaunchGame"
    >
      <NIcon :size="20" :color="'var(--primary-color)'"><Play /></NIcon>
      <span>{{ t("library.launchGame") }}</span>
    </button>
  </div>

  <!-- ============ 云存档差异确认弹窗 ============ -->
  <NModal
    :show="showLaunchMismatchDialog"
    @update:show="(v: boolean) => { if (!v) { showLaunchMismatchDialog = false; launchingGame = false } }"
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
</template>

<style scoped>
/* 去除 NMenu 选中项的左侧默认指示条 */
:deep(.n-menu-item-content::before) {
  display: none !important;
}

/* 菜单项左侧光源效果 —— 主题色自左向右渐变辉光 */
:deep(.n-menu-item-content) {
  position: relative;
  transition: all 0.2s ease;
}
:deep(.n-menu-item-content:hover) {
  background: linear-gradient(
    to right,
    color-mix(in srgb, var(--primary-color) 6%, transparent),
    transparent 80%
  ) !important;
}
:deep(.n-menu-item-content--selected) {
  background: linear-gradient(
    to right,
    color-mix(in srgb, var(--primary-color) 14%, transparent),
    transparent 65%
  ) !important;
}

/* 菜单面板展开/收起过渡动画 */
.menu-enter-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}
.menu-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.menu-enter-from {
  opacity: 0;
  transform: translateY(16px);
}
.menu-leave-to {
  opacity: 0;
  transform: translateY(16px);
}
</style>

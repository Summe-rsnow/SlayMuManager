<script setup lang="ts">
import { h, computed, ref, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { NMenu, NIcon, NSelect, NButton, NSpace, type MenuOption } from "naive-ui"
import { Library, Compass, FolderHeart, Save, Settings, Play, LoaderCircle, AlertTriangle, Menu } from "@lucide/vue"
import { useSidebarActions } from "../composables/useSidebarActions"
import AppDialog from "../components/AppDialog.vue"

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

const showPresetPanel = ref(false)
const presetMenuOpen = ref(false)
let presetPanelTimer: ReturnType<typeof setTimeout> | null = null
const PRESET_HOVER_DELAY = 100 // ms

function onPresetEnter() {
  if (presetPanelTimer) {
    clearTimeout(presetPanelTimer)
    presetPanelTimer = null
  }
  showPresetPanel.value = true
}

function onPresetLeave() {
  if (presetMenuOpen.value) return // 下拉菜单打开时不移除面板
  presetPanelTimer = setTimeout(() => {
    showPresetPanel.value = false
    presetPanelTimer = null
  }, PRESET_HOVER_DELAY)
}

function onPresetSelect(val: string) {
  handleQuickPreset(val)
  presetMenuOpen.value = false
  showPresetPanel.value = false
}

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
      </div>
    </Transition>

    <!-- 「菜单」按钮（主色调 + 高斯模糊） -->
    <button
      class="group relative w-12 hover:w-36 h-12 rounded-xl shadow-lg backdrop-blur-xl cursor-pointer select-none outline-none border-0 overflow-hidden transition-all duration-300 ease-out hover:shadow-xl active:scale-95"
      :style="{
        backgroundColor: 'color-mix(in srgb, var(--primary-color) 50%, transparent)',
        color: '#fff',
      }"
      @click="sidebarCollapsed = !sidebarCollapsed"
    >
      <!-- 闭合态 -->
      <div class="absolute inset-0 flex items-center justify-center transition-all duration-300 ease-out opacity-100 scale-100 group-hover:opacity-0 group-hover:scale-75">
        <NIcon :size="24" color="#fff"><Menu /></NIcon>
      </div>
      <!-- 展开态 -->
      <div class="absolute inset-0 flex items-center gap-3 px-4 transition-all duration-300 ease-out opacity-0 scale-75 group-hover:opacity-100 group-hover:scale-100">
        <NIcon :size="24" color="#fff" class="flex-shrink-0"><Menu /></NIcon>
        <span class="text-[17px] font-medium whitespace-nowrap text-white">{{ t("nav.menu") }}</span>
      </div>
    </button>

    <!-- 「启动游戏」按钮 + 预设选择悬停面板 -->
    <div
      class="relative"
      @mouseenter="onPresetEnter"
      @mouseleave="onPresetLeave"
    >
      <!-- 预设选择面板（悬停时在按钮上方展开） -->
      <Transition name="preset">
        <div
          v-if="showPresetPanel"
          class="absolute left-0 z-20 w-52 rounded-2xl border shadow-lg backdrop-blur-xl overflow-hidden"
          :style="{
            backgroundColor: 'color-mix(in srgb, var(--color-bg-sidebar) 60%, transparent)',
            borderColor: 'var(--color-border)',
            bottom: 'calc(100% + 8px)',
          }"
        >
          <div class="p-3">
            <NSelect
              v-model:value="quickPresetId"
              :options="quickPresetOptions"
              size="small"
              :disabled="quickPresetOptions.length === 0"
              placement="top"
              @update:value="onPresetSelect"
              @update:show="(show: boolean) => { presetMenuOpen = show }"
            />
          </div>
        </div>
      </Transition>

      <button
        class="group relative w-12 hover:w-42 h-12 rounded-xl shadow-lg backdrop-blur-xl cursor-pointer select-none outline-none border-0 overflow-hidden transition-all duration-300 ease-out hover:shadow-xl active:scale-95"
        :style="{
          backgroundColor: 'color-mix(in srgb, var(--color-bg-sidebar) 65%, transparent)',
          color: 'var(--color-text-primary)',
        }"
        :disabled="launchingGame"
        @click="handleLaunchGame"
      >
        <!-- 闭合态 -->
        <div class="absolute inset-0 flex items-center justify-center transition-all duration-300 ease-out opacity-100 scale-100 group-hover:opacity-0 group-hover:scale-75">
          <NIcon :size="24" :color="'var(--primary-color)'">
            <Play v-if="!launchingGame" />
            <LoaderCircle v-else class="animate-spin" />
          </NIcon>
        </div>
        <!-- 展开态 -->
        <div class="absolute inset-0 flex items-center gap-3 px-4 transition-all duration-300 ease-out opacity-0 scale-75 group-hover:opacity-100 group-hover:scale-100">
          <NIcon :size="24" :color="'var(--primary-color)'" class="flex-shrink-0">
            <Play v-if="!launchingGame" />
            <LoaderCircle v-else class="animate-spin" />
          </NIcon>
          <span class="text-[17px] font-medium whitespace-nowrap">{{ t("library.launch") }}</span>
        </div>
      </button>
    </div>
  </div>

  <!-- ============ 云存档差异确认弹窗 ============ -->
  <AppDialog
    :show="showLaunchMismatchDialog"
    @update:show="(v: boolean) => { if (!v) { showLaunchMismatchDialog = false; launchingGame = false } }"
    width="440px"
  >
    <template #header>
      <div class="flex items-center gap-2">
        <NIcon :size="18" color="#f0a020"><AlertTriangle /></NIcon>
        <span class="font-semibold">{{ t("library.launchMismatch.title") }}</span>
      </div>
    </template>
    <NSpace v-if="launchMismatchStatus" vertical :size="8">
      <p class="text-sm text-c-secondary">{{ t("library.launchMismatch.warning") }}</p>
      <div class="text-xs text-c-secondary bg-c-warning rounded p-2 space-y-1">
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
  </AppDialog>

  <!-- ============ End AppDialogs ============ -->
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

/* 预设面板展开/收起过渡动画（从按钮向上弹出效果） */
.preset-enter-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  transform-origin: bottom center;
}
.preset-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform-origin: bottom center;
}
.preset-enter-from {
  opacity: 0;
  transform: translateY(12px) scaleY(0.92);
}
.preset-leave-to {
  opacity: 0;
  transform: translateY(8px) scaleY(0.95);
}
</style>

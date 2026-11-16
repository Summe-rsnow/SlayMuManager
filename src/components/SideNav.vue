<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, type Component } from "vue"
import { storeToRefs } from "pinia"
import { useRoute, useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { Alert, Button, Select, useConfirm, useToast } from "@xcmm1543/miuui-vue"
import {
  Library,
  Compass,
  Save,
  Settings,
  Play,
  LoaderCircle,
  AlertTriangle,
  Menu,
} from "@lucide/vue"
import { useSidebarStore } from "@/stores/useSidebarStore"
import { useHighlightStore } from "@/stores/useHighlightStore"
import AppDialog from "@/components/AppDialog.vue"

const { t } = useI18n()
const { toast } = useToast()
const { confirm } = useConfirm()
const route = useRoute()
const router = useRouter()

const store = useSidebarStore()
const {
  launchingGame,
  showLaunchMismatchDialog,
  launchMismatchStatus,
  quickPresetId,
  quickPresetOptions,
  sidebarCollapsed,
} = storeToRefs(store)
const {
  handleLaunchGame,
  handleGoToSaves,
  handleLaunchAnyway,
  loadQuickPresets,
  handleQuickPreset,
} = store

const showPresetPanel = ref(false)
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
  if (document.querySelector('.miu-select__content[data-state="open"]')) return
  presetPanelTimer = setTimeout(() => {
    showPresetPanel.value = false
    presetPanelTimer = null
  }, PRESET_HOVER_DELAY)
}

async function onPresetSelect(val: string | number) {
  showPresetPanel.value = false
  const result = await handleQuickPreset(String(val))
  if (result.ok) {
    toast({ title: t("library.success.presetApplied", { name: result.label }), variant: "success" })
  } else {
    toast({ title: `${t("profiles.error.applyFailed")}: ${result.error}`, variant: "error" })
  }
}

async function onLaunchGame() {
  const result = await handleLaunchGame()
  if (result.ok) {
    toast({ title: t("library.success.gameLaunched"), variant: "success" })
  } else if (result.needsGamePath) {
    const accepted = await confirm({
      title: t("settings.prompt.gamePathRequired"),
      description: t("settings.prompt.gamePathRequiredDesc"),
      confirmLabel: t("settings.prompt.goToSettings"),
      cancelLabel: t("common.cancel"),
    })
    if (accepted) {
      useHighlightStore().highlight("game-path")
      router.push("/settings")
    }
  } else if (result.alreadyRunning) {
    toast({ title: t("library.info.gameAlreadyRunning"), variant: "warning" })
  } else if (result.error) {
    toast({ title: t("library.error.launchFailed", { e: result.error }), variant: "error" })
  }
}

async function onLaunchAnyway() {
  const result = await handleLaunchAnyway()
  if (result.ok) {
    toast({ title: t("library.success.gameLaunched"), variant: "success" })
  } else if (result.error) {
    toast({ title: t("library.error.launchFailed", { e: result.error }), variant: "error" })
  }
}

interface NavItem {
  key: string
  label: string
  icon: Component
}

const menuOptions = computed<NavItem[]>(() => [
  { label: t("nav.library"), key: "/", icon: Library },
  { label: t("nav.discover"), key: "/discover", icon: Compass },
  { label: t("nav.saves"), key: "/saves", icon: Save },
  { label: t("nav.settings"), key: "/settings", icon: Settings },
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

onUnmounted(() => {
  if (presetPanelTimer) {
    clearTimeout(presetPanelTimer)
    presetPanelTimer = null
  }
})
</script>

<template>
  <!-- ============ 左下浮动按钮组 ============ -->
  <div class="absolute left-8 bottom-8 z-30 flex flex-col gap-3">
    <!-- 菜单面板（展开时在按钮上方淡入，Level 3: Floating 质感） -->
    <Transition name="menu">
      <div
        v-if="!sidebarCollapsed"
        class="absolute left-0 z-20 flex flex-col w-52 glass-floating overflow-hidden"
        :style="{
          bottom: 'calc(100% + 8px)',
          maxHeight: 'calc(100vh - 140px)',
          border: 'var(--glass-floating-border)',
          boxShadow: 'var(--shadow-glass-floating)',
        }"
      >
        <nav class="flex-1 overflow-y-auto p-2.5 flex flex-col gap-1" :aria-label="t('nav.menu')">
          <button
            v-for="item in menuOptions"
            :key="item.key"
            type="button"
            class="side-nav-item flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-sm font-medium text-left cursor-pointer border-0 bg-transparent transition-all duration-200"
            :class="
              activeKey === item.key
                ? 'side-nav-item--active text-primary-theme'
                : 'text-c-secondary hover:text-primary-theme'
            "
            :aria-current="activeKey === item.key ? 'page' : undefined"
            @click="handleUpdateValue(item.key)"
          >
            <component :is="item.icon" :size="18" class="flex-shrink-0" />
            <span>{{ item.label }}</span>
          </button>
        </nav>
      </div>
    </Transition>

    <!-- 「菜单」按钮（主色调 + Level 2: Nav 玻璃质感） -->
    <button
      class="group relative w-12 hover:w-36 h-12 rounded-xl cursor-pointer select-none outline-none overflow-hidden transition-all duration-300 ease-out active:scale-95"
      :style="{
        backgroundColor: 'color-mix(in srgb, var(--primary-color) 50%, transparent)',
        backdropFilter: 'blur(var(--glass-nav-blur))',
        WebkitBackdropFilter: 'blur(var(--glass-nav-blur))',
        border: 'var(--glass-nav-border)',
        color: '#fff',
        boxShadow: 'var(--shadow-glass-nav)',
      }"
      @click="sidebarCollapsed = !sidebarCollapsed"
    >
      <!-- 闭合态 -->
      <div
        class="absolute inset-0 flex items-center justify-center transition-all duration-300 ease-out opacity-100 scale-100 group-hover:opacity-0 group-hover:scale-75"
      >
        <Menu :size="24" color="#fff" />
      </div>
      <!-- 展开态 -->
      <div
        class="absolute inset-0 flex items-center gap-3 px-4 transition-all duration-300 ease-out opacity-0 scale-75 group-hover:opacity-100 group-hover:scale-100"
      >
        <Menu :size="24" color="#fff" class="flex-shrink-0" />
        <span class="text-[17px] font-medium whitespace-nowrap text-white">{{
          t("nav.menu")
        }}</span>
      </div>
    </button>

    <!-- 「启动游戏」按钮 + 预设选择悬停面板 -->
    <div class="relative" @mouseenter="onPresetEnter" @mouseleave="onPresetLeave">
      <!-- 预设选择面板（悬停时在按钮上方展开，Level 3: Floating 质感） -->
      <Transition name="preset">
        <div
          v-if="showPresetPanel"
          class="absolute left-0 z-20 w-52 glass-floating overflow-hidden"
          :style="{
            bottom: 'calc(100% + 8px)',
            border: 'var(--glass-floating-border)',
            boxShadow: 'var(--shadow-glass-floating)',
          }"
        >
          <div class="p-3">
            <Select
              :model-value="quickPresetId ?? undefined"
              :options="quickPresetOptions"
              size="sm"
              :disabled="quickPresetOptions.length === 0"
              @update:model-value="onPresetSelect"
            />
          </div>
        </div>
      </Transition>

      <button
        class="group relative w-12 hover:w-42 h-12 rounded-xl cursor-pointer select-none outline-none overflow-hidden transition-all duration-300 ease-out active:scale-95"
        :style="{
          backgroundColor: 'color-mix(in srgb, var(--color-bg-sidebar) 65%, transparent)',
          backdropFilter: 'blur(var(--glass-nav-blur))',
          WebkitBackdropFilter: 'blur(var(--glass-nav-blur))',
          border: 'var(--glass-nav-border)',
          color: 'var(--color-text-primary)',
          boxShadow: 'var(--shadow-glass-nav)',
        }"
        :disabled="launchingGame"
        @click="onLaunchGame"
      >
        <!-- 闭合态 -->
        <div
          class="absolute inset-0 flex items-center justify-center transition-all duration-300 ease-out opacity-100 scale-100 group-hover:opacity-0 group-hover:scale-75"
        >
          <span class="text-primary-theme flex items-center justify-center">
            <Play v-if="!launchingGame" :size="24" />
            <LoaderCircle v-else :size="24" class="animate-spin" />
          </span>
        </div>
        <!-- 展开态 -->
        <div
          class="absolute inset-0 flex items-center gap-3 px-4 transition-all duration-300 ease-out opacity-0 scale-75 group-hover:opacity-100 group-hover:scale-100"
        >
          <span class="text-primary-theme flex-shrink-0 flex items-center justify-center">
            <Play v-if="!launchingGame" :size="24" />
            <LoaderCircle v-else :size="24" class="animate-spin" />
          </span>
          <span class="text-[17px] font-medium whitespace-nowrap">{{ t("library.launch") }}</span>
        </div>
      </button>
    </div>
  </div>

  <!-- ============ 云存档差异确认弹窗 ============ -->
  <AppDialog
    :show="showLaunchMismatchDialog"
    @update:show="
      (v: boolean) => {
        if (!v) {
          showLaunchMismatchDialog = false
          launchingGame = false
        }
      }
    "
    width="440px"
  >
    <template #header>
      <div class="flex items-center gap-2">
        <AlertTriangle :size="18" class="text-amber-500" />
        <span class="font-semibold">{{ t("library.launchMismatch.title") }}</span>
      </div>
    </template>
    <div v-if="launchMismatchStatus" class="flex flex-col gap-2">
      <p class="text-sm text-c-secondary">{{ t("library.launchMismatch.warning") }}</p>
      <Alert variant="warning" material="glass">
        <template #icon><AlertTriangle :size="16" /></template>
        <div class="text-xs space-y-1">
          <div v-if="launchMismatchStatus.differentCount > 0">
            {{ t("saves.cloud.mismatch.different", { n: launchMismatchStatus.differentCount }) }}
          </div>
          <div v-if="launchMismatchStatus.localOnlyCount > 0">
            {{ t("saves.cloud.mismatch.localOnly", { n: launchMismatchStatus.localOnlyCount }) }}
          </div>
          <div v-if="launchMismatchStatus.cloudOnlyCount > 0">
            {{ t("saves.cloud.mismatch.cloudOnly", { n: launchMismatchStatus.cloudOnlyCount }) }}
          </div>
        </div>
      </Alert>
      <div class="flex justify-between mt-2 gap-2">
        <Button variant="secondary" size="sm" @click="handleGoToSaves">
          {{ t("library.launchMismatch.goToSaves") }}
        </Button>
        <Button variant="solid" size="sm" @click="onLaunchAnyway">
          {{ t("library.launchMismatch.forceLaunch") }}
        </Button>
      </div>
    </div>
  </AppDialog>

  <!-- ============ End AppDialogs ============ -->
</template>

<style scoped>
/* 菜单项左侧光源效果 —— 主题色自左向右渐变辉光 */
.side-nav-item:hover {
  background: linear-gradient(
    to right,
    color-mix(in srgb, var(--primary-color) 6%, transparent),
    transparent 80%
  );
}
.side-nav-item--active {
  background: linear-gradient(
    to right,
    color-mix(in srgb, var(--primary-color) 14%, transparent),
    transparent 65%
  );
}

/* 菜单面板展开/收起过渡动画 */
.menu-enter-active {
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.menu-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.menu-enter-from {
  opacity: 0;
  transform: translateY(12px);
}
.menu-leave-to {
  opacity: 0;
  transform: translateY(8px);
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

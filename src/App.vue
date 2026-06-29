<script setup lang="ts">
import { computed, ref, watch } from "vue"
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  NButton,
  NIcon,
  NCheckbox,
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
} from "naive-ui"
import { Minus, Square, X, PackageOpen } from "@lucide/vue"
import { currentLocale } from "./i18n"
import { naiveTheme, naiveThemeOverrides } from "./theme"
import { minimizeWindow, toggleMaximizeWindow, closeWindow } from "./utils/window"
import { useStorage } from "./composables/useStorage"
import { useAppUpdateCheck } from "./composables/useAppUpdateCheck"
import { useI18n } from "vue-i18n"
import { version as APP_VERSION } from "@/../package.json"
import SideNav from "./components/SideNav.vue"
import AppDialog from "./components/AppDialog.vue"
import { useRoute, useRouter } from "vue-router"

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const defaultPage = useStorage<string>("slaymgr:default-page", "")

// 自动更新检查
const {
  showDialog,
  latestVersion,
  downloadFromGithub,
  downloadFromNetdisk,
  closeDialog,
  disableAutoCheck,
  autoCheckOnStartup,
} = useAppUpdateCheck()

// "不再提示" — 勾选后只在关闭弹窗时生效
const dontRemind = ref(false)

// 弹窗打开时重置勾选状态
watch(showDialog, (val) => {
  if (val) dontRemind.value = false
})

function closeWithCheck() {
  if (dontRemind.value) {
    disableAutoCheck()
  } else {
    closeDialog()
  }
}

function handleLater() {
  closeWithCheck()
}

function handleDownloadNetdisk() {
  downloadFromNetdisk()
  closeWithCheck()
}

function handleDownloadGithub() {
  downloadFromGithub()
  closeWithCheck()
}

// 启动时重定向到默认页面 + 自动检查更新
router.isReady().then(() => {
  const saved = defaultPage.value
  if (saved) {
    const validPaths = router.getRoutes().map((r) => r.path)
    if (validPaths.includes(saved) && saved !== route.path) {
      router.replace(saved)
    }
  }
  // 在路由就绪后触发自动更新检查
  autoCheckOnStartup()
})

const naiveLocale = computed(() => (currentLocale.value === "zh-CN" ? zhCN : enUS))
const naiveDateLocale = computed(() => (currentLocale.value === "zh-CN" ? dateZhCN : dateEnUS))
const minimize = minimizeWindow
const toggleMaximize = toggleMaximizeWindow
const close = closeWindow
</script>

<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="naiveThemeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <NNotificationProvider>
      <NMessageProvider>
        <NDialogProvider>
          <!-- 根容器使用 CSS 变量实现动态主题 -->
          <div
            class="h-screen overflow-hidden relative"
            :style="{
              backgroundColor: 'var(--color-bg-primary)',
              color: 'var(--color-text-primary)',
            }"
          >
            <!-- 内容层（全高，标题栏叠加在上方） -->
            <div class="flex h-full pt-12 box-border">
              <SideNav />
              <main
                class="flex-1 overflow-auto px-16 pt-0 pb-6"
                :style="{
                  backgroundColor: 'var(--color-main-bg)',
                }"
              >
                <router-view v-slot="{ Component, route }">
                  <KeepAlive :include="['DiscoverPage']">
                    <component :is="Component" :key="route.name" />
                  </KeepAlive>
                </router-view>
              </main>
            </div>

            <!-- 沉浸标题栏（绝对定位叠加，毛玻璃效果） -->
            <div
              data-tauri-drag-region
              class="absolute top-0 left-0 right-0 h-12 z-30 flex items-center justify-between px-4 select-none"
              :style="{
                backdropFilter: 'blur(12px)',
                WebkitBackdropFilter: 'blur(12px)',
              }"
            >
              <div class="flex items-center gap-2.5">
                <svg viewBox="0 0 128 128" class="w-6 h-6 flex-shrink-0">
                  <defs>
                    <linearGradient id="titlelogo" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stop-color="var(--primary-color)"/>
                      <stop offset="100%" style="stop-color: var(--primary-600)"/>
                    </linearGradient>
                  </defs>
                  <rect width="128" height="128" rx="28" fill="url(#titlelogo)"/>
                  <text x="64" y="84" font-family="system-ui,sans-serif" font-size="58" font-weight="800" fill="#fff" text-anchor="middle">S</text>
                </svg>
                <div class="flex items-baseline gap-1.5">
                  <span class="text-base font-semibold text-c-primary">SlayMuManager</span>
                  <span class="text-xs text-c-muted">v{{ APP_VERSION }}</span>
                </div>
              </div>
              <div class="flex items-center -mr-2 h-full">
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="minimize">
                  <template #icon><NIcon :size="15"><Minus /></NIcon></template>
                </NButton>
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="toggleMaximize">
                  <template #icon><NIcon :size="13"><Square /></NIcon></template>
                </NButton>
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="close">
                  <template #icon><NIcon :size="17"><X /></NIcon></template>
                </NButton>
              </div>
            </div>

            <!-- 应用更新弹窗 -->
            <AppDialog
              :show="showDialog"
              width="400px"
              :mask-closable="false"
            >
              <template #header>
                <div class="flex items-center gap-2">
                  <NIcon :size="20" class="text-primary-theme"><PackageOpen /></NIcon>
                  <span class="font-semibold">{{ t("update.title") }}</span>
                </div>
              </template>

              <div class="space-y-4">
                <!-- 版本对比卡片 -->
                <div class="rounded-lg border border-c-default overflow-hidden">
                  <div class="flex items-center justify-between px-4 py-3 border-b border-c-default bg-c-muted/30">
                    <span class="text-xs text-c-muted font-medium">{{ t("update.currentVersion") }}</span>
                    <span class="text-xs font-semibold px-2.5 py-0.5 rounded-full bg-c-surface text-c-secondary border border-c-default">v{{ APP_VERSION }}</span>
                  </div>
                  <div class="flex items-center justify-between px-4 py-3">
                    <span class="text-xs text-c-muted font-medium">{{ t("update.latestVersion") }}</span>
                    <span class="text-xs font-semibold px-2.5 py-0.5 rounded-full bg-primary-theme/15 text-primary-theme">v{{ latestVersion }}</span>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <div class="flex items-center gap-2">
                  <NButton size="small" quaternary class="flex-1!" @click="handleLater">
                    {{ t("update.later") }}
                  </NButton>
                  <NButton size="small" quaternary class="flex-1!" @click="handleDownloadGithub">
                    <template #icon>
                      <NIcon :size="16">
                        <svg viewBox="0 0 256 249" fill="currentColor">
                          <path d="M127.505 0C57.095 0 0 57.085 0 127.505c0 56.336 36.534 104.13 87.196 120.99 6.372 1.18 8.712-2.766 8.712-6.134 0-3.04-.119-13.085-.173-23.739-35.473 7.713-42.958-15.044-42.958-15.044-5.8-14.738-14.157-18.656-14.157-18.656-11.568-7.914.872-7.752.872-7.752 12.804.9 19.546 13.14 19.546 13.14 11.372 19.493 29.828 13.857 37.104 10.6 1.144-8.242 4.449-13.866 8.095-17.05-28.32-3.225-58.092-14.158-58.092-63.014 0-13.92 4.981-25.295 13.138-34.224-1.324-3.212-5.688-16.18 1.235-33.743 0 0 10.707-3.427 35.073 13.07 10.17-2.826 21.078-4.242 31.914-4.29 10.836.048 21.752 1.464 31.942 4.29 24.337-16.497 35.029-13.07 35.029-13.07 6.94 17.563 2.574 30.531 1.25 33.743 8.175 8.929 13.122 20.303 13.122 34.224 0 48.972-29.828 59.756-58.22 62.912 4.573 3.957 8.648 11.717 8.648 23.612 0 17.06-.148 30.791-.148 34.991 0 3.393 2.295 7.369 8.759 6.117 50.634-16.879 87.122-64.656 87.122-120.973C255.009 57.085 197.922 0 127.505 0"/>
                        </svg>
                      </NIcon>
                    </template>
                    {{ t("update.github") }}
                  </NButton>
                  <NButton size="small" type="primary" class="flex-1!" @click="handleDownloadNetdisk">
                    <template #icon>
                      <NIcon :size="16">
                        <svg viewBox="40.722 40.484 943.271 938.508" fill="currentColor">
                          <path d="m469.135 976.134c-110.259-9.764-215.516-59.535-290.768-137.168-75.49-78.11-113.831-154.553-132.168-263.859-4.763-28.339-5.477-95.97-1.19-123.833 10.24-68.822 33.1-134.072 64.297-184.32 88.35-142.407 236.71-226.47 399.598-226.47 69.537 0 132.168 12.621 192.417 39.055 52.629 23.1 110.497 64.297 149.313 106.448 60.726 65.965 91.922 122.166 114.784 206.943 18.098 66.917 18.575 160.983 1.429 227.423-19.29 73.586-45.485 126.214-92.637 184.559-40.96 50.961-84.063 86.92-140.74 117.402-59.535 32.15-114.545 48.105-184.082 53.82-34.054 2.858-47.152 2.858-80.253 0zm84.063-238.616c11.669-5 20.718-19.051 20.718-32.625 0-23.338 4.525-49.771 10.478-61.44 12.146-23.814 28.339-32.149 77.396-39.77 19.05-2.857 38.578-6.905 43.341-8.81 13.574-5.954 24.767-17.385 32.149-33.34 6.668-14.526 6.906-15.955 6.906-53.105-.238-41.198-1.667-50.248-15.955-87.16-21.195-55.486-76.92-110.734-132.168-130.738-11.43-4.048-33.577-9.525-49.295-12.383-26.195-4.287-31.196-4.525-53.82-1.905-44.77 5.239-72.394 14.05-103.352 32.625-19.527 11.907-20.956 13.098-44.532 36.435-34.53 34.054-52.39 67.156-63.345 116.689-19.051 86.92 15.48 178.604 87.874 233.853 30.243 23.1 74.537 41.674 106.686 44.77 27.624 2.858 66.68 1.19 76.92-3.096z"/>
                        </svg>
                      </NIcon>
                    </template>
                    {{ t("update.downloadNetdisk") }}
                  </NButton>
                </div>

                <!-- 不再提示 -->
                <div class="flex justify-center pt-0.5">
                  <NCheckbox v-model:checked="dontRemind" size="small">
                    <span class="text-xs text-c-muted">{{ t("update.dontRemind") }}</span>
                  </NCheckbox>
                </div>
              </div>
            </AppDialog>
          </div>
        </NDialogProvider>
      </NMessageProvider>
    </NNotificationProvider>
  </NConfigProvider>
</template>

<style scoped>
.titlebar-btn:hover {
  background-color: color-mix(in srgb, var(--color-text-primary) 10%, transparent) !important;
}
</style>

<style>
.n-tooltip {
  max-width: 420px;
  max-height: 300px;
  overflow-y: auto;
  word-break: break-word;
}
</style>

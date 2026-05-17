<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSpace, NInput, NButton, NSelect, NIcon, NInputNumber, NRadioGroup, NRadio, NSwitch, useMessage } from "naive-ui"
import { FolderSearch, Key, Globe, ChevronDown } from "lucide-vue-next"
import { setLocale } from "../i18n"
import { displayMode, setDisplayMode, themeColorKey, setThemeColor, colorPalettes, type ThemeColorKey, type DisplayMode } from "../theme"
import type { AppBootstrap } from "../types"
import { version as APP_VERSION } from "../../package.json"

const { t } = useI18n()
const message = useMessage()

const languageOptions = computed(() => [
  { label: t("settings.language.zhCN"), value: "zh-CN" },
  { label: t("settings.language.en"), value: "en" },
])

const displayModeOptions = computed<{ label: string; value: DisplayMode }[]>(() => [
  { label: t("settings.appearance.displayModeSystem"), value: "system" },
  { label: t("settings.appearance.displayModeLight"), value: "light" },
  { label: t("settings.appearance.displayModeDark"), value: "dark" },
])

const themeColorOptions: { label: string; value: ThemeColorKey }[] = [
  { label: "靛蓝", value: "indigo" as const },
  { label: "蓝色", value: "blue" as const },
  { label: "绿色", value: "green" as const },
  { label: "紫色", value: "purple" as const },
  { label: "玫红", value: "rose" as const },
  { label: "橙色", value: "orange" as const },
  { label: "青色", value: "cyan" as const },
]

// --- 状态 ---
const gamePath = ref("")
const gameValid = ref(false)
const nexusKey = ref("")
const proxyUrl = ref("")
const locale = ref("zh-CN")
const backupCount = ref(5)
const launchMode = ref("steam")
const launchCheckCloudSave = ref(true)
const loading = ref(false)
const proxyTesting = ref(false)
const updateStatus = ref<"idle" | "checking" | "uptodate" | "available">("idle")
const latestVersion = ref("")
const updateUrl = ref("")

// --- 加载 ---
async function loadSettings() {
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    gamePath.value = bootstrap.gameDirectory ?? ""
    gameValid.value = bootstrap.gameDirectoryValid
    nexusKey.value = bootstrap.nexusApiKey ?? ""
    proxyUrl.value = bootstrap.proxyUrl ?? ""
    locale.value = bootstrap.locale
    setLocale(bootstrap.locale)
    backupCount.value = bootstrap.autoBackupKeepCount
    launchMode.value = bootstrap.launchMode
    launchCheckCloudSave.value = bootstrap.launchCheckCloudSave
  } catch (e: any) {
    // 静默处理
  }
}

// --- 游戏路径 ---
async function browseGamePath() {
  try {
    const path = await invoke<string | null>("pick_game_folder")
    if (path) {
      gamePath.value = path
      await saveGamePath()
    }
  } catch (e: any) {
    message.error(`${t("settings.error.saveFailed")}: ${e}`)
  }
}

async function detectGame() {
  loading.value = true
  try {
    const result = await invoke<{ rootDir: string; foundFrom: string } | null>("detect_game_install")
    if (result) {
      gamePath.value = result.rootDir
      gameValid.value = true
      const bootstrap = await invoke<AppBootstrap>("update_game_root_dir", { rootDir: result.rootDir })
      gameValid.value = bootstrap.gameDirectoryValid
      message.success(t("settings.success.gameDetected", { from: result.foundFrom }))
    } else {
      message.warning(t("settings.warning.gameNotFound"))
    }
  } catch (e: any) {
    message.error(t("settings.error.detectFailed", { e }))
  } finally {
    loading.value = false
  }
}

async function saveGamePath() {
  if (!gamePath.value.trim()) return
  try {
    const bootstrap = await invoke<AppBootstrap>("update_game_root_dir", { rootDir: gamePath.value.trim() })
    gameValid.value = bootstrap.gameDirectoryValid
    message.success(t("settings.success.gamePathUpdated"))
  } catch (e: any) {
    message.error(t("settings.error.saveFailed", { e }))
  }
}

// --- Nexus ---
const showNexusHelp = ref(false)
function openUrl(url: string) {
  invoke("open_url_in_browser", { url })
}

async function saveNexusKey() {
  try {
    await invoke("update_nexus_api_key", { apiKey: nexusKey.value })
    message.success(t("settings.success.apiKeySaved"))
  } catch (e: any) {
    message.error(t("settings.error.saveFailed", { e }))
  }
}

// --- 代理 ---
async function saveProxy() {
  try {
    await invoke("update_proxy_url", { url: proxyUrl.value || null })
    message.success(t("settings.success.proxySaved"))
  } catch (e: any) {
    message.error(t("settings.error.saveFailed", { e }))
  }
}

async function testProxy() {
  if (!proxyUrl.value.trim()) {
    message.warning(t("settings.warning.proxyUrlRequired"))
    return
  }
  proxyTesting.value = true
  try {
    const ok = await invoke<boolean>("test_proxy", { url: proxyUrl.value.trim() })
    message[ok ? "success" : "error"](ok ? t("settings.success.proxyOk") : t("settings.error.proxyFail"))
  } catch (e: any) {
    message.error(t("settings.error.testFailed", { e }))
  } finally {
    proxyTesting.value = false
  }
}

// --- 语言 ---
async function updateLocale(val: string) {
  locale.value = val
  setLocale(val)
  try {
    await invoke("update_app_locale", { locale: val })
  } catch {}
}

// --- 启动 ---
async function handleLaunchModeChange(val: string) {
  launchMode.value = val
  try {
    await invoke("update_launch_mode", { mode: val })
  } catch {}
}

async function handleLaunchCheckCloudSaveChange(val: boolean) {
  launchCheckCloudSave.value = val
  try {
    await invoke("update_launch_check_cloud_save", { check: val })
  } catch {}
}

// --- 显示模式 ---
async function handleDisplayModeChange(val: DisplayMode) {
  setDisplayMode(val)
  try {
    await invoke("update_theme_mode", { mode: val })
  } catch {}
}

// --- 主题色 ---
async function handleThemeColorChange(val: ThemeColorKey) {
  setThemeColor(val)
  try {
    await invoke("update_theme_color", { color: val })
  } catch {}
}

// --- 备份 ---
async function updateBackupCount(val: number | null) {
  if (val == null) return
  backupCount.value = val
  try {
    await invoke("update_auto_backup_keep_count", { count: val })
  } catch {}
}

onMounted(loadSettings)

// --- 版本比较 ---
function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map(Number)
  const pb = b.split(".").map(Number)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const na = pa[i] || 0
    const nb = pb[i] || 0
    if (na > nb) return 1
    if (na < nb) return -1
  }
  return 0
}

// --- 检测更新 ---
async function checkForUpdate() {
  updateStatus.value = "checking"
  try {
    const res = await fetch("https://api.github.com/repos/Summe-rsnow/SlayMuManager/releases/latest")
    if (!res.ok) throw new Error(`GitHub API: ${res.status}`)
    const data = await res.json()
    const tag: string = data.tag_name
    latestVersion.value = tag.replace(/^v/, "")
    updateUrl.value = data.html_url
    updateStatus.value = compareVersions(latestVersion.value, APP_VERSION) > 0 ? "available" : "uptodate"
  } catch {
    updateStatus.value = "idle"
  }
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-2xl font-bold" :style="{ color: 'var(--color-text-primary)' }">{{ t("settings.title") }}</h1>
    </div>

    <div class="max-w-2xl mx-auto flex flex-col gap-4">
      <!-- 游戏路径 -->
      <NCard :title="t('settings.gamePath.title')" size="small">
        <NSpace vertical>
          <div class="flex gap-2">
            <NInput
              v-model:value="gamePath"
              :placeholder="t('settings.gamePath.placeholder')"
              clearable
            >
              <template #prefix>
                <NIcon :size="16"><FolderSearch /></NIcon>
              </template>
            </NInput>
            <NButton secondary @click="browseGamePath">{{ t("settings.gamePath.browse") }}</NButton>
            <NButton secondary @click="saveGamePath">{{ t("common.save") }}</NButton>
          </div>
          <div class="flex items-center gap-2">
            <NButton secondary size="small" :loading="loading" @click="detectGame">
              {{ t("settings.gamePath.autoDetect") }}
            </NButton>
            <span v-if="gamePath" class="text-xs" :class="gameValid ? 'text-green-600' : 'text-red-500'">
              {{ gameValid ? t("settings.gamePath.valid") : t("settings.gamePath.invalid") }}
            </span>
          </div>
        </NSpace>
      </NCard>

      <!-- 启动 -->
      <NCard title="启动" size="small">
        <NSpace vertical>
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.launch.mode") }}</span>
            <NRadioGroup
              :value="launchMode"
              size="small"
              @update:value="handleLaunchModeChange"
            >
              <NRadio value="steam">{{ t("settings.launch.steam") }}</NRadio>
              <NRadio value="direct">{{ t("settings.launch.direct") }}</NRadio>
            </NRadioGroup>
          </div>
          <div class="border-t border-c-default"></div>
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.launch.checkCloudSave") }}</span>
            <NSwitch :value="launchCheckCloudSave" @update:value="handleLaunchCheckCloudSaveChange" />
          </div>
        </NSpace>
      </NCard>

      <!-- Nexus Mods -->
      <NCard :title="t('settings.nexus.title')" size="small">
        <NSpace vertical>
          <div class="flex gap-2">
            <NInput
              v-model:value="nexusKey"
              :placeholder="t('settings.nexus.apiKeyPlaceholder')"
              type="password"
              show-password-on="click"
            >
              <template #prefix>
                <NIcon :size="16"><Key /></NIcon>
              </template>
            </NInput>
            <NButton secondary @click="saveNexusKey">{{ t("common.save") }}</NButton>
          </div>
          <div class="flex items-center gap-2">
            <NButton text size="tiny" class="text-primary-theme" @click="showNexusHelp = !showNexusHelp">
              {{ t("settings.nexus.howToGet") }}
              <NIcon :size="12" class="ml-0.5 transition-transform" :class="showNexusHelp ? '' : '-rotate-90'"><ChevronDown /></NIcon>
            </NButton>
          </div>

          <!-- 展开说明 -->
          <div v-if="showNexusHelp" class="bg-primary-10-theme border border-primary-theme rounded-lg p-3 space-y-2 text-xs">
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">1</span>
              <span>{{ t("settings.nexus.help.step1") }} — </span>
              <NButton text size="tiny" class="text-primary-600-theme! underline! p-0!" @click="openUrl('https://www.nexusmods.com/')">nexusmods.com</NButton>
            </div>
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">2</span>
              <span>{{ t("settings.nexus.help.step2") }}</span>
            </div>
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">3</span>
              <span>{{ t("settings.nexus.help.step3") }} — </span>
              <NButton text size="tiny" class="text-primary-600-theme! underline! p-0!" @click="openUrl('https://www.nexusmods.com/users/myaccount?tab=api')">{{ t("settings.nexus.help.step3Btn") }}</NButton>
            </div>
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">4</span>
              <span>{{ t("settings.nexus.help.step4") }}</span>
            </div>
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">5</span>
              <span>{{ t("settings.nexus.help.step5") }}</span>
            </div>
            <div class="flex items-start gap-2">
              <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">6</span>
              <span>{{ t("settings.nexus.help.step6") }}</span>
            </div>
          </div>
        </NSpace>
      </NCard>

      <!-- 代理 -->
      <NCard :title="t('settings.proxy.title')" size="small">
        <NSpace vertical>
          <div class="flex gap-2">
            <NInput
              v-model:value="proxyUrl"
              placeholder="http://127.0.0.1:7890"
              clearable
            >
              <template #prefix>
                <NIcon :size="16"><Globe /></NIcon>
              </template>
            </NInput>
            <NButton secondary @click="saveProxy">{{ t("common.save") }}</NButton>
          </div>
          <NSpace>
            <NButton secondary size="small" :loading="proxyTesting" @click="testProxy">
              {{ t("settings.proxy.testConnection") }}
            </NButton>
          </NSpace>
        </NSpace>
      </NCard>

      <!-- 外观 -->
      <NCard :title="t('settings.appearance.title')" size="small">
        <NSpace vertical>
          <!-- 语言 -->
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.appearance.language") }}</span>
            <NSelect
              :options="languageOptions"
              :value="locale"
              style="width: 160px"
              size="small"
              @update:value="updateLocale"
            />
          </div>

          <!-- 分隔线 -->
          <div class="border-t border-c-default"></div>

          <!-- 显示模式 -->
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.appearance.displayMode") }}</span>
            <NRadioGroup
              :value="displayMode"
              size="small"
              @update:value="handleDisplayModeChange"
            >
              <NRadio
                v-for="opt in displayModeOptions"
                :key="opt.value"
                :value="opt.value"
              >{{ opt.label }}</NRadio>
            </NRadioGroup>
          </div>

          <!-- 分隔线 -->
          <div class="border-t border-c-default"></div>

          <!-- 主题色 -->
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.appearance.themeColor") }}</span>
            <div class="flex gap-1.5">
              <button
                v-for="opt in themeColorOptions"
                :key="opt.value"
                :title="opt.label"
                class="color-btn"
                :class="{ active: themeColorKey === opt.value }"
                :style="[{ backgroundColor: colorPalettes[opt.value].DEFAULT }]"
                @click="handleThemeColorChange(opt.value)"
              />
            </div>
          </div>
        </NSpace>
      </NCard>

      <!-- 备份 -->
      <NCard :title="t('settings.backup.title')" size="small">
        <div class="flex items-center justify-between">
          <span class="text-sm">{{ t("settings.backup.autoKeepCount") }}</span>
          <NInputNumber
            :value="backupCount"
            :min="1"
            :max="20"
            size="small"
            style="width: 120px"
            @update:value="updateBackupCount"
          />
        </div>
      </NCard>

      <!-- 关于 -->
      <NCard :title="t('settings.about.title')" size="small">
        <NSpace vertical>
          <div class="flex items-center justify-between">
            <span class="text-sm flex items-center gap-2">
              <svg class="w-5 h-5" viewBox="51.2 51.2 921.6 921.6" fill="currentColor">
                <path d="m729.329 373.95c-9.795-5.945-19.062-6.785-19.144-6.785l-1.065-.05c-57.2-3.866-121.165-5.832-190.126-5.832l-13.988.005c-68.956 0-132.925 1.96-190.12 5.831l-1.066.052c-.082 0-9.349.84-19.144 6.784-15.047 9.129-24.273 25.948-27.417 49.97-10.071 76.913-4.383 173.65.19 251.393 2.938 49.966 33.407 62.459 85.048 67.149 10.782.988 69.089 5.867 159.508 5.893v-.005c90.42-.02 148.726-4.905 159.514-5.888 51.64-4.69 82.11-17.183 85.043-67.15 4.577-77.741 10.26-174.479.19-251.391-3.15-24.028-12.376-40.848-27.423-49.977zm-390.99 172.718a23.65 23.65 0 0 1 -31.687-10.845 23.68 23.68 0 0 1 10.844-31.687c2.038-1.004 50.693-24.725 110.541-43.065a23.68 23.68 0 1 1 13.88 45.292c-56.294 17.25-103.111 40.074-103.577 40.305zm268.898 35.886c-.44 2.232-11.269 54.64-50.939 54.64-21.442 0-36.1-14.049-44.984-26.772-8.694 12.708-22.805 26.772-42.655 26.772-35.533 0-50.135-48.266-51.681-53.77a11.366 11.366 0 0 1 21.878-6.17c2.75 9.652 14.13 37.202 29.798 37.202 16.374 0 28.892-23.644 31.985-31.928a11.372 11.372 0 0 1 10.65-7.388h.06a11.376 11.376 0 0 1 10.63 7.506c.107.286 11.965 31.815 34.314 31.815 20.864 0 28.565-35.952 28.641-36.32a11.346 11.346 0 0 1 13.358-8.94 11.361 11.361 0 0 1 8.945 13.353zm110.116-46.736a23.68 23.68 0 0 1 -31.683 10.844c-.47-.23-47.472-23.116-103.572-40.31a23.69 23.69 0 0 1 -15.708-29.583 23.67 23.67 0 0 1 29.578-15.703c59.848 18.34 108.498 42.061 110.551 43.065a23.68 23.68 0 0 1 10.834 31.687z"/>
                <path d="m849.92 51.2h-675.84c-67.866 0-122.88 55.014-122.88 122.88v675.84c0 67.87 55.014 122.88 122.88 122.88h675.84c67.87 0 122.88-55.01 122.88-122.88v-675.84c0-67.86-55.01-122.88-122.88-122.88zm-36.603 627.45c-2.626 44.58-21.821 78.634-55.516 98.49-25.682 15.134-54.175 19.486-81.137 21.938-32.455 2.95-92.718 6.098-164.664 6.119-71.941-.02-132.209-3.164-164.664-6.119-26.962-2.452-55.455-6.804-81.132-21.939-33.695-19.855-52.89-53.903-55.51-98.483-4.706-80.133-10.574-179.855.194-262.108 10.654-81.383 70.102-104.976 100.612-106.168a2482.642 2482.642 0 0 1 81.423-4.086c-7.536-8.535-19.88-23.322-28.815-38.114-13.737-22.737 8.53-41.687 8.53-41.687s23.68-20.367 44.528 5.213c15.698 19.266 38.38 55.997 48.62 72.954l53.207-.215c13.26 0 26.332.072 39.22.215 10.24-16.957 32.92-53.683 48.619-72.954 20.843-25.58 44.528-5.213 44.528-5.213s22.262 18.95 8.525 41.687c-8.934 14.792-21.279 29.579-28.815 38.114 28.36.978 55.562 2.34 81.423 4.08 30.515 1.198 89.958 24.791 100.613 106.174 10.778 82.248 4.915 181.97.21 262.103z"/>
              </svg>
              {{ t("settings.about.bilibili") }}
            </span>
            <NButton text size="tiny" class="text-primary-theme" @click="openUrl('https://space.bilibili.com/5242608')">
              {{ t("settings.about.open") }}
            </NButton>
          </div>
          <div class="border-t border-c-default"></div>
          <div class="flex items-center justify-between">
            <span class="text-sm flex items-center gap-2">
              <svg class="w-5 h-5" viewBox="0 0 256 249" fill="currentColor">
                <path d="M127.505 0C57.095 0 0 57.085 0 127.505c0 56.336 36.534 104.13 87.196 120.99 6.372 1.18 8.712-2.766 8.712-6.134 0-3.04-.119-13.085-.173-23.739-35.473 7.713-42.958-15.044-42.958-15.044-5.8-14.738-14.157-18.656-14.157-18.656-11.568-7.914.872-7.752.872-7.752 12.804.9 19.546 13.14 19.546 13.14 11.372 19.493 29.828 13.857 37.104 10.6 1.144-8.242 4.449-13.866 8.095-17.05-28.32-3.225-58.092-14.158-58.092-63.014 0-13.92 4.981-25.295 13.138-34.224-1.324-3.212-5.688-16.18 1.235-33.743 0 0 10.707-3.427 35.073 13.07 10.17-2.826 21.078-4.242 31.914-4.29 10.836.048 21.752 1.464 31.942 4.29 24.337-16.497 35.029-13.07 35.029-13.07 6.94 17.563 2.574 30.531 1.25 33.743 8.175 8.929 13.122 20.303 13.122 34.224 0 48.972-29.828 59.756-58.22 62.912 4.573 3.957 8.648 11.717 8.648 23.612 0 17.06-.148 30.791-.148 34.991 0 3.393 2.295 7.369 8.759 6.117 50.634-16.879 87.122-64.656 87.122-120.973C255.009 57.085 197.922 0 127.505 0"/>
                <path d="M47.755 181.634c-.28.633-1.278.823-2.185.389-.925-.416-1.445-1.28-1.145-1.916.275-.652 1.273-.834 2.196-.396.927.415 1.455 1.287 1.134 1.923M54.027 187.23c-.608.564-1.797.302-2.604-.589-.834-.889-.99-2.077-.373-2.65.627-.563 1.78-.3 2.616.59.834.899.996 2.08.36 2.65M58.33 194.39c-.782.543-2.06.034-2.849-1.1-.781-1.133-.781-2.493.017-3.038.792-.545 2.05-.055 2.85 1.07.78 1.153.78 2.513-.019 3.069M65.606 202.683c-.699.77-2.187.564-3.277-.488-1.114-1.028-1.425-2.487-.724-3.258.707-.772 2.204-.555 3.302.488 1.107 1.026 1.445 2.496.7 3.258M75.01 205.483c-.307.998-1.741 1.452-3.185 1.028-1.442-.437-2.386-1.607-2.095-2.616.3-1.005 1.74-1.478 3.195-1.024 1.44.435 2.386 1.596 2.086 2.612M85.714 206.67c.036 1.052-1.189 1.924-2.705 1.943-1.525.033-2.758-.818-2.774-1.852 0-1.062 1.197-1.926 2.721-1.951 1.516-.03 2.758.815 2.758 1.86M96.228 206.267c.182 1.026-.872 2.08-2.377 2.36-1.48.27-2.85-.363-3.039-1.38-.184-1.052.89-2.105 2.367-2.378 1.508-.262 2.857.355 3.049 1.398"/>
              </svg>
              {{ t("settings.about.github") }}
            </span>
            <NButton text size="tiny" class="text-primary-theme" @click="openUrl('https://github.com/Summe-rsnow/SlayMuManager')">
              {{ t("settings.about.open") }}
            </NButton>
          </div>
          <div class="border-t border-c-default"></div>
          <div class="flex items-center justify-between">
            <span class="text-sm">{{ t("settings.about.checkUpdate") }}</span>
            <NButton
              v-if="updateStatus === 'idle'"
              text
              size="tiny"
              class="text-primary-theme"
              @click="checkForUpdate"
            >
              {{ t("settings.about.checkUpdate") }}
            </NButton>
            <span v-else-if="updateStatus === 'checking'" class="text-xs text-gray-400">
              {{ t("settings.about.checking") }}
            </span>
            <div v-else-if="updateStatus === 'uptodate'" class="flex items-center gap-2">
              <span class="text-xs text-green-600">v{{ APP_VERSION }} · {{ t("settings.about.upToDate") }}</span>
            </div>
            <div v-else class="flex items-center gap-2">
              <span class="text-xs text-primary-theme">{{ t("settings.about.newVersion", { ver: `v${latestVersion}` }) }}</span>
              <NButton text size="tiny" class="text-primary-theme" @click="openUrl(updateUrl)">
                {{ t("settings.about.open") }}
              </NButton>
            </div>
          </div>
        </NSpace>
      </NCard>
    </div>
  </div>
</template>

<style scoped>
.color-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}
.color-btn:hover {
  transform: scale(1.1);
}
.color-btn.active {
  border-color: white;
  box-shadow: 0 0 0 1.5px var(--primary-color);
  transform: scale(1.15);
}
</style>

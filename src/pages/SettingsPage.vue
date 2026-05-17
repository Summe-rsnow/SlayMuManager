<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSpace, NInput, NButton, NSelect, NIcon, NInputNumber, NRadioGroup, NRadio, NSwitch, useMessage } from "naive-ui"
import { FolderSearch, Key, Globe, ChevronDown } from "lucide-vue-next"
import { setLocale } from "../i18n"
import { displayMode, setDisplayMode, themeColorKey, setThemeColor, colorPalettes, type ThemeColorKey, type DisplayMode } from "../theme"
import type { AppBootstrap } from "../types"

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

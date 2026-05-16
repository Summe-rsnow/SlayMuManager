<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSpace, NInput, NButton, NSelect, NIcon, NInputNumber, useMessage } from "naive-ui"
import { FolderSearch, Key, Globe } from "lucide-vue-next"
import { setLocale } from "../i18n"
import type { AppBootstrap } from "../types"

const { t } = useI18n()
const message = useMessage()

const languageOptions = computed(() => [
  { label: t("settings.language.zhCN"), value: "zh-CN" },
  { label: t("settings.language.en"), value: "en" },
])

// --- 状态 ---
const gamePath = ref("")
const gameValid = ref(false)
const nexusKey = ref("")
const proxyUrl = ref("")
const locale = ref("zh-CN")
const backupCount = ref(5)
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
      <h1 class="text-2xl font-bold text-gray-800">{{ t("settings.title") }}</h1>
    </div>

    <div class="max-w-2xl flex flex-col gap-4">
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
          <p class="text-xs text-gray-400">
            {{ t("settings.nexus.hint") }}
          </p>
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

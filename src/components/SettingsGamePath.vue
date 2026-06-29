<script setup lang="ts">
import { ref, onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NInput, NButton, NIcon } from "naive-ui"
import { FolderSearch } from "@lucide/vue"
import type { AppBootstrap } from "../types"
import { useHighlightStore } from "@/stores/useHighlightStore"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()
const { highlightedSetting } = storeToRefs(useHighlightStore())

const gamePath = ref("")
const gameValid = ref(false)
const saving = ref(false)
const detecting = ref(false)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  gamePath.value = b.gameDirectory ?? ""
  gameValid.value = b.gameDirectoryValid
})

async function browseGamePath() {
  try {
    const path = await invoke<string | null>("pick_game_folder")
    if (path) {
      gamePath.value = path
      await saveGamePath()
    }
  } catch (e: unknown) {
    // 静默处理
  }
}

async function detectGame() {
  detecting.value = true
  try {
    const result = await invoke<{ rootDir: string; foundFrom: string } | null>("detect_game_install")
    if (result) {
      gamePath.value = result.rootDir
      gameValid.value = true
      const b = await invoke<AppBootstrap>("update_game_root_dir", { rootDir: result.rootDir })
      gameValid.value = b.gameDirectoryValid
    }
  } catch (e: unknown) {
    // 静默处理
  } finally {
    detecting.value = false
  }
}

async function saveGamePath() {
  saving.value = true
  try {
    const b = await invoke<AppBootstrap>("update_game_root_dir", { rootDir: gamePath.value.trim() })
    gameValid.value = b.gameDirectoryValid
  } catch (e: unknown) {
    // 静默处理
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <SettingsSection id="setting-game-path" :title="t('settings.gamePath.title')" :highlighted="highlightedSetting === 'game-path'">
    <div class="flex gap-2">
      <NInput v-model:value="gamePath" :placeholder="t('settings.gamePath.placeholder')" clearable>
        <template #prefix><NIcon :size="16"><FolderSearch /></NIcon></template>
      </NInput>
      <NButton secondary @click="browseGamePath">{{ t("settings.gamePath.browse") }}</NButton>
      <NButton secondary :loading="saving" @click="saveGamePath">{{ t("common.save") }}</NButton>
    </div>
    <div class="flex items-center gap-2">
      <NButton secondary size="small" :loading="detecting" @click="detectGame">
        {{ t("settings.gamePath.autoDetect") }}
      </NButton>
      <span v-if="gamePath" class="text-xs" :class="gameValid ? 'text-green-600' : 'text-red-500'">
        {{ gameValid ? t("settings.gamePath.valid") : t("settings.gamePath.invalid") }}
      </span>
    </div>
  </SettingsSection>
</template>

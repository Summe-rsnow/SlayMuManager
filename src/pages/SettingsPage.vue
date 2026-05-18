<script setup lang="ts">
import { useI18n } from "vue-i18n"
import SettingsGamePath from "../components/SettingsGamePath.vue"
import SettingsLaunch from "../components/SettingsLaunch.vue"
import SettingsNexus from "../components/SettingsNexus.vue"
import SettingsProxy from "../components/SettingsProxy.vue"
import SettingsAppearance from "../components/SettingsAppearance.vue"
import SettingsDiscover from "../components/SettingsDiscover.vue"
import SettingsBackup from "../components/SettingsBackup.vue"
import SettingsAbout from "../components/SettingsAbout.vue"

const { t } = useI18n()
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-c-primary">{{ t("settings.title") }}</h1>
    </div>

    <div class="max-w-2xl mx-auto flex flex-col gap-4">
      <SettingsGamePath />
      <SettingsLaunch />
      <SettingsNexus />
      <SettingsProxy />
      <SettingsAppearance />
      <SettingsDiscover />
      <SettingsBackup />
      <SettingsAbout />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
} from "naive-ui"
import { currentLocale } from "./i18n"
import { themeOverrides } from "./naive-theme"
import TitleBar from "./components/TitleBar.vue"
import SideNav from "./components/SideNav.vue"

const naiveLocale = computed(() => (currentLocale.value === "zh-CN" ? zhCN : enUS))
const naiveDateLocale = computed(() => (currentLocale.value === "zh-CN" ? dateZhCN : dateEnUS))
</script>

<template>
  <NConfigProvider :theme-overrides="themeOverrides" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <NNotificationProvider>
      <NMessageProvider>
        <NDialogProvider>
          <div class="h-screen flex flex-col bg-white text-gray-900 overflow-hidden">
            <TitleBar />
            <div class="flex flex-1 overflow-hidden">
              <SideNav />
              <main class="flex-1 overflow-auto p-6">
                <router-view />
              </main>
            </div>
          </div>
        </NDialogProvider>
      </NMessageProvider>
    </NNotificationProvider>
  </NConfigProvider>
</template>

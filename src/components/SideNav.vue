<script setup lang="ts">
import { h, computed } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { NMenu, NIcon, type MenuOption } from "naive-ui"
import { Library, Compass, FolderHeart, Save, Settings } from "lucide-vue-next"

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

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
    (o) => typeof o.key === "string" && route.path.startsWith(o.key) && o.key !== "/"
  )
  return prefix ? (prefix.key as string) : null
})

function handleUpdateValue(key: string) {
  router.push(key)
}
</script>

<template>
  <nav
    class="w-52 flex-shrink-0 flex flex-col p-3"
    :style="{
      borderRight: '1px solid var(--color-border)',
      backgroundColor: 'var(--color-bg-sidebar)',
    }"
  >
      <NMenu
        :value="activeKey"
        :options="menuOptions"
        :indent="16"
        :default-expanded-keys="[]"
        @update:value="handleUpdateValue"
      />
  </nav>
</template>

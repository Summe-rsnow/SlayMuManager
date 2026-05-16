<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NInput, NIcon, NSpace, NSelect, NPagination, useMessage,
} from "naive-ui"
import { Search, Download, ExternalLink, ThumbsUp, PackageOpen, ArrowDown } from "lucide-vue-next"
import type { RemoteMod, RemoteModSearchResult, ModFileInfo } from "../types"

const { t } = useI18n()
const message = useMessage()

// --- 排序选项 ---
const sortOptions = computed(() => [
  { label: t("discover.sort.latestAdded"), value: "latest_added" },
  { label: t("discover.sort.latestUpdated"), value: "latest_updated" },
  { label: t("discover.sort.trending"), value: "trending" },
  { label: t("discover.sort.downloads"), value: "downloads" },
])

// --- 状态 ---
const query = ref("")
const sortBy = ref("latest_added")
const results = ref<RemoteMod[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = 10
const loading = ref(false)
const initialLoading = ref(true)
const searched = ref(false)
const downloading = ref<Record<string, boolean>>({})

// --- 组件生命周期守卫（防止切换页面时异步回调卡死）---
const isActive = ref(true)
onBeforeUnmount(() => { isActive.value = false })

// --- 初始加载（空搜索浏览最新 Mod） ---
onMounted(() => doSearch())

async function doSearch(resetPage = true) {
  if (resetPage) page.value = 1
  loading.value = true
  searched.value = true
  try {
    const res = await invoke<RemoteModSearchResult>("search_remote_mods", {
      query: query.value.trim(),
      page: page.value,
      sortBy: sortBy.value,
    })
    if (!isActive.value) return
    results.value = res.items
    totalCount.value = res.totalCount
  } catch (e: any) {
    if (!isActive.value) return
    message.error(t("discover.error.searchFailed") + ": " + e)
    results.value = []
  } finally {
    if (isActive.value) {
      loading.value = false
      initialLoading.value = false
    }
  }
}

function onSortChange(val: string) {
  sortBy.value = val
  doSearch()
}

function onPageChange(p: number) {
  page.value = p
  doSearch(false)
}

// --- 下载 ---
async function downloadMod(mod: RemoteMod) {
  const modId = parseInt(mod.remoteId)
  if (isNaN(modId)) {
    message.error(t("discover.error.invalidModId"))
    return
  }

  downloading.value[mod.remoteId] = true
  try {
    // 1. 获取文件列表
    const files = await invoke<ModFileInfo[]>("get_nexus_mod_files", { modId })

    // 2. 选文件：优先主文件 + MAIN 分类
    const mainFile =
      files.find((f) => f.isPrimary && f.category === "MAIN") ??
      files.find((f) => f.category === "MAIN") ??
      files[0]

    if (!mainFile) {
      message.error(t("discover.error.noDownloadableFile"))
      return
    }

    // 3. 下载并安装
    await invoke("download_and_install_mod", {
      modId,
      fileId: mainFile.fileId,
      enableAfterInstall: true,
    })
    message.success(t("discover.success.installed", { name: mod.name }))
  } catch (e: any) {
    if (String(e).includes("免费用户") || String(e).includes("403")) {
      message.info(t("discover.info.downloadViaBrowser", { name: mod.name }))
      openModPage(mod.detailUrl)
    } else {
      message.error(t("discover.error.downloadFailed") + ": " + e)
    }
  } finally {
    downloading.value[mod.remoteId] = false
  }
}

function openModPage(url: string) {
  invoke("open_url_in_browser", { url }).catch(() => {})
}

function formatCount(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return n.toString()
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-800">{{ t("discover.title") }}</h1>
      <p class="text-sm text-gray-500 mt-1">{{ t("discover.subtitle") }}</p>
    </div>

    <!-- 搜索栏 + 排序 -->
    <div class="flex gap-2 mb-4">
      <NInput
        v-model:value="query"
        size="large"
        :placeholder="t('discover.searchPlaceholder')"
        clearable
        @keyup.enter="doSearch()"
      >
        <template #prefix>
          <NIcon :size="18"><Search /></NIcon>
        </template>
      </NInput>
      <NSelect
        :value="sortBy"
        :options="sortOptions"
        style="width: 140px"
        size="large"
        @update:value="onSortChange"
      />
      <NButton size="large" type="primary" :loading="loading" @click="doSearch()">
        {{ t("common.search") }}
      </NButton>
    </div>

    <!-- 初始加载骨架屏 -->
    <div v-if="initialLoading" class="grid grid-cols-1 gap-3">
      <NCard v-for="i in 5" :key="i" size="small">
        <div class="flex items-start gap-3 animate-pulse">
          <div class="w-16 h-16 rounded-lg bg-gray-200 flex-shrink-0" />
          <div class="flex-1 space-y-2">
            <div class="h-4 bg-gray-200 rounded w-2/3" />
            <div class="h-3 bg-gray-100 rounded w-full" />
            <div class="h-3 bg-gray-100 rounded w-1/2" />
          </div>
        </div>
      </NCard>
    </div>

    <!-- 结果列表 -->
    <div v-else-if="results.length > 0">
      <div class="text-sm text-gray-400 mb-3">
        {{ t("discover.resultCount", { total: totalCount }) }}
      </div>

      <div class="grid grid-cols-1 gap-3 mb-6">
        <NCard
          v-for="mod in results"
          :key="mod.remoteId"
          size="small"
          class="hover:shadow-md transition-shadow"
        >
          <div class="flex items-start gap-3">
            <!-- 缩略图 -->
            <div
              v-if="mod.pictureUrl"
              class="w-16 h-16 rounded-lg flex-shrink-0 overflow-hidden bg-gray-100"
            >
              <img
                :src="mod.pictureUrl"
                :alt="mod.name"
                class="w-full h-full object-cover"
                loading="lazy"
                referrerpolicy="no-referrer"
              />
            </div>
            <div
              v-else
              class="w-16 h-16 rounded-lg flex-shrink-0 bg-gray-100 flex items-center justify-center"
            >
              <NIcon :size="24" color="#9ca3af"><PackageOpen /></NIcon>
            </div>

            <!-- 信息 -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-semibold text-gray-800 truncate">{{ mod.name }}</span>
                <span v-if="mod.latestVersion" class="text-xs text-gray-400 font-mono">
                  v{{ mod.latestVersion }}
                </span>
              </div>
              <p v-if="mod.summary" class="text-xs text-gray-500 mb-2 line-clamp-2">
                {{ mod.summary }}
              </p>
              <div class="flex items-center gap-3 text-xs text-gray-400">
                <span>{{ mod.author ?? t("discover.unknownAuthor") }}</span>
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ThumbsUp /></NIcon>
                  {{ formatCount(mod.endorsementCount) }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ArrowDown /></NIcon>
                  {{ formatCount(mod.downloadCount) }}
                </span>
              </div>
            </div>

            <!-- 操作 -->
            <NSpace :size="4" class="flex-shrink-0">
              <NButton size="tiny" secondary @click="openModPage(mod.detailUrl)">
                <template #icon><NIcon :size="12"><ExternalLink /></NIcon></template>
                {{ t("discover.details") }}
              </NButton>
              <NButton
                size="tiny"
                type="primary"
                :loading="downloading[mod.remoteId]"
                @click="downloadMod(mod)"
              >
                <template #icon><NIcon :size="12"><Download /></NIcon></template>
                {{ t("discover.install") }}
              </NButton>
            </NSpace>
          </div>
        </NCard>
      </div>

      <!-- 分页 -->
      <div v-if="totalCount > pageSize" class="flex justify-center mb-8">
        <NPagination
          :page="page"
          :page-size="pageSize"
          :item-count="totalCount"
          @update:page="onPageChange"
        />
      </div>
    </div>

    <NCard v-else-if="searched && !loading" size="small">
      <div class="text-center py-12 text-gray-400">
        <NIcon :size="48" class="c-gray-300 mb-3"><Search /></NIcon>
        <p>{{ t("discover.empty.notFound") }}</p>
        <p class="text-sm mt-2">{{ t("discover.empty.needsApiKey") }}</p>
      </div>
    </NCard>

    <NCard v-else size="small">
      <div class="text-center py-12 text-gray-400">
        <NIcon :size="48" class="c-gray-300 mb-3"><PackageOpen /></NIcon>
        <p>{{ t("discover.empty.startSearch") }}</p>
        <p class="text-sm mt-1">{{ t("discover.empty.needsApiKey") }}</p>
      </div>
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted } from "vue"
import { onBeforeRouteLeave } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  NCard, NButton, NInput, NIcon, NSelect, NPagination, NInputNumber, NModal, NPopover, useMessage,
} from "naive-ui"
import { Search, ExternalLink, ThumbsUp, PackageOpen, ArrowDown, List, Languages } from "lucide-vue-next"
import type { RemoteMod, RemoteModSearchResult, AppBootstrap } from "../types"
import { useIsActive } from "../composables/useIsActive"
import { currentLocale } from "../i18n"
import { translateText, showTranslateQuotaTip } from "../composables/useTranslation"
import { discoverColumns } from "../composables/useDiscoverColumns"
import { prefetchEnabled, getPageCache, setPageCache } from "../composables/usePageCache"
import TruncatedText from "../components/TruncatedText.vue"

const { t } = useI18n()
const message = useMessage()
const { isActive } = useIsActive()

// --- 发现页列数映射 ---
const gridColsClass = computed(() => {
  const map: Record<number, string> = {
    1: "grid grid-cols-1 gap-4 mb-6 items-start",
    2: "grid grid-cols-2 gap-4 mb-6 items-start",
    3: "grid grid-cols-3 gap-4 mb-6 items-start",
    4: "grid grid-cols-4 gap-4 mb-6 items-start",
  }
  return map[discoverColumns.value] || map[3]
})
const skeletonColsClass = computed(() => {
  const map: Record<number, string> = {
    1: "grid grid-cols-1 gap-4 items-start",
    2: "grid grid-cols-2 gap-4 items-start",
    3: "grid grid-cols-3 gap-4 items-start",
    4: "grid grid-cols-4 gap-4 items-start",
  }
  return map[discoverColumns.value] || map[3]
})

// --- 排序选项 ---
const sortOptions = computed(() => [
  { label: t("discover.sort.latestAdded"), value: "latest_added" },
  { label: t("discover.sort.latestUpdated"), value: "latest_updated" },
  { label: t("discover.sort.trending"), value: "trending" },
  { label: t("discover.sort.downloads"), value: "downloads" },
])

// --- 每页条数选项 ---
const pageSizeOptions = [
  { label: "12", value: 12 },
  { label: "18", value: 18 },
  { label: "30", value: 30 },
  { label: "60", value: 60 },
]

// --- 状态 ---
const query = ref("")
const sortBy = ref("latest_added")
const results = ref<RemoteMod[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(12)
const loading = ref(false)
const initialLoading = ref(true)
const searched = ref(false)
const hasApiKey = ref(true)
const imageLoadFailed = ref<Record<string, boolean>>({})
function onImgError(modId: string) {
  imageLoadFailed.value = { ...imageLoadFailed.value, [modId]: true }
}

// --- 翻译 ---
const translatedTexts = reactive<Record<string, string>>({})
const translatingMods = reactive<Record<string, boolean>>({})
const showTranslation = reactive<Record<string, boolean>>({})

async function handleTranslate(mod: RemoteMod) {
  if (!mod.summary || translatingMods[mod.remoteId]) return
  translatingMods[mod.remoteId] = true
  try {
    const result = await translateText(mod.summary)
    if (result.ok) {
      translatedTexts[mod.remoteId] = result.text
      showTranslation[mod.remoteId] = true
    } else {
      console.warn("[discover] translate failed:", result.error)
      message.warning(t("discover.translateFailed") + ": " + result.error)
    }
  } finally {
    translatingMods[mod.remoteId] = false
  }
}

function toggleTranslation(modId: string) {
  showTranslation[modId] = !showTranslation[modId]
}

// --- 初始加载（空搜索浏览最新 Mod） ---
onMounted(async () => {
  try {
    const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
    hasApiKey.value = !!bootstrap.nexusApiKey
  } catch { /* ignore */ }
  doSearch()
})

async function doSearch(resetPage = true) {
  if (resetPage) page.value = 1
  searched.value = true
  const q = query.value.trim()
  const pg = page.value
  const ps = pageSize.value
  const sb = sortBy.value

  // 预取缓存命中：直接展示，不显示 loading
  if (prefetchEnabled.value) {
    const cached = getPageCache(q, sb, pg, ps)
    if (cached) {
      results.value = cached.items
      totalCount.value = cached.totalCount
      initialLoading.value = false
      return
    }
  }

  loading.value = true
  try {
    const res = await invoke<RemoteModSearchResult>("search_remote_mods", {
      query: q,
      page: pg,
      pageSize: ps,
      sortBy: sb,
    })
    if (!isActive.value) return
    results.value = res.items
    totalCount.value = res.totalCount

    // 写入缓存
    if (prefetchEnabled.value) {
      setPageCache(q, sb, pg, ps, res.items, res.totalCount)
      // 后台预取相邻页
      prefetchAdjacentPages(q, sb, pg, ps)
    }
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

/** 后台预取前后页数据（静默，不显示 loading） */
async function prefetchAdjacentPages(
  q: string, sb: string, currentPage: number, ps: number
) {
  const pages = [currentPage - 1, currentPage + 1]
  for (const p of pages) {
    if (p < 1) continue
    if (getPageCache(q, sb, p, ps)) continue // 已有缓存
    try {
      const res = await invoke<RemoteModSearchResult>("search_remote_mods", {
        query: q,
        page: p,
        pageSize: ps,
        sortBy: sb,
      })
      if (isActive.value) {
        setPageCache(q, sb, p, ps, res.items, res.totalCount)
      }
    } catch {
      // 预取失败静默忽略
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

function onPageSizeChange(val: number) {
  pageSize.value = val
  doSearch(true) // resetPage = true
}

const jumpPage = ref<number | null>(null)
const totalPages = computed(() => Math.ceil(totalCount.value / pageSize.value))

function jumpToPage() {
  const p = jumpPage.value
  if (p == null || p < 1 || p > totalPages.value) return
  onPageChange(p)
  jumpPage.value = null
}

const showImagePreview = ref(false)
const previewImageUrl = ref("")

function openImagePreview(url: string) {
  previewImageUrl.value = url
  showImagePreview.value = true
}

function openModPage(url: string) {
  invoke("open_url_in_browser", { url }).catch(() => {})
}

function formatCount(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return n.toString()
}

// 离开发现页时提前清空结果，避免大量图片拖慢页面切换
onBeforeRouteLeave(() => {
  results.value = []
  imageLoadFailed.value = {}
})

// 组件卸载时也清理
onUnmounted(() => {
  results.value = []
  imageLoadFailed.value = {}
})
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="mb-6">
      <h1 class="text-2xl font-bold" :style="{ color: 'var(--color-text-primary)' }">{{ t("discover.title") }}</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t("discover.subtitle") }}</p>
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

    <div class="flex-1">

    <!-- 初始加载骨架屏 -->
    <div v-if="initialLoading" :class="skeletonColsClass">
      <NCard v-for="i in pageSize" :key="i" :style="{ minHeight: '150px' }">
        <div class="flex gap-4 h-full animate-pulse">
          <div class="w-28 h-28 rounded-lg flex-shrink-0" :style="{ backgroundColor: 'var(--color-bg-secondary)' }" />
          <div class="flex-1 flex flex-col">
            <div class="h-5 w-2/3 rounded" :style="{ backgroundColor: 'var(--color-bg-secondary)' }" />
            <div class="flex-1 flex flex-col justify-end mt-2">
              <div class="h-4 rounded w-full" :style="{ backgroundColor: 'var(--color-bg-secondary)' }" />
              <div class="h-4 rounded w-3/4 mt-1" :style="{ backgroundColor: 'var(--color-bg-secondary)' }" />
            </div>
          </div>
        </div>
      </NCard>
    </div>

    <!-- 结果列表 -->
    <div v-else-if="results.length > 0">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm" :style="{ color: 'var(--color-text-muted)' }">
          {{ t("discover.resultCount", { total: totalCount }) }}
        </span>
        <span />
      </div>

      <div :class="gridColsClass">
        <NCard
          v-for="mod in results"
          :key="mod.remoteId"
          class="discover-card hover:shadow-md transition-shadow"
          :style="{ minHeight: '150px' }"
        >
          <div class="flex gap-4 h-full">
            <!-- 左侧：缩略图 -->
            <div
              v-if="mod.pictureUrl"
              class="w-28 h-28 rounded-lg flex-shrink-0 overflow-hidden" :style="{ backgroundColor: 'var(--color-bg-secondary)' }"
            >
              <img
                v-show="!imageLoadFailed[mod.remoteId]"
                :src="mod.pictureUrl"
                :alt="mod.name"
                class="w-full h-full object-cover cursor-pointer"
                loading="lazy"
                referrerpolicy="no-referrer"
                @error="onImgError(mod.remoteId)"
                @click="openImagePreview(mod.pictureUrl!)"
              />
              <div
                v-show="imageLoadFailed[mod.remoteId]"
                class="w-full h-full flex items-center justify-center"
              >
                <NIcon :size="32" :color="'var(--color-text-muted)'"><PackageOpen /></NIcon>
              </div>
            </div>
            <div
              v-else
              class="w-28 h-28 rounded-lg flex-shrink-0 flex items-center justify-center"
              :style="{ backgroundColor: 'var(--color-bg-secondary)' }"
            >
              <NIcon :size="32" :color="'var(--color-text-muted)'"><PackageOpen /></NIcon>
            </div>

            <!-- 右侧：标题 + 说明 + 统计 -->
            <div class="flex-1 flex flex-col min-w-0">
              <!-- 标题 + 版本号 + 跳转按钮 -->
              <div class="flex items-start justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0 flex-1">
                  <NPopover v-if="mod.name" trigger="hover" placement="top" :width="320">
                    <template #trigger>
                      <span class="font-semibold text-base truncate cursor-help" :style="{ color: 'var(--color-text-primary)' }">{{ mod.name }}</span>
                    </template>
                    <div class="text-xs leading-relaxed break-words max-w-xs">{{ mod.name }}</div>
                  </NPopover>
                  <span v-if="mod.latestVersion" class="text-xs font-mono flex-shrink-0" :style="{ color: 'var(--color-text-muted)' }">
                    v{{ mod.latestVersion }}
                  </span>
                </div>
                <NButton size="small" secondary class="flex-shrink-0" @click="openModPage(mod.detailUrl)">
                  <template #icon><NIcon :size="13"><ExternalLink /></NIcon></template>
                  {{ t("discover.details") }}
                </NButton>
              </div>

              <!-- 说明 + 翻译（内容撑开） -->
              <div class="min-h-0 mt-2">
                <TruncatedText :text="mod.summary" />

                <!-- 翻译区域（仅中文用户） -->
                <div v-if="mod.summary && currentLocale === 'zh-CN'" class="flex flex-wrap items-start gap-x-1.5 mt-1">
                  <!-- 已翻译：切换按钮 + 译文 -->
                  <template v-if="translatedTexts[mod.remoteId]">
                    <button
                      class="translate-toggle"
                      @click="toggleTranslation(mod.remoteId)"
                    >
                      <NIcon :size="12"><Languages /></NIcon>
                      {{ showTranslation[mod.remoteId] ? t("discover.showOriginal") : t("discover.translate") }}
                    </button>
                    <p
                      v-if="showTranslation[mod.remoteId]"
                      class="text-xs leading-relaxed w-full mt-0.5"
                      :style="{ color: 'var(--color-text-secondary)' }"
                    >
                      {{ translatedTexts[mod.remoteId] }}
                    </p>
                  </template>
                  <!-- 翻译中 -->
                  <span v-else-if="translatingMods[mod.remoteId]" class="text-xs" :style="{ color: 'var(--color-text-muted)' }">
                    {{ t("discover.translating") }}
                  </span>
                  <!-- 未翻译：显示翻译按钮（带配额提示） -->
                  <NPopover v-else trigger="hover" placement="top" :width="240" :disabled="!showTranslateQuotaTip">
                    <template #trigger>
                      <button
                        class="translate-toggle"
                        @click="handleTranslate(mod)"
                      >
                        <NIcon :size="12"><Languages /></NIcon>
                        {{ t("discover.translate") }}
                      </button>
                    </template>
                    <span class="text-xs">{{ t("discover.translateQuota") }}</span>
                  </NPopover>
                </div>
              </div>

              <!-- 统计：靠底部 -->
              <div class="flex items-center gap-3 text-xs pt-2 mt-auto" :style="{ color: 'var(--color-text-muted)' }">
                <span>{{ mod.author ?? t("discover.unknownAuthor") }}</span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ThumbsUp /></NIcon>
                  {{ formatCount(mod.endorsementCount) }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ArrowDown /></NIcon>
                  {{ formatCount(mod.downloadCount) }}
                </span>
              </div>
            </div>
          </div>
        </NCard>
      </div>
    </div>

    <NCard v-else-if="searched && !loading" size="small">
      <div class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
        <NIcon :size="48" class="mb-3" :color="'var(--color-text-muted)'"><Search /></NIcon>
        <p>{{ t("discover.empty.notFound") }}</p>
        <p v-if="!hasApiKey" class="text-sm mt-2">{{ t("discover.empty.needsApiKey") }}</p>
      </div>
    </NCard>

    <NCard v-else size="small">
      <div class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
        <NIcon :size="48" class="mb-3" :color="'var(--color-text-muted)'"><PackageOpen /></NIcon>
        <p>{{ t("discover.empty.startSearch") }}</p>
        <p v-if="!hasApiKey" class="text-sm mt-1">{{ t("discover.empty.needsApiKey") }}</p>
      </div>
    </NCard>
    </div>

    <!-- 分页 + 每页条数（浮动药丸） -->
    <div v-if="!initialLoading && results.length > 0 && totalCount > pageSize" class="flex justify-center sticky bottom-0 z-10 py-4">
      <div class="discover-pagination-bar flex items-center gap-3 px-5 py-2.5 rounded-full border shadow-lg backdrop-blur-xl"
        :style="{
          backgroundColor: 'color-mix(in srgb, var(--color-bg-primary) 70%, transparent)',
          borderColor: 'var(--color-border)',
        }"
      >
        <div class="flex items-center gap-1.5">
          <NIcon :size="14" :style="{ color: 'var(--color-text-muted)' }"><List /></NIcon>
          <NSelect
            :value="pageSize"
            :options="pageSizeOptions"
            style="width: 80px"
            size="tiny"
            @update:value="onPageSizeChange"
          />
        </div>
        <div class="discover-pagination">
        <NPagination
          :page="page"
          :page-size="pageSize"
          :item-count="totalCount"
          @update:page="onPageChange"
          size="small"
        />
        </div>
        <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ t("discover.jumpTo") }}</span>
        <NInputNumber
          v-model:value="jumpPage"
          size="tiny"
          :min="1"
          :max="totalPages"
          :placeholder="String(page)"
          style="width: 70px"
          @keyup.enter="jumpToPage"
        />
        <NButton size="tiny" secondary @click="jumpToPage">{{ t("discover.jumpToBtn") }}</NButton>
      </div>
    </div>

    <!-- 图片放大预览 -->
    <NModal :show="showImagePreview" @update:show="(v: boolean) => !v && (showImagePreview = false)">
      <div
        class="flex items-center justify-center"
        style="max-width: 90vw; max-height: 90vh;"
        @click="showImagePreview = false"
      >
        <img
          v-if="previewImageUrl"
          :src="previewImageUrl"
          class="max-w-full max-h-[85vh] rounded-lg shadow-2xl object-contain"
          style="max-width: 85vw;"
        />
      </div>
    </NModal>
  </div>
</template>

<style scoped>
.discover-card {
  --n-border-color: color-mix(in srgb, var(--color-border), var(--color-text-muted) 50%);
}

/* 翻译切换按钮 */
.translate-toggle {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: 0.7rem;
  line-height: 1;
  padding: 1px 5px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  color: var(--primary-color);
  background-color: color-mix(in srgb, var(--primary-color) 8%, transparent);
  transition: background-color 0.15s;
  white-space: nowrap;
}
.translate-toggle:hover {
  background-color: color-mix(in srgb, var(--primary-color) 18%, transparent);
}

/* 分页按钮药丸形状 */
.discover-pagination :deep(.n-pagination-item) {
  border-radius: 9999px !important;
}
.discover-pagination :deep(.n-pagination-item.n-pagination-item--active) {
  border-radius: 9999px !important;
}
</style>

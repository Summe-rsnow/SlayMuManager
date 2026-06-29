<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue"
import { storeToRefs } from "pinia"
import { useRouter, useRoute } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import {
  NButton, NInput, NIcon, NSelect, NPopconfirm, useMessage, useDialog,
} from "naive-ui"
import { Search, ExternalLink, ThumbsUp, ArrowDown, PackageOpen, RefreshCw, Plus, X } from "@lucide/vue"
import type { RemoteMod, RemoteModSearchResult, AppBootstrap, WorkshopMod, WorkshopSearchResult } from "../types"
import { useIsActive } from "@/composables/useIsActive"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import { useSearchPrefetch } from "@/composables/useSearchPrefetch"
import { useHighlightStore } from "@/stores/useHighlightStore"
import { useModCacheStore } from "@/stores/useModCacheStore"
import EmptyState from "@/components/EmptyState.vue"
import DiscoverPagination from "@/components/DiscoverPagination.vue"
import DiscoverCard from "@/components/DiscoverCard.vue"

defineOptions({ name: "DiscoverPage" })

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const router = useRouter()
const route = useRoute()
const highlightStore = useHighlightStore()
const { isActive } = useIsActive()
const { fetchMods } = useModCacheStore()
const prefStore = useDiscoverPrefStore()
const { discoverColumns, prefetchEnabled } = storeToRefs(prefStore)

// --- 预取缓存（Nexus + Workshop 共用泛型 composable）---
const nexusCache = useSearchPrefetch<RemoteMod>("nx")
const workshopCache = useSearchPrefetch<WorkshopMod>("ws")

// --- 发现页列数映射 ---
const gridColsClass = computed(() => {
  const map: Record<number, string> = {
    1: "columns-1 gap-4 mb-6",
    2: "columns-2 gap-4 mb-6",
    3: "columns-3 gap-4 mb-6",
    4: "columns-4 gap-4 mb-6",
  }
  return map[discoverColumns.value] || map[3]
})
const skeletonColsClass = computed(() => {
  const map: Record<number, string> = {
    1: "columns-1 gap-4",
    2: "columns-2 gap-4",
    3: "columns-3 gap-4",
    4: "columns-4 gap-4",
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

// --- 状态 ---
const tab = ref(route.query.tab === "workshop" ? "workshop" : "nexus")
const isNexus = computed(() => tab.value === "nexus")
const query = ref("")
const sortBy = ref("latest_added")
const results = ref<RemoteMod[]>([])
const totalCount = ref(0)
const page = ref(1)
const pageSize = ref(12)

// --- 创意工坊状态 ---
const workshopQuery = ref("")
const workshopSortBy = ref("latest_added")
const workshopResults = ref<WorkshopMod[]>([])
const workshopLoading = ref(false)
const workshopSearched = ref(false)
const workshopPage = ref(1)
const workshopPageSize = ref(12)
const workshopTotalCount = ref(0)
const subscribingWorkshop = ref(new Set<number>())
const unsubscribingWorkshop = ref(new Set<number>())

function switchNexusTab() {
  tab.value = "nexus"
  const q = { ...route.query }
  delete q.tab
  router.replace({ query: q })
}

function switchWorkshopTab() {
  tab.value = "workshop"
  router.replace({ query: { ...route.query, tab: "workshop" } })
  if (!workshopSearched.value) {
    searchWorkshop()
  }
}

/** 静默获取一页（仅写入缓存，不更新 UI） */
async function workshopFetchSilent(query: string, sortBy: string, page: number, pageSize: number) {
  try {
    const result = await invoke<WorkshopSearchResult>("search_workshop", { query, page, pageSize, sortBy })
    workshopCache.setCache(query, sortBy, page, pageSize, result.items, result.totalCount)
  } catch { /* silent */ }
}

/** 后台预取 Workshop 相邻页 */
function workshopPrefetchAdjacent(q: string, sb: string, currentPage: number, ps: number) {
  workshopCache.prefetchAdjacent(q, sb, currentPage, ps, workshopFetchSilent)
}

async function subscribeToWorkshop(id: number) {
  subscribingWorkshop.value = new Set(subscribingWorkshop.value).add(id)
  try {
    await invoke("subscribe_workshop_mod", { publishedFileId: id })
    const mod = workshopResults.value.find(m => m.id === id)
    if (mod) mod.subscribed = true
  } catch (e: unknown) {
    console.error("Workshop subscribe error:", e)
    message.error(String(e))
  } finally {
    const next = new Set(subscribingWorkshop.value)
    next.delete(id)
    subscribingWorkshop.value = next
  }
}

async function unsubscribeFromWorkshop(id: number) {
  unsubscribingWorkshop.value = new Set(unsubscribingWorkshop.value).add(id)
  try {
    await invoke("unsubscribe_workshop_mod", { publishedFileId: id })
    const mod = workshopResults.value.find(m => m.id === id)
    if (mod) mod.subscribed = false
    message.success(t("library.mod.unsubscribed"))
    await fetchMods()
  } catch (e: unknown) {
    console.error("Workshop unsubscribe error:", e)
    message.error(String(e))
  } finally {
    const next = new Set(unsubscribingWorkshop.value)
    next.delete(id)
    unsubscribingWorkshop.value = next
  }
}

// 请求计数器：递增 ID 匹配事件，忽略过期响应
const searchReqId = ref(0)

// --- 每页条数选项（基于列数 n: 4n, 6n, 10n, 20n）---
const PAGE_SIZE_MULTIPLIERS = [4, 6, 10, 20] as const
const pageSizeOptions = computed(() => {
  const n = discoverColumns.value
  return PAGE_SIZE_MULTIPLIERS.map(s => ({ label: `${s * n}`, value: s * n }))
})
// 列数变化时复位 pageSize 到第一个合法值（immediate 确保跨页面切换后也生效）
watch(discoverColumns, (n) => {
  const valid = PAGE_SIZE_MULTIPLIERS.map(s => s * n)
  if (!valid.includes(pageSize.value)) {
    pageSize.value = valid[0]
  }
}, { immediate: true })
const loading = ref(false)
const initialLoading = ref(true)
const searched = ref(false)
const hasApiKey = ref(true)

// --- 后台搜索事件 ---

interface DiscoverSearchEvent {
  reqId: number
  success: boolean
  error: string | null
  query: string
  page: number
  pageSize: number
  sortBy: string
  result: RemoteModSearchResult | null
}

let unlistenSearch: UnlistenFn | null = null

// 启动后台搜索（不 await，结果由事件驱动）
function startSearch() {
  const q = query.value.trim()
  const pg = page.value
  const ps = pageSize.value
  const sb = sortBy.value

  // 递增 reqId，前一次请求的响应会被忽略
  searchReqId.value++
  const currentReqId = searchReqId.value

  loading.value = true

  invoke("start_remote_search", {
    query: q,
    page: pg,
    pageSize: ps,
    sortBy: sb,
    reqId: currentReqId,
  }).catch((e: unknown) => {
    if (!isActive.value) return
    loading.value = false
    initialLoading.value = false
    message.error(t("discover.error.searchFailed") + ": " + String(e))
  })

  // 后台预取相邻页（使用非匹配 reqId，事件只缓存不更新 UI）
  if (prefetchEnabled.value) {
    prefetchAdjacentPages(q, sb, pg, ps, currentReqId)
  }
}

// 处理搜索结果事件
function handleSearchEvent(event: DiscoverSearchEvent) {
  if (!isActive.value) return

  // 无论是否匹配 reqId，都写入缓存（这样预取页也能被缓存）
  if (event.result) {
    nexusCache.setCache(event.query, event.sortBy, event.page, event.pageSize, event.result.items, event.result.totalCount)
  }

  // 仅匹配当前请求 ID 才更新 UI
  if (event.reqId !== searchReqId.value) return

  loading.value = false
  initialLoading.value = false

  if (event.success && event.result) {
    results.value = event.result.items
    totalCount.value = event.result.totalCount
  } else {
    message.error(t("discover.error.searchFailed") + ": " + (event.error ?? "unknown"))
    results.value = []
  }
}

async function doSearch(resetPage = true) {
  if (resetPage) page.value = 1
  searched.value = true
  const q = query.value.trim()
  const pg = page.value
  const ps = pageSize.value
  const sb = sortBy.value

  // 缓存命中：直接展示
  if (prefetchEnabled.value) {
    const cached = nexusCache.getCache(q, sb, pg, ps)
    if (cached) {
      results.value = cached.items
      totalCount.value = cached.totalCount
      initialLoading.value = false
      // 即使有缓存，也启动后台刷新（带上当前 reqId，后续事件会更新 UI）
      startSearch()
      return
    }
  }

  // 缓存未命中：启动后台搜索
  startSearch()
}

/** 后台预取前后页数据（使用 reqId=0 避免 UI 更新） */
function prefetchAdjacentPages(
  q: string, sb: string, currentPage: number, ps: number, _mainReqId: number
) {
  nexusCache.prefetchAdjacent(q, sb, currentPage, ps, (query, sortBy, p, pageSize) => {
    invoke("start_remote_search", {
      query, page: p, pageSize, sortBy, reqId: 0,
    }).catch(() => {})
  })
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
  doSearch(true)
}

function onWorkshopPageChange(p: number) {
  workshopPage.value = p
  searchWorkshop(p)
}

function onWorkshopPageSizeChange(val: number) {
  workshopPageSize.value = val
  workshopPage.value = 1
  searchWorkshop(1)
}

function onWorkshopSortChange(val: string) {
  workshopSortBy.value = val
  workshopPage.value = 1
  searchWorkshop(1)
}

function refreshWorkshop() {
  searchWorkshop(workshopPage.value, true)
}

async function searchWorkshop(pg?: number, ignoreCache?: boolean) {
  workshopLoading.value = true
  workshopSearched.value = true
  const pageNum = pg ?? workshopPage.value
  const q = workshopQuery.value
  const sb = workshopSortBy.value
  const ps = workshopPageSize.value

  // 缓存命中且非强制刷新：直接展示，后台静默刷新
  if (!ignoreCache && prefetchEnabled.value) {
    const cached = workshopCache.getCache(q, sb, pageNum, ps)
    if (cached) {
      workshopResults.value = cached.items
      workshopTotalCount.value = cached.totalCount
      workshopPage.value = pageNum
      workshopLoading.value = false
      workshopFetchSilent(q, sb, pageNum, ps)
      return
    }
  }

  try {
    const result = await invoke<WorkshopSearchResult>("search_workshop", {
      query: q,
      page: pageNum,
      pageSize: ps,
      sortBy: sb,
    })
    workshopResults.value = result.items
    workshopTotalCount.value = result.totalCount
    workshopPage.value = pageNum
    workshopCache.setCache(q, sb, pageNum, ps, result.items, result.totalCount)

    if (prefetchEnabled.value) {
      workshopPrefetchAdjacent(q, sb, pageNum, ps)
    }
  } catch (e: unknown) {
    console.error("Workshop search error:", e)
    message.error(String(e))
  } finally {
    workshopLoading.value = false
  }
}

const showPagination = computed(() => {
  if (isNexus.value) {
    return !initialLoading.value && results.value.length > 0 && totalCount.value > pageSize.value
  }
  return workshopResults.value.length > 0 && workshopTotalCount.value > workshopResults.value.length
})

function openModPage(url: string) {
  invoke("open_url_in_browser", { url }).catch(() => {})
}

function formatCount(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return n.toString()
}

// --- 生命周期 ---

onMounted(() => {
  // 设置后台搜索事件监听
  listen<DiscoverSearchEvent>("slaymgr:discover-search-result", (event) => {
    handleSearchEvent(event.payload)
  }).then((fn) => {
    unlistenSearch = fn
  })

  // 获取 bootstrap（API key 信息，不阻塞搜索）
  invoke<AppBootstrap>("get_app_bootstrap").then((bootstrap) => {
    if (isActive.value) {
      hasApiKey.value = !!bootstrap.nexusApiKey
    }
  }).catch(() => {})

  // 立即触发搜索
  doSearch()
})

// 组件销毁前取消事件监听
onBeforeUnmount(() => {
  unlistenSearch?.()
})

// --- 用户操作入口：未配置 API Key 时弹窗引导 ---
const showApiKeyDialog = () => {
  dialog.warning({
    title: t("settings.prompt.apiKeyRequired"),
    content: t("settings.prompt.apiKeyRequiredDesc"),
    positiveText: t("settings.prompt.goToSettings"),
    negativeText: t("common.cancel"),
    onPositiveClick: () => {
      highlightStore.highlight("nexus")
      router.push("/settings")
    },
    maskClosable: true,
  })
}

function handleUserSearch() {
  if (!hasApiKey.value) {
    showApiKeyDialog()
    return
  }
  doSearch(true)
}

function handleUserRefresh() {
  if (!hasApiKey.value) {
    showApiKeyDialog()
    return
  }
  doSearch(true)
}

</script>

<template>
  <div class="flex flex-col h-full">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-c-primary">{{ t("discover.title") }}</h1>
      <p class="text-sm mt-1 text-c-secondary">{{ t("discover.subtitle") }}</p>
      <div class="flex gap-1 mt-3">
        <NButton size="small" :type="tab === 'nexus' ? 'primary' : 'default'" @click="switchNexusTab">Nexus</NButton>
        <NButton size="small" :type="tab === 'workshop' ? 'primary' : 'default'" @click="switchWorkshopTab">
          {{ t("discover.workshop.tab") }}
        </NButton>
      </div>
    </div>

    <!-- Nexus 搜索 -->
    <template v-if="isNexus">
    <!-- 搜索栏 + 排序 + 刷新 -->
    <div class="flex gap-2 mb-4">
      <NInput
        v-model:value="query"
        size="large"
        :placeholder="t('discover.searchPlaceholder')"
        clearable
        @keyup.enter="handleUserSearch()"
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
      <NButton size="large" type="primary" :loading="loading" @click="handleUserSearch()">
        {{ t("common.search") }}
      </NButton>
      <NButton size="large" secondary :loading="loading" @click="handleUserRefresh">
        <template #icon><NIcon :size="18"><RefreshCw /></NIcon></template>
        {{ t("common.refresh") }}
      </NButton>
    </div>

    <div class="flex-1">

    <!-- 初始加载骨架屏 -->
    <div v-if="initialLoading" :class="skeletonColsClass">
      <NCard v-for="i in pageSize" :key="i" class="break-inside-avoid mb-4" :style="{ minHeight: '150px' }">
        <div class="flex gap-4 h-full animate-pulse">
          <div class="w-28 h-28 rounded-lg flex-shrink-0 bg-c-secondary" />
          <div class="flex-1 flex flex-col">
            <div class="h-5 w-2/3 rounded bg-c-secondary" />
            <div class="flex-1 flex flex-col justify-end mt-2">
              <div class="h-4 rounded w-full bg-c-secondary" />
              <div class="h-4 rounded w-3/4 mt-1 bg-c-secondary" />
            </div>
          </div>
        </div>
      </NCard>
    </div>

    <!-- 结果列表 -->
    <div v-else-if="results.length > 0">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm text-c-muted">
          {{ t("discover.resultCount", { total: totalCount }) }}
        </span>
        <span />
      </div>

      <div :class="gridColsClass">
        <DiscoverCard
          v-for="mod in results"
          :key="mod.remoteId"
          :name="mod.name"
          :image-url="mod.pictureUrl"
          :description="mod.summary"
          :author="mod.author ?? t('discover.unknownAuthor')"
          :version="mod.latestVersion"
          :stats="[
            { icon: ThumbsUp, value: formatCount(mod.endorsementCount) },
            { icon: ArrowDown, value: formatCount(mod.downloadCount) },
          ]"
        >
          <template #actions>
            <NButton size="small" secondary class="flex-shrink-0" @click="openModPage(mod.detailUrl)">
              <template #icon><NIcon :size="13"><ExternalLink /></NIcon></template>
              {{ t("discover.details") }}
            </NButton>
          </template>
        </DiscoverCard>
      </div>
    </div>

    <EmptyState v-else-if="searched && !loading" :icon="Search" :title="t('discover.empty.notFound')" :description="hasApiKey ? undefined : t('discover.empty.needsApiKey')" bordered />

    <EmptyState v-else :icon="PackageOpen" :title="t('discover.empty.startSearch')" :description="hasApiKey ? undefined : t('discover.empty.needsApiKey')" bordered />
    </div>
    </template>

    <!-- Steam 创意工坊 -->
    <div v-show="tab === 'workshop'">
      <div class="flex gap-2 mb-4">
        <NInput v-model:value="workshopQuery" size="large" :placeholder="t('discover.searchPlaceholder')"
          clearable @keyup.enter="searchWorkshop()">
          <template #prefix>
            <NIcon :size="18"><Search /></NIcon>
          </template>
        </NInput>
        <NSelect
          :value="workshopSortBy"
          :options="sortOptions"
          style="width: 140px"
          size="large"
          @update:value="onWorkshopSortChange"
        />
        <NButton size="large" type="primary" :loading="workshopLoading" @click="searchWorkshop(1)">
          {{ t("common.search") }}
        </NButton>
        <NButton size="large" secondary :loading="workshopLoading" @click="refreshWorkshop">
          <template #icon><NIcon :size="18"><RefreshCw /></NIcon></template>
          {{ t("common.refresh") }}
        </NButton>
      </div>
      <div v-if="workshopResults.length > 0">
        <div class="flex items-center justify-between mb-3">
          <span class="text-sm text-c-muted">{{ t("discover.resultCount", { total: workshopTotalCount }) }}</span>
        </div>
        <div :class="gridColsClass">
          <DiscoverCard
            v-for="mod in workshopResults"
            :key="mod.id"
            :name="mod.name"
            :image-url="mod.previewUrl"
            :description="mod.description"
            :author="mod.author"
            :stats="[
              { icon: ThumbsUp, value: mod.votesUp },
            ]"
          >
            <template #actions>
              <div class="flex items-center gap-2 flex-shrink-0">
                <NPopconfirm v-if="mod.subscribed" @positive-click="unsubscribeFromWorkshop(mod.id)">
                  <template #trigger>
                    <NButton size="small" secondary :loading="unsubscribingWorkshop.has(mod.id)">
                      <template #icon><NIcon :size="13"><X /></NIcon></template>
                      {{ t("discover.workshop.unsubscribe") }}
                    </NButton>
                  </template>
                  {{ t("library.mod.confirmUnsubscribe", { name: mod.name }) }}
                </NPopconfirm>
                <NButton v-else size="small" secondary :loading="subscribingWorkshop.has(mod.id)" @click="subscribeToWorkshop(mod.id)">
                  <template #icon><NIcon :size="13"><Plus /></NIcon></template>
                  {{ t("discover.workshop.subscribe") }}
                </NButton>
              </div>
            </template>
          </DiscoverCard>
        </div>
      </div>
      <EmptyState v-else-if="workshopSearched && !workshopLoading" :icon="Search" :title="t('discover.empty.notFound')" bordered />
    </div>

    <!-- 分页栏（浮动药丸） -->
    <div v-if="showPagination" class="flex justify-center sticky bottom-0 z-10 py-4">
      <DiscoverPagination
        v-if="isNexus"
        :page="page"
        :page-size="pageSize"
        :total-count="totalCount"
        :page-size-options="pageSizeOptions"
        @update:page="onPageChange"
        @update:page-size="onPageSizeChange"
      />
      <DiscoverPagination
        v-else
        :page="workshopPage"
        :page-size="workshopPageSize"
        :total-count="workshopTotalCount"
        :page-size-options="pageSizeOptions"
        @update:page="onWorkshopPageChange"
        @update:page-size="onWorkshopPageSizeChange"
      />
    </div>
  </div>
</template>

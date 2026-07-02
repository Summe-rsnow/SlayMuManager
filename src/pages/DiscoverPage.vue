<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useRouter, useRoute } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
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
const nexusResults = ref<RemoteMod[]>([])
const nexusTotalCount = ref(0)
const nexusLoading = ref(false)
const nexusInitialLoading = ref(true)
const nexusSearched = ref(false)
const nexusPage = ref(1)
const nexusPageSize = ref(12)

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
    invoke<boolean>("check_steam_status").then((running) => {
      if (!running) {
        dialog.warning({
          title: t("discover.workshop.tab"),
          content: t("discover.workshop.needSteam"),
          positiveText: t("common.ok"),
          maskClosable: true,
        })
        return
      }
      searchWorkshop()
    }).catch(() => {
      searchWorkshop()
    })
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
    if (mod) {
      mod.subscribed = true
      mod.subscribers++
    }
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
    if (mod) {
      mod.subscribed = false
      mod.subscribers = Math.max(0, mod.subscribers - 1)
    }
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

// --- 每页条数选项（基于列数 n: 4n, 6n, 10n, 20n）---
const PAGE_SIZE_MULTIPLIERS = [4, 6, 10, 20] as const
const pageSizeOptions = computed(() => {
  const n = discoverColumns.value
  return PAGE_SIZE_MULTIPLIERS.map(s => ({ label: `${s * n}`, value: s * n }))
})
// 列数变化时复位 nexusPageSize 到第一个合法值（immediate 确保跨页面切换后也生效）
watch(discoverColumns, (n) => {
  const valid = PAGE_SIZE_MULTIPLIERS.map(s => s * n)
  if (!valid.includes(nexusPageSize.value)) {
    nexusPageSize.value = valid[0]
  }
}, { immediate: true })
const hasApiKey = ref(true)

async function searchNexus(pg?: number, ignoreCache?: boolean) {
  nexusLoading.value = true
  nexusSearched.value = true
  const pageNum = pg ?? nexusPage.value
  const q = query.value.trim()
  const sb = sortBy.value
  const ps = nexusPageSize.value

  if (!ignoreCache && prefetchEnabled.value) {
    const cached = nexusCache.getCache(q, sb, pageNum, ps)
    if (cached) {
      nexusResults.value = cached.items
      nexusTotalCount.value = cached.totalCount
      nexusPage.value = pageNum
      nexusLoading.value = false
      nexusInitialLoading.value = false
      nexusFetchSilent(q, sb, pageNum, ps)
      return
    }
  }

  try {
    const result = await invoke<RemoteModSearchResult>("search_remote_mods", {
      query: q,
      page: pageNum,
      pageSize: ps,
      sortBy: sb,
    })
    nexusResults.value = result.items
    nexusTotalCount.value = result.totalCount
    nexusPage.value = pageNum
    nexusInitialLoading.value = false
    nexusCache.setCache(q, sb, pageNum, ps, result.items, result.totalCount)

    if (prefetchEnabled.value) {
      nexusPrefetchAdjacent(q, sb, pageNum, ps)
    }
  } catch (e: unknown) {
    console.error("Nexus search error:", e)
    message.error(String(e))
    nexusResults.value = []
    nexusInitialLoading.value = false
  } finally {
    nexusLoading.value = false
  }
}

/** 静默获取一页（仅写入缓存，不更新 UI） */
async function nexusFetchSilent(query: string, sortBy: string, page: number, pageSize: number) {
  try {
    const result = await invoke<RemoteModSearchResult>("search_remote_mods", { query, page, pageSize, sortBy })
    nexusCache.setCache(query, sortBy, page, pageSize, result.items, result.totalCount)
  } catch { /* silent */ }
}

function nexusPrefetchAdjacent(q: string, sb: string, currentPage: number, ps: number) {
  nexusCache.prefetchAdjacent(q, sb, currentPage, ps, nexusFetchSilent)
}

function onSortChange(val: string) {
  sortBy.value = val
  searchNexus()
}

function onPageChange(p: number) {
  nexusPage.value = p
  searchNexus()
}

function onPageSizeChange(val: number) {
  nexusPageSize.value = val
  searchNexus(1)
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
    return !nexusInitialLoading.value && nexusResults.value.length > 0 && nexusTotalCount.value > nexusPageSize.value
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
  // 获取 bootstrap（API key 信息，不阻塞搜索）
  invoke<AppBootstrap>("get_app_bootstrap").then((bootstrap) => {
    if (isActive.value) {
      hasApiKey.value = !!bootstrap.nexusApiKey
    }
  }).catch(() => {})

  // 立即触发搜索
  searchNexus()
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
  searchNexus(1)
}

function handleUserRefresh() {
  if (!hasApiKey.value) {
    showApiKeyDialog()
    return
  }
  searchNexus(undefined, true)
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
      <NButton size="large" type="primary" :loading="nexusLoading" @click="handleUserSearch()">
        {{ t("common.search") }}
      </NButton>
      <NButton size="large" secondary :loading="nexusLoading" @click="handleUserRefresh">
        <template #icon><NIcon :size="18"><RefreshCw /></NIcon></template>
        {{ t("common.refresh") }}
      </NButton>
    </div>

    <div class="flex-1">

    <!-- 初始加载骨架屏 -->
    <div v-if="nexusInitialLoading" :class="skeletonColsClass">
      <NCard v-for="i in nexusPageSize" :key="i" class="break-inside-avoid mb-4" :style="{ minHeight: '150px' }">
        <div class="flex gap-4 h-full animate-pulse">
          <div class="w-28 h-28 rounded-lg flex-shrink-0 bg-c-secondary" />
          <div class="flex-1 flex flex-col min-w-0">
            <!-- 标题行：名称 + 版本号 + 操作按钮 -->
            <div class="flex items-start justify-between gap-2">
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <div class="h-5 w-2/3 rounded bg-c-secondary" />
                <div class="h-4 w-10 rounded bg-c-secondary flex-shrink-0" />
              </div>
              <div class="h-7 w-16 rounded bg-c-secondary flex-shrink-0" />
            </div>
            <!-- 描述区：~6 行文本（匹配 TruncatedText line-clamp: 6） + 翻译按钮 -->
            <div class="min-h-0 mt-2 flex flex-col gap-1">
              <div class="h-3 rounded w-full bg-c-secondary" />
              <div class="h-3 rounded w-full bg-c-secondary" />
              <div class="h-3 rounded w-5/6 bg-c-secondary" />
              <div class="h-3 rounded w-full bg-c-secondary" />
              <div class="h-3 rounded w-4/5 bg-c-secondary" />
            </div>
            <div class="h-6 w-20 rounded bg-c-secondary mt-1.5" />
            <!-- 底部：作者 + 统计信息 -->
            <div class="flex items-center gap-3 pt-2 mt-auto">
              <div class="h-3.5 w-16 rounded bg-c-secondary" />
              <div class="h-3.5 w-20 rounded bg-c-secondary" />
              <div class="h-3.5 w-16 rounded bg-c-secondary" />
            </div>
          </div>
        </div>
      </NCard>
    </div>

    <!-- 结果列表 -->
    <div v-else-if="nexusResults.length > 0">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm text-c-muted">
          {{ t("discover.resultCount", { total: nexusTotalCount }) }}
        </span>
        <span />
      </div>

      <div :class="gridColsClass">
        <DiscoverCard
          v-for="mod in nexusResults"
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

    <EmptyState v-else-if="nexusSearched && !nexusLoading" :icon="Search" :title="t('discover.empty.notFound')" :description="hasApiKey ? undefined : t('discover.empty.needsApiKey')" bordered />

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
              { icon: ArrowDown, value: mod.votesDown },
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
                <NButton size="small" secondary class="flex-shrink-0" @click="openModPage(`https://steamcommunity.com/sharedfiles/filedetails/?id=${mod.id}`)">
                  <template #icon><NIcon :size="13"><ExternalLink /></NIcon></template>
                  {{ t("discover.details") }}
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
        :page="nexusPage"
        :page-size="nexusPageSize"
        :total-count="nexusTotalCount"
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

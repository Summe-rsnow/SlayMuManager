<script setup lang="ts">
import { ref, computed, watch, onMounted, onActivated, onDeactivated } from "vue"
import { storeToRefs } from "pinia"
import { useRouter, useRoute } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import {
  Button,
  Card,
  EmptyState,
  Input,
  PageHeader,
  Pagination,
  Popconfirm,
  SegmentedControl,
  Select,
  useConfirm,
  useToast,
} from "@xcmm1543/miuui-vue"
import {
  Search,
  ExternalLink,
  ThumbsUp,
  ArrowDown,
  PackageOpen,
  RefreshCw,
  Plus,
  X,
  Download,
  LayoutGrid,
  Columns2,
  List,
} from "@lucide/vue"
import type {
  RemoteMod,
  RemoteModSearchResult,
  AppBootstrap,
  WorkshopMod,
  WorkshopSearchResult,
} from "../types"
import { useIsActive } from "@/composables/useIsActive"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import { useSearchPrefetch } from "@/composables/useSearchPrefetch"
import { useHighlightStore } from "@/stores/useHighlightStore"
import { useModCacheStore } from "@/stores/useModCacheStore"
import { yieldToMain, scheduleBackgroundTask } from "@/utils/scheduler"
import DiscoverCard from "@/components/DiscoverCard.vue"
import SkeletonDiscoverCard from "@/components/SkeletonDiscoverCard.vue"
import SkeletonDiscoverSplitView from "@/components/SkeletonDiscoverSplitView.vue"
import NexusDownloadModal from "@/components/NexusDownloadModal.vue"
import DiscoverSplitView from "@/components/DiscoverSplitView.vue"
defineOptions({ name: "DiscoverPage" })

const { t } = useI18n()
const { toast } = useToast()
const { confirm } = useConfirm()
const router = useRouter()
const route = useRoute()
const highlightStore = useHighlightStore()
const { isActive } = useIsActive()
const { fetchMods } = useModCacheStore()
const prefStore = useDiscoverPrefStore()
const { discoverColumns, prefetchEnabled, discoverViewMode, discoverSourceTab, discoverPageSize } =
  storeToRefs(prefStore)
const { setDiscoverViewMode, setDiscoverSourceTab, setDiscoverPageSize } = prefStore
const selectedNexusId = ref<string | null>(null)
const selectedWorkshopId = ref<number | null>(null)

// --- 预取缓存（Nexus + Workshop 共用泛型 composable）---
const nexusCache = useSearchPrefetch<RemoteMod>("nx")
const workshopCache = useSearchPrefetch<WorkshopMod>("ws")

// --- Nexus 模组下载弹窗 ---
const showDownloadModal = ref(false)
const downloadModId = ref<number | null>(null)
const downloadModName = ref("")

function openNexusDownload(mod: RemoteMod) {
  const idNum = parseInt(mod.remoteId, 10)
  if (isNaN(idNum)) return
  downloadModId.value = idNum
  downloadModName.value = mod.name
  showDownloadModal.value = true
}

// --- 发现页列数与瀑布流自动补位映射 ---
const colCount = computed(() => {
  const n = discoverColumns.value
  return n >= 1 && n <= 4 ? n : 3
})

const gridColsClass = computed(() => {
  const map: Record<number, string> = {
    1: "grid grid-cols-1 gap-4 items-start",
    2: "grid grid-cols-2 gap-4 items-start",
    3: "grid grid-cols-3 gap-4 items-start",
    4: "grid grid-cols-4 gap-4 items-start",
  }
  return map[colCount.value] || map[3]
})

function distributeMasonry<T>(
  items: T[],
  cols: number,
  getText: (item: T) => string | null | undefined,
): T[][] {
  const buckets: T[][] = Array.from({ length: cols }, () => [])
  if (cols <= 1) {
    buckets[0] = items
    return buckets
  }
  const heights = Array.from({ length: cols }, () => 0)
  for (const item of items) {
    let minIdx = 0
    for (let i = 1; i < cols; i++) {
      if (heights[i] < heights[minIdx]) minIdx = i
    }
    buckets[minIdx].push(item)
    const len = getText(item)?.trim().length ?? 0
    const approxLines = Math.min(6, Math.max(1, Math.ceil(len / 42)))
    heights[minIdx] += 132 + approxLines * 18
  }
  return buckets
}

const isRouteActive = ref(true)
let nexusReqSeq = 0
let workshopReqSeq = 0
let workshopTabSeq = 0

const CARD_RENDER_CHUNK = 12
const renderedNexusCount = ref(CARD_RENDER_CHUNK)
const renderedWorkshopCount = ref(CARD_RENDER_CHUNK)

async function hydrateCards(total: number, targetRef: { value: number }, isCurrent: () => boolean) {
  targetRef.value = Math.min(total, CARD_RENDER_CHUNK)
  while (targetRef.value < total) {
    await yieldToMain()
    if (!isActive.value || !isCurrent()) return
    targetRef.value = Math.min(total, targetRef.value + CARD_RENDER_CHUNK)
  }
}

const nexusMasonryColumns = computed(() =>
  distributeMasonry(
    nexusResults.value.slice(0, renderedNexusCount.value),
    colCount.value,
    (m) => m.summary,
  ),
)

const workshopMasonryColumns = computed(() =>
  distributeMasonry(
    workshopResults.value.slice(0, renderedWorkshopCount.value),
    colCount.value,
    (m) => m.description,
  ),
)

// --- 视图与数据源分段选项 ---
const viewModeOptions = computed(() => [
  {
    value: "grid",
    label: t("settings.discover.viewModeGrid"),
    title: t("settings.discover.viewModeGrid"),
  },
  {
    value: "split",
    label: t("settings.discover.viewModeSplit"),
    title: t("settings.discover.viewModeSplit"),
  },
])

const sourceTabOptions = computed(() => [
  { value: "nexus", label: "Nexus" },
  { value: "workshop", label: t("discover.workshop.tab") },
])

function onSourceTabChange(val: string) {
  if (val === "nexus") switchNexusTab()
  else if (val === "workshop") switchWorkshopTab()
}

// --- 排序选项 ---
const sortOptions = computed(() => [
  { label: t("discover.sort.latestAdded"), value: "latest_added" },
  { label: t("discover.sort.latestUpdated"), value: "latest_updated" },
  { label: t("discover.sort.trending"), value: "trending" },
  { label: t("discover.sort.downloads"), value: "downloads" },
])

// --- 状态 ---
const tab = ref<"nexus" | "workshop">(
  route.query.tab === "workshop" || route.query.tab === "nexus"
    ? (route.query.tab as "nexus" | "workshop")
    : discoverSourceTab.value || "nexus",
)
const isNexus = computed(() => tab.value === "nexus")
const query = ref("")
const sortBy = ref("latest_added")
const nexusResults = ref<RemoteMod[]>([])
const nexusTotalCount = ref(0)
const nexusLoading = ref(false)
const nexusInitialLoading = ref(true)
const nexusSearched = ref(false)
const nexusPage = ref(1)
const nexusPageSize = computed({
  get: () => discoverPageSize.value,
  set: (val: number) => setDiscoverPageSize(val),
})

// --- 创意工坊状态 ---
const workshopQuery = ref("")
const workshopSortBy = ref("latest_added")
const workshopResults = ref<WorkshopMod[]>([])
const workshopLoading = ref(false)
const workshopSearched = ref(false)
const workshopPage = ref(1)
const workshopPageSize = computed({
  get: () => discoverPageSize.value,
  set: (val: number) => setDiscoverPageSize(val),
})
const workshopTotalCount = ref(0)
const subscribingWorkshop = ref(new Set<number>())
const unsubscribingWorkshop = ref(new Set<number>())

watch(
  nexusResults,
  (list) => {
    if (list && list.length > 0) {
      if (!selectedNexusId.value || !list.some((m) => m.remoteId === selectedNexusId.value)) {
        selectedNexusId.value = list[0].remoteId
      }
    } else {
      selectedNexusId.value = null
    }
  },
  { immediate: true },
)

watch(
  workshopResults,
  (list) => {
    if (list && list.length > 0) {
      if (!selectedWorkshopId.value || !list.some((m) => m.id === selectedWorkshopId.value)) {
        selectedWorkshopId.value = list[0].id
      }
    } else {
      selectedWorkshopId.value = null
    }
  },
  { immediate: true },
)

function switchNexusTab() {
  workshopTabSeq++
  tab.value = "nexus"
  setDiscoverSourceTab("nexus")
  const q = { ...route.query }
  delete q.tab
  router.replace({ query: q })
  if (!nexusSearched.value && !nexusLoading.value) {
    void searchNexus()
  }
}

async function switchWorkshopTab() {
  const tabSeq = ++workshopTabSeq
  tab.value = "workshop"
  setDiscoverSourceTab("workshop")
  router.replace({ query: { ...route.query, tab: "workshop" } })
  if (!workshopSearched.value) {
    workshopLoading.value = true
    try {
      const running = await invoke<boolean>("check_steam_status")
      if (!isActive.value || tabSeq !== workshopTabSeq || tab.value !== "workshop") {
        if (isActive.value && !workshopSearched.value) {
          workshopLoading.value = false
        }
        return
      }
      if (!running) {
        workshopLoading.value = false
        if (!isRouteActive.value) return
        await confirm({
          title: t("discover.workshop.tab"),
          description: t("discover.workshop.needSteam"),
          confirmLabel: t("common.ok"),
          cancelLabel: t("common.close"),
        })
        return
      }
      void searchWorkshop()
    } catch {
      if (isActive.value && tabSeq === workshopTabSeq && tab.value === "workshop") {
        void searchWorkshop()
      } else if (isActive.value && !workshopSearched.value) {
        workshopLoading.value = false
      }
    }
  }
}

/** 静默获取一页（仅写入缓存，不更新 UI） */
async function workshopFetchSilent(query: string, sortBy: string, page: number, pageSize: number) {
  try {
    const result = await invoke<WorkshopSearchResult>("search_workshop", {
      query,
      page,
      pageSize,
      sortBy,
    })
    workshopCache.setCache(query, sortBy, page, pageSize, result.items, result.totalCount)
  } catch {
    /* silent */
  }
}

/** 后台预取 Workshop 相邻页 */
function workshopPrefetchAdjacent(q: string, sb: string, currentPage: number, ps: number) {
  workshopCache.prefetchAdjacent(q, sb, currentPage, ps, workshopFetchSilent)
}

async function subscribeToWorkshop(id: number) {
  subscribingWorkshop.value = new Set(subscribingWorkshop.value).add(id)
  try {
    await invoke("subscribe_workshop_mod", { publishedFileId: id })
    if (!isActive.value) return
    const mod = workshopResults.value.find((m) => m.id === id)
    if (mod) {
      mod.subscribed = true
      mod.subscribers++
    }
    toast({ title: t("discover.workshop.subscribedSuccess"), variant: "success" })
  } catch (e: unknown) {
    if (!isActive.value) return
    console.error("Workshop subscribe error:", e)
    const ok = await confirm({
      title: t("discover.workshop.subscribeFailedTitle"),
      description: String(e),
      confirmLabel: t("discover.workshop.openInWeb"),
      cancelLabel: t("common.cancel"),
    })
    if (ok) {
      openModPage(`https://steamcommunity.com/sharedfiles/filedetails/?id=${id}`)
    }
  } finally {
    if (isActive.value) {
      const next = new Set(subscribingWorkshop.value)
      next.delete(id)
      subscribingWorkshop.value = next
    }
  }
}

async function unsubscribeFromWorkshop(id: number) {
  unsubscribingWorkshop.value = new Set(unsubscribingWorkshop.value).add(id)
  try {
    await invoke("unsubscribe_workshop_mod", { publishedFileId: id })
    if (!isActive.value) return
    const mod = workshopResults.value.find((m) => m.id === id)
    if (mod) {
      mod.subscribed = false
      mod.subscribers = Math.max(0, mod.subscribers - 1)
    }
    toast({ title: t("library.mod.unsubscribed"), variant: "success" })
    await fetchMods()
  } catch (e: unknown) {
    if (!isActive.value) return
    console.error("Workshop unsubscribe error:", e)
    toast({ title: String(e), variant: "error" })
  } finally {
    if (isActive.value) {
      const next = new Set(unsubscribingWorkshop.value)
      next.delete(id)
      unsubscribingWorkshop.value = next
    }
  }
}

// --- 每页条数选项（基于列数 n: 4n, 6n, 10n, 20n，确保每行填满）---
const PAGE_SIZE_MULTIPLIERS = [4, 6, 10, 20] as const
const pageSizeOptions = computed(() => {
  const n = discoverColumns.value || 3
  return PAGE_SIZE_MULTIPLIERS.map((s) => ({ label: `${s * n}`, value: s * n }))
})
// 列数变化时复位 discoverPageSize 到当前列数首个合法倍数，并触发搜索
watch(
  discoverColumns,
  (n) => {
    const valid = PAGE_SIZE_MULTIPLIERS.map((s) => s * n)
    if (!valid.includes(discoverPageSize.value)) {
      discoverPageSize.value = valid[0]
    }
  },
  { immediate: false },
)
const hasApiKey = ref(true)

async function searchNexus(pg?: number, ignoreCache?: boolean) {
  const reqId = ++nexusReqSeq
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
      void hydrateCards(cached.items.length, renderedNexusCount, () => reqId === nexusReqSeq)
      scheduleBackgroundTask(() => {
        if (!isActive.value || !isRouteActive.value || reqId !== nexusReqSeq) return
        void nexusFetchSilent(q, sb, pageNum, ps)
      })
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
    nexusCache.setCache(q, sb, pageNum, ps, result.items, result.totalCount)
    if (!isActive.value || reqId !== nexusReqSeq) return

    nexusResults.value = result.items
    nexusTotalCount.value = result.totalCount
    nexusPage.value = pageNum
    nexusInitialLoading.value = false
    void hydrateCards(result.items.length, renderedNexusCount, () => reqId === nexusReqSeq)

    if (prefetchEnabled.value) {
      scheduleBackgroundTask(() => {
        if (!isActive.value || !isRouteActive.value || reqId !== nexusReqSeq) return
        nexusPrefetchAdjacent(q, sb, pageNum, ps)
      })
    }
  } catch (e: unknown) {
    if (!isActive.value || reqId !== nexusReqSeq) return
    console.error("Nexus search error:", e)
    if (isRouteActive.value && tab.value === "nexus") {
      toast({ title: String(e), variant: "error" })
    }
    nexusResults.value = []
    renderedNexusCount.value = 0
    nexusInitialLoading.value = false
  } finally {
    if (isActive.value && reqId === nexusReqSeq) nexusLoading.value = false
  }
}

/** 静默获取一页（仅写入缓存，不更新 UI） */
async function nexusFetchSilent(query: string, sortBy: string, page: number, pageSize: number) {
  try {
    const result = await invoke<RemoteModSearchResult>("search_remote_mods", {
      query,
      page,
      pageSize,
      sortBy,
    })
    nexusCache.setCache(query, sortBy, page, pageSize, result.items, result.totalCount)
  } catch {
    /* silent */
  }
}

function nexusPrefetchAdjacent(q: string, sb: string, currentPage: number, ps: number) {
  nexusCache.prefetchAdjacent(q, sb, currentPage, ps, nexusFetchSilent)
}

function onSortChange(val: string | number | undefined) {
  if (val == null) return
  sortBy.value = String(val)
  searchNexus()
}

function onPageChange(p: number) {
  nexusPage.value = p
  searchNexus()
}

function onPageSizeChange(val: number) {
  setDiscoverPageSize(val)
  searchNexus(1)
}

function onWorkshopPageChange(p: number) {
  workshopPage.value = p
  searchWorkshop(p)
}

function onWorkshopPageSizeChange(val: number) {
  setDiscoverPageSize(val)
  workshopPage.value = 1
  searchWorkshop(1)
}

function onWorkshopSortChange(val: string | number | undefined) {
  if (val == null) return
  workshopSortBy.value = String(val)
  workshopPage.value = 1
  searchWorkshop(1)
}

function refreshWorkshop() {
  searchWorkshop(workshopPage.value, true)
}

async function searchWorkshop(pg?: number, ignoreCache?: boolean) {
  const reqId = ++workshopReqSeq
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
      void hydrateCards(cached.items.length, renderedWorkshopCount, () => reqId === workshopReqSeq)
      scheduleBackgroundTask(() => {
        if (!isActive.value || !isRouteActive.value || reqId !== workshopReqSeq) return
        void workshopFetchSilent(q, sb, pageNum, ps)
      })
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
    workshopCache.setCache(q, sb, pageNum, ps, result.items, result.totalCount)
    if (!isActive.value || reqId !== workshopReqSeq) return

    workshopResults.value = result.items
    workshopTotalCount.value = result.totalCount
    workshopPage.value = pageNum
    void hydrateCards(result.items.length, renderedWorkshopCount, () => reqId === workshopReqSeq)

    if (prefetchEnabled.value) {
      scheduleBackgroundTask(() => {
        if (!isActive.value || !isRouteActive.value || reqId !== workshopReqSeq) return
        workshopPrefetchAdjacent(q, sb, pageNum, ps)
      })
    }
  } catch (e: unknown) {
    if (!isActive.value || reqId !== workshopReqSeq) return
    console.error("Workshop search error:", e)
    if (isRouteActive.value && tab.value === "workshop") {
      toast({ title: String(e), variant: "error" })
    }
  } finally {
    if (isActive.value && reqId === workshopReqSeq) {
      workshopLoading.value = false
    }
  }
}

const showPagination = computed(() => {
  if (isNexus.value) {
    return (
      !nexusInitialLoading.value &&
      nexusResults.value.length > 0 &&
      nexusTotalCount.value > nexusPageSize.value
    )
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

onMounted(async () => {
  // 让当前帧先完成路由切换与骨架渲染，再异步发起后端请求
  await yieldToMain()
  if (!isActive.value) return

  // 获取 bootstrap（API key 信息，不阻塞搜索）
  invoke<AppBootstrap>("get_app_bootstrap")
    .then((bootstrap) => {
      if (isActive.value) {
        hasApiKey.value = !!bootstrap.nexusApiKey
      }
    })
    .catch(() => {})

  // 初始加载：如果保存的标签是 workshop，启动工坊检查与搜索；否则搜索 Nexus
  if (tab.value === "workshop") {
    void switchWorkshopTab()
  } else {
    void searchNexus()
  }
})

onActivated(() => {
  isRouteActive.value = true
  if (tab.value === "workshop" && !workshopSearched.value && !workshopLoading.value) {
    void switchWorkshopTab()
  } else if (tab.value === "nexus" && !nexusSearched.value && !nexusLoading.value) {
    void searchNexus()
  }
})

onDeactivated(() => {
  isRouteActive.value = false
})

// --- 用户操作入口：未配置 API Key 时弹窗引导 ---
const showApiKeyDialog = async () => {
  const ok = await confirm({
    title: t("settings.prompt.apiKeyRequired"),
    description: t("settings.prompt.apiKeyRequiredDesc"),
    confirmLabel: t("settings.prompt.goToSettings"),
    cancelLabel: t("common.cancel"),
  })
  if (ok) {
    highlightStore.highlight("nexus")
    router.push("/settings")
  }
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
  <div class="relative flex flex-col h-full overflow-hidden max-h-full">
    <PageHeader
      class="flex-shrink-0 mb-5"
      :title="t('discover.title')"
      :description="t('discover.subtitle')"
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <SegmentedControl
            :model-value="discoverViewMode"
            :options="viewModeOptions"
            size="xs"
            icon-only
            material="glass"
            :aria-label="t('settings.discover.viewMode')"
            @update:model-value="(v) => setDiscoverViewMode(v as 'grid' | 'split')"
          >
            <template #item="{ option }">
              <LayoutGrid v-if="option.value === 'grid'" :size="14" />
              <Columns2 v-else :size="14" />
            </template>
          </SegmentedControl>
          <SegmentedControl
            :model-value="tab"
            :options="sourceTabOptions"
            size="sm"
            material="glass"
            :aria-label="t('discover.title')"
            @update:model-value="onSourceTabChange"
          />
        </div>
      </template>
    </PageHeader>

    <!-- Nexus 搜索 -->
    <template v-if="isNexus">
      <!-- 搜索栏 + 排序 + 刷新 -->
      <div class="flex gap-2 mb-3 flex-shrink-0">
        <Input
          v-model="query"
          size="lg"
          :placeholder="t('discover.searchPlaceholder')"
          clearable
          class="flex-1"
          @enter="handleUserSearch()"
        >
          <template #prefix>
            <Search :size="18" />
          </template>
        </Input>
        <Select
          :model-value="sortBy"
          :options="sortOptions"
          class="w-36"
          size="lg"
          @update:model-value="onSortChange"
        />
        <Button size="lg" variant="solid" :loading="nexusLoading" @click="handleUserSearch()">
          {{ t("common.search") }}
        </Button>
        <Button size="lg" variant="secondary" :loading="nexusLoading" @click="handleUserRefresh">
          <template #icon><RefreshCw :size="18" /></template>
          {{ t("common.refresh") }}
        </Button>
      </div>

      <div class="flex-1 min-h-0 flex flex-col overflow-hidden">
        <!-- 初始加载骨架屏 -->
        <div
          v-if="nexusInitialLoading || (nexusLoading && nexusResults.length === 0)"
          class="flex-1 min-h-0 flex flex-col overflow-hidden"
        >
          <div class="flex items-center justify-between mb-3 flex-shrink-0">
            <div class="h-5 w-28 rounded bg-c-secondary/40 animate-pulse" />
          </div>
          <SkeletonDiscoverSplitView v-if="discoverViewMode === 'split'" />
          <div
            v-else
            :class="gridColsClass"
            class="flex-1 min-h-0 overflow-y-auto scrollbar-hide pb-24"
          >
            <SkeletonDiscoverCard
              v-for="i in Math.min(nexusPageSize, 12)"
              :key="'skel-init-' + i"
            />
          </div>
        </div>
        <!-- 结果列表 -->
        <div
          v-else-if="nexusResults.length > 0"
          class="flex-1 min-h-0 flex flex-col overflow-hidden"
        >
          <div class="flex items-center justify-between mb-3 flex-shrink-0">
            <span class="text-sm text-c-muted">
              {{ t("discover.resultCount", { total: nexusTotalCount }) }}
            </span>
          </div>

          <div
            v-if="discoverViewMode === 'grid'"
            :class="gridColsClass"
            class="flex-1 min-h-0 overflow-y-auto scrollbar-hide pb-24"
          >
            <div
              v-for="(colItems, colIdx) in nexusMasonryColumns"
              :key="'nx-col-' + colIdx"
              class="flex flex-col gap-4 min-w-0"
            >
              <DiscoverCard
                v-for="mod in colItems"
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
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <Button size="sm" variant="solid" @click="openNexusDownload(mod)">
                      <template #icon><Download :size="13" /></template>
                      {{ t("discover.download") }}
                    </Button>
                    <Button size="sm" variant="secondary" @click="openModPage(mod.detailUrl)">
                      <template #icon><ExternalLink :size="13" /></template>
                      {{ t("discover.details") }}
                    </Button>
                  </div>
                </template>
              </DiscoverCard>
            </div>
          </div>
          <DiscoverSplitView
            v-else
            type="nexus"
            :nexus-items="nexusResults"
            :selected-id="selectedNexusId"
            @select-nexus="(m) => (selectedNexusId = m.remoteId)"
            @download-nexus="openNexusDownload"
            @open-details="openModPage"
          />
        </div>

        <Card v-else-if="nexusSearched && !nexusLoading" material="glass" padding="sm">
          <EmptyState
            :title="t('discover.empty.notFound')"
            :description="hasApiKey ? undefined : t('discover.empty.needsApiKey')"
          >
            <template #icon><Search :size="40" /></template>
          </EmptyState>
        </Card>

        <Card v-else material="glass" padding="sm">
          <EmptyState
            :title="t('discover.empty.startSearch')"
            :description="hasApiKey ? undefined : t('discover.empty.needsApiKey')"
          >
            <template #icon><PackageOpen :size="40" /></template>
          </EmptyState>
        </Card>
      </div>
    </template>

    <!-- Steam 创意工坊 -->
    <div v-show="tab === 'workshop'" class="flex-1 min-h-0 flex flex-col overflow-hidden">
      <div class="flex gap-2 mb-3 flex-shrink-0">
        <Input
          v-model="workshopQuery"
          size="lg"
          :placeholder="t('discover.searchPlaceholder')"
          clearable
          class="flex-1"
          @enter="searchWorkshop()"
        >
          <template #prefix>
            <Search :size="18" />
          </template>
        </Input>
        <Select
          :model-value="workshopSortBy"
          :options="sortOptions"
          class="w-36"
          size="lg"
          @update:model-value="onWorkshopSortChange"
        />
        <Button size="lg" variant="solid" :loading="workshopLoading" @click="searchWorkshop(1)">
          {{ t("common.search") }}
        </Button>
        <Button size="lg" variant="secondary" :loading="workshopLoading" @click="refreshWorkshop">
          <template #icon><RefreshCw :size="18" /></template>
          {{ t("common.refresh") }}
        </Button>
      </div>
      <div
        v-if="workshopLoading && workshopResults.length === 0"
        class="flex-1 min-h-0 flex flex-col overflow-hidden"
      >
        <div class="flex items-center justify-between mb-3 flex-shrink-0">
          <div class="h-5 w-28 rounded bg-c-secondary/40 animate-pulse" />
        </div>
        <SkeletonDiscoverSplitView v-if="discoverViewMode === 'split'" />
        <div
          v-else
          :class="gridColsClass"
          class="flex-1 min-h-0 overflow-y-auto scrollbar-hide pb-24"
        >
          <SkeletonDiscoverCard v-for="i in Math.min(workshopPageSize, 12)" :key="'skel-ws-' + i" />
        </div>
      </div>
      <div
        v-else-if="workshopResults.length > 0"
        class="flex-1 min-h-0 flex flex-col overflow-hidden"
      >
        <div class="flex items-center justify-between mb-3 flex-shrink-0">
          <span class="text-sm text-c-muted">{{
            t("discover.resultCount", { total: workshopTotalCount })
          }}</span>
        </div>
        <div
          v-if="discoverViewMode === 'grid'"
          :class="gridColsClass"
          class="flex-1 min-h-0 overflow-y-auto scrollbar-hide pb-24"
        >
          <div
            v-for="(colItems, colIdx) in workshopMasonryColumns"
            :key="'ws-col-' + colIdx"
            class="flex flex-col gap-4 min-w-0"
          >
            <DiscoverCard
              v-for="mod in colItems"
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
                  <Popconfirm
                    v-if="mod.subscribed"
                    :title="t('library.mod.confirmUnsubscribe', { name: mod.name })"
                    danger
                    material="glass"
                    @confirm="unsubscribeFromWorkshop(mod.id)"
                  >
                    <template #trigger>
                      <Button
                        size="sm"
                        variant="secondary"
                        :loading="unsubscribingWorkshop.has(mod.id)"
                      >
                        <template #icon><X :size="13" /></template>
                        {{ t("discover.workshop.unsubscribe") }}
                      </Button>
                    </template>
                  </Popconfirm>
                  <Button
                    v-else
                    size="sm"
                    variant="secondary"
                    :loading="subscribingWorkshop.has(mod.id)"
                    @click="subscribeToWorkshop(mod.id)"
                  >
                    <template #icon><Plus :size="13" /></template>
                    {{ t("discover.workshop.subscribe") }}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    class="flex-shrink-0"
                    @click="
                      openModPage(
                        `https://steamcommunity.com/sharedfiles/filedetails/?id=${mod.id}`,
                      )
                    "
                  >
                    <template #icon><ExternalLink :size="13" /></template>
                    {{ t("discover.details") }}
                  </Button>
                </div>
              </template>
            </DiscoverCard>
          </div>
        </div>
        <DiscoverSplitView
          v-else
          type="workshop"
          :workshop-items="workshopResults"
          :selected-id="selectedWorkshopId"
          :subscribing-ids="subscribingWorkshop"
          :unsubscribing-ids="unsubscribingWorkshop"
          @select-workshop="(m) => (selectedWorkshopId = m.id)"
          @subscribe-workshop="subscribeToWorkshop"
          @unsubscribe-workshop="unsubscribeFromWorkshop"
          @open-details="openModPage"
        />
      </div>
      <Card v-else-if="workshopSearched && !workshopLoading" material="glass" padding="sm">
        <EmptyState :title="t('discover.empty.notFound')">
          <template #icon><Search :size="40" /></template>
        </EmptyState>
      </Card>
    </div>

    <!-- 分页栏（浮动在发现页上方且位置固定，不占文档流空间） -->
    <div
      v-if="showPagination"
      class="absolute bottom-5 left-1/2 -translate-x-1/2 z-30 pointer-events-none"
    >
      <div class="pointer-events-auto">
        <Pagination
          v-if="isNexus"
          mode="numeric"
          material="glass"
          radius="full"
          :page="nexusPage"
          :page-size="nexusPageSize"
          :total="nexusTotalCount"
          :page-size-options="pageSizeOptions"
          @update:page="onPageChange"
          @update:page-size="onPageSizeChange"
        >
          <template #prefix><List :size="14" class="text-c-muted" /></template>
        </Pagination>
        <Pagination
          v-else
          mode="numeric"
          material="glass"
          radius="full"
          :page="workshopPage"
          :page-size="workshopPageSize"
          :total="workshopTotalCount"
          :page-size-options="pageSizeOptions"
          @update:page="onWorkshopPageChange"
          @update:page-size="onWorkshopPageSizeChange"
        >
          <template #prefix><List :size="14" class="text-c-muted" /></template>
        </Pagination>
      </div>
    </div>
    <!-- Nexus 模组下载弹窗 -->
    <NexusDownloadModal
      v-model:show="showDownloadModal"
      :mod-id="downloadModId"
      :mod-name="downloadModName"
    />
  </div>
</template>

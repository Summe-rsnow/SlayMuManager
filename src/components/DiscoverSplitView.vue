<script setup lang="ts">
import { ref, computed } from "vue"
import { useI18n } from "vue-i18n"
import { storeToRefs } from "pinia"
import { NButton, NIcon, NTag, NModal } from "naive-ui"
import { ThumbsUp, ArrowDown, ExternalLink, Download, Plus, X, User, HardDrive } from "@lucide/vue"
import ModPreviewImage from "./ModPreviewImage.vue"
import ModTranslateBlock from "./ModTranslateBlock.vue"
import FloatingTip from "./FloatingTip.vue"
import EmptyState from "./EmptyState.vue"
import { renderBBCode } from "@/utils/bbcode"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import type { RemoteMod, WorkshopMod } from "@/types"
const props = defineProps<{
  type: "nexus" | "workshop"
  nexusItems?: RemoteMod[]
  workshopItems?: WorkshopMod[]
  selectedId: string | number | null
  subscribingIds?: Set<number>
  unsubscribingIds?: Set<number>
}>()

const emit = defineEmits<{
  (e: "selectNexus", item: RemoteMod): void
  (e: "selectWorkshop", item: WorkshopMod): void
  (e: "downloadNexus", item: RemoteMod): void
  (e: "openDetails", url: string): void
  (e: "subscribeWorkshop", id: number): void
  (e: "unsubscribeWorkshop", id: number): void
}>()

const { t } = useI18n()
const discoverPrefStore = useDiscoverPrefStore()
const { parseWorkshopBBCode } = storeToRefs(discoverPrefStore)
const previewModalShow = ref(false)
const previewModalImageUrl = ref("")

function openImagePreview(url: string) {
  previewModalImageUrl.value = url
  previewModalShow.value = true
}
function formatCount(num?: number | null): string {
  if (!num) return "0"
  if (num >= 1000000) return (num / 1000000).toFixed(1) + "M"
  if (num >= 1000) return (num / 1000).toFixed(1) + "k"
  return String(num)
}
const selectedNexus = computed(() => {
  if (props.type !== "nexus" || !props.nexusItems) return null
  return (
    props.nexusItems.find((m) => m.remoteId === String(props.selectedId)) ||
    props.nexusItems[0] ||
    null
  )
})

const selectedWorkshop = computed(() => {
  if (props.type !== "workshop" || !props.workshopItems) return null
  return (
    props.workshopItems.find((m) => m.id === Number(props.selectedId)) ||
    props.workshopItems[0] ||
    null
  )
})

// 过滤非空标签：无有效标签时彻底不显示
const validWorkshopTags = computed(() => {
  if (!selectedWorkshop.value?.tags) return []
  return selectedWorkshop.value.tags
    .map((t) => (typeof t === "string" ? t.trim() : ""))
    .filter((t) => t.length > 0 && t !== "—" && t !== "-" && t !== "None")
})
const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  border: "var(--glass-surface-border)",
  boxShadow: "var(--shadow-glass-surface)",
}))
</script>

<template>
  <div class="flex-1 flex gap-4 min-h-0 overflow-hidden w-full h-full">
    <!-- ================= 左侧模组列表 ================= -->
    <div
      class="w-80 md:w-96 flex-shrink-0 flex flex-col rounded-xl overflow-hidden h-full min-h-0"
      :style="cardStyle"
    >
      <div
        class="flex-1 min-h-0 overflow-y-auto p-3.5 space-y-2.5 scrollbar-thin pb-24"
        @wheel.stop
      >
        <!-- Nexus 列表条目 -->
        <template v-if="type === 'nexus'">
          <div
            v-for="item in nexusItems"
            :key="item.remoteId"
            class="split-list-item p-3 rounded-xl cursor-pointer transition-all duration-200 border flex items-center gap-3"
            :class="selectedNexus?.remoteId === item.remoteId ? 'active-item' : ''"
            @click="emit('selectNexus', item)"
          >
            <ModPreviewImage
              :src="item.thumbnailUrl || item.pictureUrl"
              :alt="item.name"
              :previewable="false"
              class="w-14 h-14 rounded-lg flex-shrink-0 pointer-events-none"
            />

            <div class="min-w-0 flex-1">
              <div class="font-semibold text-sm truncate text-c-primary leading-tight mb-1">
                {{ item.name }}
              </div>
              <div class="text-xs text-c-muted truncate mb-1.5 flex items-center gap-1">
                <span class="truncate">{{ item.author || t("discover.unknownAuthor") }}</span>
                <span
                  v-if="item.latestVersion && item.latestVersion !== '—'"
                  class="font-mono text-c-muted flex-shrink-0"
                >
                  · v{{ item.latestVersion }}
                </span>
              </div>
              <div class="flex items-center gap-3 text-[11px] text-c-muted font-mono">
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ThumbsUp /></NIcon>
                  {{ formatCount(item.endorsementCount) }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ArrowDown /></NIcon>
                  {{ formatCount(item.downloadCount) }}
                </span>
              </div>
            </div>
          </div>
        </template>

        <!-- Workshop 列表条目 -->
        <template v-else>
          <div
            v-for="item in workshopItems"
            :key="item.id"
            class="split-list-item p-3 rounded-xl cursor-pointer transition-all duration-200 border flex items-center gap-3"
            :class="selectedWorkshop?.id === item.id ? 'active-item' : ''"
            @click="emit('selectWorkshop', item)"
          >
            <ModPreviewImage
              :src="item.previewUrl"
              :alt="item.name"
              :previewable="false"
              class="w-14 h-14 rounded-lg flex-shrink-0 pointer-events-none"
            />

            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-1.5 mb-1">
                <span class="font-semibold text-sm truncate text-c-primary leading-tight flex-1">
                  {{ item.name }}
                </span>
                <NTag
                  v-if="item.subscribed"
                  type="success"
                  size="tiny"
                  :bordered="false"
                  class="flex-shrink-0"
                >
                  {{ t("discover.workshop.subscribed") }}
                </NTag>
              </div>
              <div class="text-xs text-c-muted truncate mb-1.5 flex items-center gap-1">
                <NIcon :size="11" class="text-c-muted flex-shrink-0"><User /></NIcon>
                <span class="truncate">{{ item.author }}</span>
              </div>
              <div class="flex items-center gap-3 text-[11px] text-c-muted font-mono">
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ThumbsUp /></NIcon>
                  {{ formatCount(item.votesUp) }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="12"><ArrowDown /></NIcon>
                  {{ formatCount(item.votesDown) }}
                </span>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- ================= 右侧详情面板 ================= -->
    <div
      class="flex-1 min-w-0 flex flex-col rounded-xl overflow-hidden h-full min-h-0"
      :style="cardStyle"
    >
      <!-- Nexus 详情视图 -->
      <template v-if="type === 'nexus' && selectedNexus">
        <div
          class="flex-1 min-h-0 flex flex-col p-6 scrollbar-thin space-y-4 overflow-y-auto pb-24"
          @wheel.stop
        >
          <!-- 顶部区域：左侧封面大图 + 右侧标题/作者/版本/统计/功能按钮 -->
          <div class="flex items-start gap-5 pb-4 border-b border-c-default flex-shrink-0">
            <!-- 左侧模组封面大图（点击直接弹窗放大查看） -->
            <div
              v-if="selectedNexus.pictureUrl"
              class="relative w-40 h-28 rounded-xl overflow-hidden flex-shrink-0 border border-c-default cursor-pointer group shadow-sm bg-black/10"
              title="点击查看大图"
              @click="openImagePreview(selectedNexus.pictureUrl!)"
            >
              <img
                :src="selectedNexus.pictureUrl"
                :alt="selectedNexus.name"
                class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                loading="lazy"
              />
              <div class="absolute inset-0 bg-black/0 group-hover:bg-black/15 transition-colors" />
            </div>

            <!-- 右侧核心信息与操作按钮 -->
            <div class="flex-1 min-w-0 flex flex-col justify-between self-stretch">
              <div>
                <div class="flex items-center justify-between gap-3 mb-1.5">
                  <div class="flex items-center gap-2 flex-wrap min-w-0">
                    <h2
                      class="text-xl font-bold text-c-primary tracking-tight truncate max-w-[360px]"
                      :title="selectedNexus.name"
                    >
                      {{ selectedNexus.name }}
                    </h2>
                    <NTag
                      v-if="selectedNexus.latestVersion && selectedNexus.latestVersion !== '—'"
                      size="small"
                      :bordered="false"
                      class="font-mono flex-shrink-0"
                    >
                      v{{ selectedNexus.latestVersion }}
                    </NTag>
                  </div>

                  <!-- 右上角功能按钮组 -->
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <NButton
                      type="primary"
                      size="small"
                      @click="emit('downloadNexus', selectedNexus)"
                    >
                      <template #icon
                        ><NIcon :size="14"><Download /></NIcon
                      ></template>
                      {{ t("discover.download") }}
                    </NButton>
                    <NButton
                      secondary
                      size="small"
                      @click="emit('openDetails', selectedNexus.detailUrl)"
                    >
                      <template #icon
                        ><NIcon :size="14"><ExternalLink /></NIcon
                      ></template>
                      {{ t("discover.details") }}
                    </NButton>
                  </div>
                </div>

                <div class="flex items-center gap-4 text-xs text-c-muted mt-1">
                  <span class="flex items-center gap-1 truncate max-w-[180px]">
                    <NIcon :size="13"><User /></NIcon>
                    {{ selectedNexus.author || t("discover.unknownAuthor") }}
                  </span>
                  <span class="flex items-center gap-1 font-mono">
                    <NIcon :size="13"><ThumbsUp /></NIcon>
                    {{ formatCount(selectedNexus.endorsementCount) }} 点赞
                  </span>
                  <span class="flex items-center gap-1 font-mono">
                    <NIcon :size="13"><ArrowDown /></NIcon>
                    {{ formatCount(selectedNexus.downloadCount) }} 下载
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 下方描述与翻译区域（充分展开纵向展示空间） -->
          <div class="flex-1 p-4 rounded-xl border border-c-default space-y-3" :style="cardStyle">
            <div class="text-xs font-semibold text-c-muted uppercase tracking-wider">模组描述</div>
            <div class="text-sm text-c-primary leading-relaxed whitespace-pre-line">
              <FloatingTip :text="selectedNexus.summary ?? ''" truncated />
            </div>
            <ModTranslateBlock :text="selectedNexus.summary" />
          </div>
        </div>
      </template>

      <!-- Workshop 详情视图 -->
      <template v-else-if="type === 'workshop' && selectedWorkshop">
        <div
          class="flex-1 min-h-0 flex flex-col p-6 scrollbar-thin space-y-4 overflow-y-auto pb-24"
          @wheel.stop
        >
          <!-- 顶部区域：左侧封面大图 + 右侧标题/作者/好评/差评/标签/功能按钮 -->
          <div class="flex items-start gap-5 pb-4 border-b border-c-default flex-shrink-0">
            <!-- 左侧工坊模组封面大图（点击直接弹窗放大查看） -->
            <div
              v-if="selectedWorkshop.previewUrl"
              class="relative w-40 h-28 rounded-xl overflow-hidden flex-shrink-0 border border-c-default cursor-pointer group shadow-sm bg-black/10"
              title="点击查看大图"
              @click="openImagePreview(selectedWorkshop.previewUrl!)"
            >
              <img
                :src="selectedWorkshop.previewUrl"
                :alt="selectedWorkshop.name"
                class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                loading="lazy"
              />
              <div class="absolute inset-0 bg-black/0 group-hover:bg-black/15 transition-colors" />
            </div>

            <!-- 右侧核心信息与操作按钮 -->
            <div class="flex-1 min-w-0 flex flex-col justify-between self-stretch">
              <div>
                <div class="flex items-center justify-between gap-3 mb-1.5">
                  <div class="flex items-center gap-2 flex-wrap min-w-0">
                    <h2
                      class="text-xl font-bold text-c-primary tracking-tight truncate max-w-[360px]"
                      :title="selectedWorkshop.name"
                    >
                      {{ selectedWorkshop.name }}
                    </h2>
                    <NTag
                      v-if="selectedWorkshop.subscribed"
                      type="success"
                      size="small"
                      :bordered="false"
                      class="flex-shrink-0"
                    >
                      {{ t("discover.workshop.subscribed") }}
                    </NTag>
                  </div>

                  <!-- 右上角功能按钮组 -->
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <NButton
                      v-if="selectedWorkshop.subscribed"
                      size="small"
                      secondary
                      :loading="unsubscribingIds?.has(selectedWorkshop.id)"
                      @click="emit('unsubscribeWorkshop', selectedWorkshop.id)"
                    >
                      <template #icon
                        ><NIcon :size="14"><X /></NIcon
                      ></template>
                      {{ t("discover.workshop.unsubscribe") }}
                    </NButton>
                    <NButton
                      v-else
                      type="primary"
                      size="small"
                      :loading="subscribingIds?.has(selectedWorkshop.id)"
                      @click="emit('subscribeWorkshop', selectedWorkshop.id)"
                    >
                      <template #icon
                        ><NIcon :size="14"><Plus /></NIcon
                      ></template>
                      {{ t("discover.workshop.subscribe") }}
                    </NButton>

                    <NButton
                      secondary
                      size="small"
                      @click="
                        emit(
                          'openDetails',
                          `https://steamcommunity.com/sharedfiles/filedetails/?id=${selectedWorkshop.id}`,
                        )
                      "
                    >
                      <template #icon
                        ><NIcon :size="14"><ExternalLink /></NIcon
                      ></template>
                      {{ t("discover.details") }}
                    </NButton>
                  </div>
                </div>

                <div class="flex items-center gap-4 text-xs text-c-muted mt-1">
                  <span class="flex items-center gap-1 truncate max-w-[180px]">
                    <NIcon :size="13"><User /></NIcon>
                    {{ selectedWorkshop.author }}
                  </span>
                  <span class="flex items-center gap-1 font-mono">
                    <NIcon :size="13"><ThumbsUp /></NIcon>
                    {{ formatCount(selectedWorkshop.votesUp) }} 好评
                  </span>
                  <span class="flex items-center gap-1 font-mono">
                    <NIcon :size="13"><ArrowDown /></NIcon>
                    {{ formatCount(selectedWorkshop.votesDown) }} 差评
                  </span>
                </div>

                <!-- 标签列表 -->
                <div v-if="validWorkshopTags.length > 0" class="flex gap-1.5 flex-wrap mt-2">
                  <NTag
                    v-for="tag in validWorkshopTags"
                    :key="tag"
                    size="tiny"
                    :bordered="false"
                    type="default"
                  >
                    {{ tag }}
                  </NTag>
                </div>
              </div>
            </div>
          </div>

          <!-- 下方模组描述与BBCode渲染 -->
          <div class="flex-1 p-4 rounded-xl border border-c-default space-y-3" :style="cardStyle">
            <div class="text-xs font-semibold text-c-muted uppercase tracking-wider">模组描述</div>
            <div
              v-if="parseWorkshopBBCode"
              class="text-sm text-c-primary leading-relaxed bbcode-body"
              v-html="renderBBCode(selectedWorkshop.description ?? '')"
            />
            <div v-else class="text-sm text-c-primary leading-relaxed whitespace-pre-line">
              <FloatingTip :text="selectedWorkshop.description ?? ''" truncated />
            </div>
            <ModTranslateBlock :text="selectedWorkshop.description" />
          </div>
        </div>
      </template>
      <!-- 未选中任何条目提示 -->
      <EmptyState v-else :icon="HardDrive" :title="t('discover.noModSelected')" class="my-auto" />
    </div>
    <!-- 详情大图轻量全屏弹窗预览（与列表图片放大体验保持一致） -->
    <NModal
      :show="previewModalShow"
      :style="{
        backdropFilter: 'blur(var(--blur-backdrop)) saturate(1.3)',
        WebkitBackdropFilter: 'blur(var(--blur-backdrop)) saturate(1.3)',
        background: 'var(--blur-backdrop-bg)',
      }"
      @update:show="(v: boolean) => !v && (previewModalShow = false)"
    >
      <div
        class="flex items-center justify-center cursor-zoom-out"
        style="max-width: 90vw; max-height: 90vh"
        @click="previewModalShow = false"
      >
        <img
          v-if="previewModalImageUrl"
          :src="previewModalImageUrl"
          class="max-w-full max-h-[85vh] rounded-lg shadow-2xl object-contain"
          style="max-width: 85vw"
        />
      </div>
    </NModal>
  </div>
</template>

<style scoped>
.split-list-item {
  background-color: var(--glass-surface-bg) !important;
  border: var(--glass-surface-border) !important;
  box-shadow: var(--shadow-glass-surface) !important;
  backdrop-filter: blur(var(--glass-surface-blur)) saturate(var(--blur-saturate));
  -webkit-backdrop-filter: blur(var(--glass-surface-blur)) saturate(var(--blur-saturate));
  transform: translateZ(0);
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
}
.split-list-item:hover {
  background-color: var(--glass-surface-bg-hover) !important;
  border-color: color-mix(
    in srgb,
    var(--primary-color) 35%,
    var(--glass-surface-border)
  ) !important;
  box-shadow: var(--shadow-card-hover) !important;
}
.split-list-item.active-item {
  border-color: var(--primary-color) !important;
  background-color: color-mix(
    in srgb,
    var(--primary-color) 12%,
    var(--glass-surface-bg)
  ) !important;
  box-shadow: var(--shadow-glow), var(--shadow-card-hover) !important;
}
</style>

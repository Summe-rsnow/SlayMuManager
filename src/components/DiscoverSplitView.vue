<script setup lang="ts">
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import {
  NButton, NIcon, NTag,
} from "naive-ui"
import {
  ThumbsUp, ArrowDown, ExternalLink, Download, Plus, X,
  User, PackageOpen, HardDrive,
} from "@lucide/vue"
import ModTranslateBlock from "./ModTranslateBlock.vue"
import EmptyState from "./EmptyState.vue"
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

function formatCount(num: number): string {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + "M"
  if (num >= 1000) return (num / 1000).toFixed(1) + "k"
  return String(num)
}

const selectedNexus = computed(() => {
  if (props.type !== "nexus" || !props.nexusItems) return null
  return props.nexusItems.find(m => m.remoteId === String(props.selectedId)) || props.nexusItems[0] || null
})

const selectedWorkshop = computed(() => {
  if (props.type !== "workshop" || !props.workshopItems) return null
  return props.workshopItems.find(m => m.id === Number(props.selectedId)) || props.workshopItems[0] || null
})

const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur))",
  borderColor: "var(--glass-surface-border)",
}))
</script>

<template>
  <div class="flex-1 flex gap-4 min-h-0 overflow-hidden h-[calc(100vh-14rem)]">
    <!-- ================= 左侧模组列表 ================= -->
    <div
      class="w-80 md:w-96 flex-shrink-0 flex flex-col rounded-xl border overflow-hidden"
      :style="cardStyle"
    >
      <div class="px-3 py-2 border-b border-c-default flex items-center justify-between text-xs text-c-muted bg-c-secondary/20">
        <span>{{ type === 'nexus' ? 'Nexus Mods' : t('discover.workshop.tab') }}</span>
        <span class="font-mono">
          {{ type === 'nexus' ? (nexusItems?.length || 0) : (workshopItems?.length || 0) }} 项
        </span>
      </div>

      <div class="flex-1 overflow-y-auto p-2 space-y-1.5 scrollbar-thin">
        <!-- Nexus 列表条目 -->
        <template v-if="type === 'nexus'">
          <div
            v-for="item in nexusItems"
            :key="item.remoteId"
            class="split-list-item p-2.5 rounded-lg cursor-pointer transition-all border flex items-center gap-2.5"
            :class="selectedNexus?.remoteId === item.remoteId
              ? 'border-primary-theme bg-primary-theme/12 shadow-sm'
              : 'border-transparent hover:border-c-default hover:bg-c-secondary/30'"
            @click="emit('selectNexus', item)"
          >
            <div class="w-12 h-12 rounded-md overflow-hidden bg-c-secondary/40 flex-shrink-0 relative">
              <img
                v-if="item.pictureUrl"
                :src="item.pictureUrl"
                :alt="item.name"
                class="w-full h-full object-cover"
                loading="lazy"
              />
              <div v-else class="w-full h-full flex items-center justify-center text-c-muted">
                <NIcon :size="20"><PackageOpen /></NIcon>
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="font-semibold text-sm truncate text-c-primary leading-tight mb-1">
                {{ item.name }}
              </div>
              <div class="text-xs text-c-muted truncate mb-1">
                {{ item.author || t('discover.unknownAuthor') }}
                <span v-if="item.latestVersion" class="font-mono text-c-muted"> · v{{ item.latestVersion }}</span>
              </div>
              <div class="flex items-center gap-2 text-[11px] text-c-muted font-mono">
                <span class="flex items-center gap-0.5">
                  <NIcon :size="11"><ThumbsUp /></NIcon>
                  {{ formatCount(item.endorsementCount) }}
                </span>
                <span class="flex items-center gap-0.5">
                  <NIcon :size="11"><ArrowDown /></NIcon>
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
            class="split-list-item p-2.5 rounded-lg cursor-pointer transition-all border flex items-center gap-2.5"
            :class="selectedWorkshop?.id === item.id
              ? 'border-primary-theme bg-primary-theme/12 shadow-sm'
              : 'border-transparent hover:border-c-default hover:bg-c-secondary/30'"
            @click="emit('selectWorkshop', item)"
          >
            <div class="w-12 h-12 rounded-md overflow-hidden bg-c-secondary/40 flex-shrink-0 relative">
              <img
                v-if="item.previewUrl"
                :src="item.previewUrl"
                :alt="item.name"
                class="w-full h-full object-cover"
                loading="lazy"
              />
              <div v-else class="w-full h-full flex items-center justify-center text-c-muted">
                <NIcon :size="20"><PackageOpen /></NIcon>
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-1.5 mb-1">
                <span class="font-semibold text-sm truncate text-c-primary leading-tight flex-1">
                  {{ item.name }}
                </span>
                <NTag v-if="item.subscribed" type="success" size="tiny" :bordered="false" class="flex-shrink-0">
                  {{ t("discover.workshop.subscribed") }}
                </NTag>
              </div>
              <div class="text-xs text-c-muted truncate mb-1">
                {{ item.author }}
              </div>
              <div class="flex items-center gap-2 text-[11px] text-c-muted font-mono">
                <span class="flex items-center gap-0.5">
                  <NIcon :size="11"><ThumbsUp /></NIcon>
                  {{ formatCount(item.votesUp) }}
                </span>
                <span class="flex items-center gap-0.5">
                  <NIcon :size="11"><ArrowDown /></NIcon>
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
      class="flex-1 min-w-0 flex flex-col rounded-xl border overflow-hidden"
      :style="cardStyle"
    >
      <!-- Nexus 详情视图 -->
      <template v-if="type === 'nexus' && selectedNexus">
        <div class="flex-1 overflow-y-auto p-5 scrollbar-thin">
          <!-- 封面图展示 -->
          <div v-if="selectedNexus.pictureUrl" class="w-full max-h-72 rounded-xl overflow-hidden mb-4 bg-c-secondary/20 relative shadow-sm">
            <img
              :src="selectedNexus.pictureUrl"
              :alt="selectedNexus.name"
              class="w-full h-full object-cover max-h-72"
            />
          </div>

          <!-- 标题与核心操作栏 -->
          <div class="flex items-start justify-between gap-4 mb-4 pb-4 border-b border-c-default">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap mb-1.5">
                <h2 class="text-xl font-bold text-c-primary truncate">{{ selectedNexus.name }}</h2>
                <NTag v-if="selectedNexus.latestVersion" size="small" :bordered="false" class="font-mono">
                  v{{ selectedNexus.latestVersion }}
                </NTag>
              </div>
              <div class="flex items-center gap-3 text-xs text-c-muted">
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><User /></NIcon>
                  {{ selectedNexus.author || t('discover.unknownAuthor') }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ThumbsUp /></NIcon>
                  {{ formatCount(selectedNexus.endorsementCount) }} 次点赞
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ArrowDown /></NIcon>
                  {{ formatCount(selectedNexus.downloadCount) }} 次下载
                </span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-2 flex-shrink-0">
              <NButton type="primary" size="medium" @click="emit('downloadNexus', selectedNexus)">
                <template #icon><NIcon :size="16"><Download /></NIcon></template>
                {{ t("discover.download") }}
              </NButton>
              <NButton secondary size="medium" @click="emit('openDetails', selectedNexus.detailUrl)">
                <template #icon><NIcon :size="15"><ExternalLink /></NIcon></template>
                {{ t("discover.details") }}
              </NButton>
            </div>
          </div>

          <!-- 简介与翻译区域 -->
          <div class="space-y-3">
            <div class="text-sm font-semibold text-c-primary">模组描述</div>
            <div class="text-sm text-c-secondary leading-relaxed whitespace-pre-line p-3 rounded-xl bg-c-secondary/20 border border-c-default/40">
              {{ selectedNexus.summary || t("profiles.noDescription") }}
            </div>
            <ModTranslateBlock :text="selectedNexus.summary" />
          </div>
        </div>
      </template>

      <!-- Workshop 详情视图 -->
      <template v-else-if="type === 'workshop' && selectedWorkshop">
        <div class="flex-1 overflow-y-auto p-5 scrollbar-thin">
          <!-- 封面大图 -->
          <div v-if="selectedWorkshop.previewUrl" class="w-full max-h-72 rounded-xl overflow-hidden mb-4 bg-c-secondary/20 relative shadow-sm">
            <img
              :src="selectedWorkshop.previewUrl"
              :alt="selectedWorkshop.name"
              class="w-full h-full object-cover max-h-72"
            />
          </div>

          <!-- 标题与核心操作栏 -->
          <div class="flex items-start justify-between gap-4 mb-4 pb-4 border-b border-c-default">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap mb-1.5">
                <h2 class="text-xl font-bold text-c-primary truncate">{{ selectedWorkshop.name }}</h2>
                <NTag v-if="selectedWorkshop.subscribed" type="success" size="small" :bordered="false">
                  {{ t("discover.workshop.subscribed") }}
                </NTag>
              </div>
              <div class="flex items-center gap-3 text-xs text-c-muted">
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><User /></NIcon>
                  {{ selectedWorkshop.author }}
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ThumbsUp /></NIcon>
                  {{ formatCount(selectedWorkshop.votesUp) }} 好评
                </span>
                <span class="flex items-center gap-1">
                  <NIcon :size="13"><ArrowDown /></NIcon>
                  {{ formatCount(selectedWorkshop.votesDown) }} 差评
                </span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-2 flex-shrink-0">
              <NButton
                v-if="selectedWorkshop.subscribed"
                size="medium"
                secondary
                :loading="unsubscribingIds?.has(selectedWorkshop.id)"
                @click="emit('unsubscribeWorkshop', selectedWorkshop.id)"
              >
                <template #icon><NIcon :size="15"><X /></NIcon></template>
                {{ t("discover.workshop.unsubscribe") }}
              </NButton>
              <NButton
                v-else
                type="primary"
                size="medium"
                :loading="subscribingIds?.has(selectedWorkshop.id)"
                @click="emit('subscribeWorkshop', selectedWorkshop.id)"
              >
                <template #icon><NIcon :size="16"><Plus /></NIcon></template>
                {{ t("discover.workshop.subscribe") }}
              </NButton>

              <NButton
                secondary
                size="medium"
                @click="emit('openDetails', `https://steamcommunity.com/sharedfiles/filedetails/?id=${selectedWorkshop.id}`)"
              >
                <template #icon><NIcon :size="15"><ExternalLink /></NIcon></template>
                {{ t("discover.details") }}
              </NButton>
            </div>
          </div>

          <!-- 标签展示 -->
          <div v-if="selectedWorkshop.tags && selectedWorkshop.tags.length > 0" class="flex gap-1.5 flex-wrap mb-3">
            <NTag v-for="tag in selectedWorkshop.tags" :key="tag" size="tiny" :bordered="false">
              {{ tag }}
            </NTag>
          </div>

          <!-- 详情文本与翻译 -->
          <div class="space-y-3">
            <div class="text-sm font-semibold text-c-primary">模组描述</div>
            <div class="text-sm text-c-secondary leading-relaxed whitespace-pre-line p-3 rounded-xl bg-c-secondary/20 border border-c-default/40">
              {{ selectedWorkshop.description || t("profiles.noDescription") }}
            </div>
            <ModTranslateBlock :text="selectedWorkshop.description" />
          </div>
        </div>
      </template>

      <!-- 未选中提示 -->
      <EmptyState
        v-else
        :icon="HardDrive"
        :title="t('discover.noModSelected')"
        class="my-auto"
      />
    </div>
  </div>
</template>

<style scoped>
.split-list-item:hover {
  background-color: var(--glass-surface-bg-hover) !important;
}
</style>

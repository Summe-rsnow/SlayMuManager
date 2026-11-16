<script setup lang="ts">
import type { Component } from "vue"
import { Card, Image } from "@xcmm1543/miuui-vue"
import { PackageOpen } from "@lucide/vue"
import FloatingTip from "./FloatingTip.vue"
import ModTranslateBlock from "./ModTranslateBlock.vue"

defineProps<{
  name: string
  imageUrl: string | null | undefined
  description: string | null | undefined
  author: string
  version?: string | null
  stats: Array<{ icon: Component; value: string | number }>
}>()
</script>

<template>
  <Card material="glass" interactive padding="sm" class="min-w-0">
    <div class="flex gap-4 flex-1 min-w-0">
      <Image
        :src="imageUrl"
        :alt="name"
        previewable
        referrer-policy="no-referrer"
        radius="md"
        class="w-28 h-28 flex-shrink-0"
      >
        <template #fallback>
          <PackageOpen :size="32" class="text-c-muted" />
        </template>
      </Image>
      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="font-semibold text-base text-c-primary truncate">{{ name }}</span>
            <span v-if="version" class="text-xs font-mono flex-shrink-0 text-c-muted"
              >v{{ version }}</span
            >
          </div>
          <slot name="actions" />
        </div>
        <div class="min-h-0 mt-2">
          <FloatingTip :text="description ?? ''" truncated :max-lines="6" />
          <ModTranslateBlock :text="description" />
        </div>
        <div class="flex items-center gap-3 text-xs pt-2 mt-auto text-c-muted">
          <span class="truncate max-w-[120px]">{{ author }}</span>
          <span v-for="(stat, i) in stats" :key="i" class="flex items-center gap-1 flex-shrink-0">
            <component :is="stat.icon" :size="13" />
            {{ stat.value }}
          </span>
        </div>
      </div>
    </div>
  </Card>
</template>

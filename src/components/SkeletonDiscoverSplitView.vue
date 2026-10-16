<script setup lang="ts">
import { computed } from "vue"

const cardStyle = computed(() => ({
  backgroundColor: "var(--glass-surface-bg)",
  backdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  WebkitBackdropFilter: "blur(var(--glass-surface-blur)) saturate(var(--blur-saturate))",
  border: "var(--glass-surface-border)",
  boxShadow: "var(--shadow-glass-surface)",
}))
</script>

<template>
  <div class="flex-1 flex gap-4 min-h-0 overflow-hidden w-full h-full animate-pulse">
    <!-- 左侧列表骨架 -->
    <div
      class="w-80 md:w-96 flex-shrink-0 flex flex-col rounded-xl overflow-hidden h-full min-h-0"
      :style="cardStyle"
    >
      <div class="flex-1 min-h-0 p-3.5 space-y-2.5 overflow-hidden pb-24">
        <div
          v-for="i in 8"
          :key="'split-skel-left-' + i"
          class="p-3 rounded-xl border border-c-default flex items-center gap-3 bg-c-secondary/15"
        >
          <div class="w-14 h-14 rounded-lg flex-shrink-0 bg-c-secondary" />
          <div class="min-w-0 flex-1 space-y-2">
            <div class="h-4 w-3/4 rounded bg-c-secondary" />
            <div class="h-3 w-1/2 rounded bg-c-secondary" />
            <div class="flex items-center gap-3">
              <div class="h-2.5 w-10 rounded bg-c-secondary" />
              <div class="h-2.5 w-10 rounded bg-c-secondary" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧详情骨架 -->
    <div
      class="flex-1 min-w-0 flex flex-col rounded-xl overflow-hidden h-full min-h-0"
      :style="cardStyle"
    >
      <div class="flex-1 min-h-0 flex flex-col p-6 space-y-4 overflow-hidden pb-24">
        <!-- 顶部左图右信息骨架 -->
        <div class="flex items-start gap-5 pb-4 border-b border-c-default flex-shrink-0">
          <div class="w-40 h-28 rounded-xl flex-shrink-0 bg-c-secondary" />
          <div class="flex-1 min-w-0 flex flex-col justify-between h-28 py-1">
            <div class="flex items-center justify-between gap-3">
              <div class="h-6 w-1/2 rounded bg-c-secondary" />
              <div class="flex items-center gap-2">
                <div class="h-7 w-16 rounded bg-c-secondary" />
                <div class="h-7 w-16 rounded bg-c-secondary" />
              </div>
            </div>
            <div class="flex items-center gap-4">
              <div class="h-3.5 w-24 rounded bg-c-secondary" />
              <div class="h-3.5 w-16 rounded bg-c-secondary" />
              <div class="h-3.5 w-16 rounded bg-c-secondary" />
            </div>
            <div class="flex items-center gap-2">
              <div class="h-5 w-12 rounded bg-c-secondary" />
              <div class="h-5 w-16 rounded bg-c-secondary" />
            </div>
          </div>
        </div>

        <!-- 下方描述区域骨架 -->
        <div class="flex-1 p-4 rounded-xl border border-c-default space-y-3.5" :style="cardStyle">
          <div class="h-4 w-20 rounded bg-c-secondary" />
          <div class="space-y-2 pt-1">
            <div class="h-3.5 w-full rounded bg-c-secondary" />
            <div class="h-3.5 w-full rounded bg-c-secondary" />
            <div class="h-3.5 w-4/5 rounded bg-c-secondary" />
            <div class="h-3.5 w-3/5 rounded bg-c-secondary" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

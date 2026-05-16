<script setup lang="ts">
import { NButton, NIcon } from "naive-ui"
import { useWindow } from "../composables/useWindow"
import { Minus, Square, X } from "lucide-vue-next"

const { minimize, toggleMaximize, close } = useWindow()
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex items-center justify-between h-10 select-none flex-shrink-0"
    :style="{ backgroundColor: 'var(--color-titlebar-bg)', borderBottom: '1px solid var(--color-titlebar-border)' }"
  >
    <div class="flex items-center gap-2 px-4">
      <svg viewBox="0 0 128 128" class="w-4 h-4 flex-shrink-0">
        <defs>
          <linearGradient id="titlelogo" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#6366f1"/>
            <stop offset="100%" stop-color="#8b5cf6"/>
          </linearGradient>
        </defs>
        <rect width="128" height="128" rx="28" fill="url(#titlelogo)"/>
        <text x="64" y="84" font-family="system-ui,sans-serif" font-size="58" font-weight="800" fill="#fff" text-anchor="middle">S</text>
      </svg>
      <span class="text-sm font-semibold text-c-primary">SlayMuManager</span>
    </div>
    <div class="flex h-full">
      <NButton
        text
        class="h-full! w-10! rounded-none!"
        :style="{ color: 'var(--color-text-muted)' }"
        @click="minimize"
      >
        <template #icon>
          <NIcon :size="14"><Minus /></NIcon>
        </template>
      </NButton>
      <NButton
        text
        class="h-full! w-10! rounded-none!"
        :style="{ color: 'var(--color-text-muted)' }"
        @click="toggleMaximize"
      >
        <template #icon>
          <NIcon :size="12"><Square /></NIcon>
        </template>
      </NButton>
      <NButton
        text
        class="h-full! w-10! rounded-none! hover:bg-red-500!"
        :style="{ color: 'var(--color-text-muted)' }"
        @click="close"
      >
        <template #icon>
          <NIcon :size="16"><X /></NIcon>
        </template>
      </NButton>
    </div>
  </div>
</template>

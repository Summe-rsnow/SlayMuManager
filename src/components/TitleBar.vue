<script setup lang="ts">
import { NButton, NIcon } from "naive-ui"
import { useWindow } from "../composables/useWindow"
import { Minus, Square, X } from "lucide-vue-next"

const { minimize, toggleMaximize, close } = useWindow()
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex items-center justify-between h-10 bg-gray-50 border-b border-gray-100 select-none flex-shrink-0"
  >
    <div class="flex items-center gap-2 px-4">
      <span class="text-sm font-semibold text-gray-700">SlayMuManager</span>
    </div>
    <div class="flex h-full">
      <NButton
        text
        class="h-full! w-10! rounded-none! c-gray-500 hover:c-gray-700 hover:bg-gray-200!"
        @click="minimize"
      >
        <template #icon>
          <NIcon :size="14"><Minus /></NIcon>
        </template>
      </NButton>
      <NButton
        text
        class="h-full! w-10! rounded-none! c-gray-500 hover:c-gray-700 hover:bg-gray-200!"
        @click="toggleMaximize"
      >
        <template #icon>
          <NIcon :size="12"><Square /></NIcon>
        </template>
      </NButton>
      <NButton
        text
        class="h-full! w-10! rounded-none! c-gray-500 hover:c-white hover:bg-red-500!"
        @click="close"
      >
        <template #icon>
          <NIcon :size="16"><X /></NIcon>
        </template>
      </NButton>
    </div>
  </div>
</template>

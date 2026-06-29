<script setup lang="ts">
import { NIcon } from "naive-ui"
import { Loader2 } from "@lucide/vue"

defineProps<{
  loading: boolean
  text?: string
}>()
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="loading" class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm">
        <div class="flex flex-col items-center gap-3">
          <NIcon :size="32" class="animate-spin text-primary"><Loader2 /></NIcon>
          <p v-if="text" class="text-sm text-c-primary">{{ text }}</p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

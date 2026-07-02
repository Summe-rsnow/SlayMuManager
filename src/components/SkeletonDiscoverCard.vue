<template>
  <div class="break-inside-avoid mb-4 rounded-2xl overflow-hidden" :style="{ backgroundColor: 'var(--color-bg-card)', border: '1px solid var(--color-border)' }">
    <div class="flex gap-4 p-4 h-full">
      <div class="skeleton-shimmer w-20 h-20 rounded-lg flex-shrink-0" />
      <div class="flex-1 min-w-0 space-y-2">
        <div class="skeleton-shimmer h-4 w-3/4" />
        <div class="skeleton-shimmer h-3 w-full" />
        <div class="skeleton-shimmer h-3 w-1/2" />
      </div>
    </div>
  </div>
</template>

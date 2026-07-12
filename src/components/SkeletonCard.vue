<template>
  <div class="flex items-center justify-between p-3 rounded-xl border border-l-3" :style="{ borderLeftColor: 'var(--color-text-muted)', borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg-secondary)' }">
    <div class="flex-1 min-w-0 space-y-2">
      <div class="flex items-center gap-2">
        <div class="skeleton-shimmer h-4 w-40" />
        <div class="skeleton-shimmer h-2.5 w-12" />
      </div>
      <div class="skeleton-shimmer h-2.5 w-24" />
    </div>
    <div class="flex items-center gap-2 ml-4">
      <div class="skeleton-shimmer h-6 w-6 rounded" />
      <div class="skeleton-shimmer h-5 w-10 rounded" />
    </div>
  </div>
</template>

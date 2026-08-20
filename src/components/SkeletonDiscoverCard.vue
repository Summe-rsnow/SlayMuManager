<template>
  <div class="glass-surface break-inside-avoid mb-4 rounded-xl overflow-hidden p-4" :style="{ minHeight: '150px' }">
    <div class="flex gap-4 h-full animate-pulse">
      <div class="w-28 h-28 rounded-lg flex-shrink-0 bg-c-secondary" />
      <div class="flex-1 flex flex-col min-w-0">
        <!-- 标题行：名称 + 版本号 + 操作按钮 -->
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <div class="h-5 w-2/3 rounded bg-c-secondary" />
            <div class="h-4 w-10 rounded bg-c-secondary flex-shrink-0" />
          </div>
          <div class="h-7 w-16 rounded bg-c-secondary flex-shrink-0" />
        </div>
        <!-- 描述区 -->
        <div class="min-h-0 mt-2 flex flex-col gap-1.5">
          <div class="h-3 rounded w-full bg-c-secondary" />
          <div class="h-3 rounded w-full bg-c-secondary" />
          <div class="h-3 rounded w-5/6 bg-c-secondary" />
        </div>
        <!-- 底部：作者 + 统计 -->
        <div class="flex items-center gap-3 pt-3 mt-auto">
          <div class="h-3.5 w-16 rounded bg-c-secondary" />
          <div class="h-3.5 w-20 rounded bg-c-secondary" />
          <div class="h-3.5 w-16 rounded bg-c-secondary" />
        </div>
      </div>
    </div>
  </div>
</template>

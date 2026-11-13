<script setup lang="ts">
import { Card, Skeleton } from "@miuui/vue"
</script>

<template>
  <Card material="glass" padding="sm" class="min-w-0">
    <div class="flex gap-4 flex-1 min-w-0">
      <Skeleton variant="rect" width="112px" height="112px" radius="md" class="flex-shrink-0" />
      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <Skeleton variant="text" width="66%" height="20px" />
            <Skeleton variant="text" width="40px" height="16px" class="flex-shrink-0" />
          </div>
          <Skeleton variant="rect" width="64px" height="28px" radius="sm" class="flex-shrink-0" />
        </div>
        <div class="min-h-0 mt-2 flex flex-col gap-1.5">
          <Skeleton variant="text" width="100%" height="12px" />
          <Skeleton variant="text" width="100%" height="12px" />
          <Skeleton variant="text" width="82%" height="12px" />
        </div>
        <div class="flex items-center gap-3 pt-2 mt-auto">
          <Skeleton variant="text" width="64px" height="14px" />
          <Skeleton variant="text" width="80px" height="14px" />
          <Skeleton variant="text" width="64px" height="14px" />
        </div>
      </div>
    </div>
  </Card>
</template>

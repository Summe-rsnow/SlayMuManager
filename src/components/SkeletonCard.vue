<script setup lang="ts">
import { Card, Skeleton } from "@miuui/vue"
</script>

<template>
  <Card material="glass" padding="sm">
    <div class="flex items-center justify-between">
      <div class="flex-1 min-w-0 flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <Skeleton variant="text" width="160px" height="16px" />
          <Skeleton variant="text" width="48px" height="10px" />
        </div>
        <Skeleton variant="text" width="96px" height="10px" />
      </div>
      <div class="flex items-center gap-2 ml-4">
        <Skeleton variant="rect" width="24px" height="24px" radius="sm" />
        <Skeleton variant="rect" width="40px" height="20px" radius="sm" />
      </div>
    </div>
  </Card>
</template>

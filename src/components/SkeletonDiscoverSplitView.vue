<script setup lang="ts">
import { Card, Skeleton } from "@miuui/vue"
</script>

<template>
  <div class="flex-1 flex gap-4 min-h-0 overflow-hidden w-full h-full">
    <Card
      material="glass"
      padding="none"
      class="w-80 md:w-96 flex-shrink-0 h-full min-h-0 overflow-hidden"
    >
      <div class="flex-1 min-h-0 p-3.5 space-y-2.5 overflow-hidden pb-24">
        <Card v-for="i in 8" :key="'split-skel-left-' + i" material="glass" padding="sm">
          <div class="flex items-center gap-3">
            <Skeleton variant="rect" width="56px" height="56px" radius="md" class="flex-shrink-0" />
            <div class="min-w-0 flex-1 space-y-2">
              <Skeleton variant="text" width="75%" height="16px" />
              <Skeleton variant="text" width="50%" height="12px" />
              <div class="flex items-center gap-3">
                <Skeleton variant="text" width="40px" height="10px" />
                <Skeleton variant="text" width="40px" height="10px" />
              </div>
            </div>
          </div>
        </Card>
      </div>
    </Card>

    <Card material="glass" padding="none" class="flex-1 min-w-0 h-full min-h-0 overflow-hidden">
      <div class="flex-1 min-h-0 flex flex-col p-6 space-y-4 overflow-hidden pb-24">
        <div class="flex items-start gap-5 pb-4 border-b border-c-default flex-shrink-0">
          <Skeleton variant="rect" width="160px" height="112px" radius="lg" class="flex-shrink-0" />
          <div class="flex-1 min-w-0 flex flex-col justify-between h-28 py-1">
            <div class="flex items-center justify-between gap-3">
              <Skeleton variant="text" width="50%" height="24px" />
              <div class="flex items-center gap-2">
                <Skeleton variant="rect" width="64px" height="28px" radius="sm" />
                <Skeleton variant="rect" width="64px" height="28px" radius="sm" />
              </div>
            </div>
            <div class="flex items-center gap-4">
              <Skeleton variant="text" width="96px" height="14px" />
              <Skeleton variant="text" width="64px" height="14px" />
              <Skeleton variant="text" width="64px" height="14px" />
            </div>
            <div class="flex items-center gap-2">
              <Skeleton variant="rect" width="48px" height="20px" radius="sm" />
              <Skeleton variant="rect" width="64px" height="20px" radius="sm" />
            </div>
          </div>
        </div>

        <Card material="glass" padding="sm" class="flex-1">
          <Skeleton variant="text" width="80px" height="16px" />
          <div class="space-y-2 pt-3">
            <Skeleton variant="text" width="100%" height="14px" />
            <Skeleton variant="text" width="100%" height="14px" />
            <Skeleton variant="text" width="80%" height="14px" />
            <Skeleton variant="text" width="60%" height="14px" />
          </div>
        </Card>
      </div>
    </Card>
  </div>
</template>

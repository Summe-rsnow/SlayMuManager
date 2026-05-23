<script setup lang="ts">
defineProps<{
  text: string | null | undefined
}>()
</script>

<template>
  <p v-if="text" class="text-xs text-c-muted">{{ text }}</p>
  <div v-else style="min-height: 1.125rem" />
</template>

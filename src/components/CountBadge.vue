<script setup lang="ts">
withDefaults(defineProps<{
  label: string
  count: number
  dotColor?: string
}>(), {
  dotColor: "bg-c-muted",
})
</script>

<template>
  <span class="flex items-center gap-1.5 whitespace-nowrap">
    <span class="w-2 h-2 rounded-full inline-block flex-shrink-0" :class="dotColor" />
    {{ label }} {{ count }}
  </span>
</template>

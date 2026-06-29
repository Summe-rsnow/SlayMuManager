<script setup lang="ts">
import { ref } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { NIcon, NPopover } from "naive-ui"
import { Languages } from "@lucide/vue"
import { translateText } from "@/utils/translate"
import { currentLocale } from "@/i18n"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"

const props = defineProps<{
  text: string | null | undefined
}>()

const { t } = useI18n()
const { showTranslateQuotaTip } = storeToRefs(useDiscoverPrefStore())

const translatedText = ref("")
const translating = ref(false)
const showTranslation = ref(false)

async function handleTranslate() {
  if (!props.text || translating.value) return
  translating.value = true
  try {
    const result = await translateText(props.text)
    if (result.ok) {
      translatedText.value = result.text
      showTranslation.value = true
    }
  } finally {
    translating.value = false
  }
}

function toggleTranslation() {
  showTranslation.value = !showTranslation.value
}
</script>

<template>
  <div v-if="text && currentLocale === 'zh-CN'" class="flex flex-wrap items-start gap-x-1.5 mt-1">
    <template v-if="translatedText">
      <button class="translate-toggle" @click="toggleTranslation">
        <NIcon :size="12"><Languages /></NIcon>
        {{ showTranslation ? t("discover.showOriginal") : t("discover.translate") }}
      </button>
      <p v-if="showTranslation" class="text-xs leading-relaxed w-full mt-0.5 text-c-secondary">
        {{ translatedText }}
      </p>
    </template>
    <span v-else-if="translating" class="text-xs text-c-muted">{{ t("discover.translating") }}</span>
    <NPopover v-else trigger="hover" placement="top" :width="240" :disabled="!showTranslateQuotaTip">
      <template #trigger>
        <button class="translate-toggle" @click="handleTranslate">
          <NIcon :size="12"><Languages /></NIcon>
          {{ t("discover.translate") }}
        </button>
      </template>
      <span class="text-xs">{{ t("discover.translateQuota") }}</span>
    </NPopover>
  </div>
</template>

<style scoped>
.translate-toggle {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: 0.7rem;
  line-height: 1;
  padding: 1px 5px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  color: var(--primary-color);
  background-color: color-mix(in srgb, var(--primary-color) 8%, transparent);
  transition: background-color 0.15s;
  white-space: nowrap;
}
.translate-toggle:hover {
  background-color: color-mix(in srgb, var(--primary-color) 18%, transparent);
}
</style>

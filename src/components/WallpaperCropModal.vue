<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue"
import { useI18n } from "vue-i18n"
import { NButton, NIcon, NSlider, NSpace, NRadioGroup, NRadioButton } from "naive-ui"
import { ZoomIn, ZoomOut, RotateCcw, Maximize2, Minimize2, Crop } from "@lucide/vue"
import AppDialog from "./AppDialog.vue"

const props = withDefaults(
  defineProps<{
    show: boolean
    imageUrl: string
    initialFillColor?: string
    mode?: "crop" | "preview"
    title?: string
  }>(),
  {
    mode: "crop",
  },
)
const emit = defineEmits<{
  (e: "update:show", v: boolean): void
  (e: "apply", payload: { croppedUrl: string; originalUrl: string; fillColor: string }): void
}>()

const { t } = useI18n()

// 画面比例选项
type AspectRatioKey = "16:9" | "16:10" | "4:3" | "window"
const selectedRatio = ref<AspectRatioKey>("16:9")

const ratioOptions: { label: string; value: AspectRatioKey }[] = [
  { label: "16:9", value: "16:9" },
  { label: "16:10", value: "16:10" },
  { label: "4:3", value: "4:3" },
  { label: "自适应", value: "window" },
]

// 预设背景填充色：仅保留深色和浅色（移除纯黑），以及自定义
const DARK_FILL = "#12121A"
const LIGHT_FILL = "#F4F5F7"

const fillColor = ref<string>(props.initialFillColor || DARK_FILL)

const colorPresets = [
  { key: "dark", color: DARK_FILL, labelKey: "settings.appearance.cropDarkFill" },
  { key: "light", color: LIGHT_FILL, labelKey: "settings.appearance.cropLightFill" },
]

// 变换状态
const zoom = ref(1)
const offsetX = ref(0)
const offsetY = ref(0)
const saving = ref(false)

// 视口容器基准宽度
const VIEWPORT_WIDTH = 620

const currentAspectRatio = computed(() => {
  switch (selectedRatio.value) {
    case "16:10":
      return 16 / 10
    case "4:3":
      return 4 / 3
    case "window":
      return window.innerWidth > 0 && window.innerHeight > 0
        ? window.innerWidth / window.innerHeight
        : 16 / 9
    case "16:9":
    default:
      return 16 / 9
  }
})

const viewportHeight = computed(() => {
  const rawH = Math.round(VIEWPORT_WIDTH / currentAspectRatio.value)
  return Math.min(360, Math.max(220, rawH))
})

// 图片尺寸状态
const naturalWidth = ref(0)
const naturalHeight = ref(0)
const imgRef = ref<HTMLImageElement | null>(null)

function onImageLoad(e: Event) {
  const target = e.target as HTMLImageElement | null
  if (!target) return
  naturalWidth.value = target.naturalWidth
  naturalHeight.value = target.naturalHeight
  fitToFrame()
}

// 适应模式：缩放至完整容纳在视口中
function fitToFrame() {
  zoom.value = 1
  offsetX.value = 0
  offsetY.value = 0
}

// 填满模式：缩放至完全覆盖视口
function fillFrame() {
  if (naturalWidth.value <= 0 || naturalHeight.value <= 0) return
  const fitScale = Math.min(
    VIEWPORT_WIDTH / naturalWidth.value,
    viewportHeight.value / naturalHeight.value,
  )
  const baseW = naturalWidth.value * fitScale
  const baseH = naturalHeight.value * fitScale
  const fillScale = Math.max(VIEWPORT_WIDTH / baseW, viewportHeight.value / baseH)
  zoom.value = Math.max(1, Math.round(fillScale * 100) / 100)
  offsetX.value = 0
  offsetY.value = 0
}

// 基准渲染尺寸（zoom = 1 时 fit 在容器中）
const baseDimensions = computed(() => {
  if (naturalWidth.value <= 0 || naturalHeight.value <= 0) {
    return { width: VIEWPORT_WIDTH, height: viewportHeight.value }
  }
  const fitScale = Math.min(
    VIEWPORT_WIDTH / naturalWidth.value,
    viewportHeight.value / naturalHeight.value,
  )
  return {
    width: naturalWidth.value * fitScale,
    height: naturalHeight.value * fitScale,
  }
})

// 鼠标拖拽平移
const isDragging = ref(false)
let dragStartX = 0
let dragStartY = 0
let initialOffsetX = 0
let initialOffsetY = 0

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  isDragging.value = true
  dragStartX = e.clientX
  dragStartY = e.clientY
  initialOffsetX = offsetX.value
  initialOffsetY = offsetY.value
  window.addEventListener("mousemove", onMouseMove)
  window.addEventListener("mouseup", onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  offsetX.value = initialOffsetX + (e.clientX - dragStartX)
  offsetY.value = initialOffsetY + (e.clientY - dragStartY)
}

function onMouseUp() {
  isDragging.value = false
  window.removeEventListener("mousemove", onMouseMove)
  window.removeEventListener("mouseup", onMouseUp)
}

// 鼠标滚轮缩放
function onWheel(e: WheelEvent) {
  e.preventDefault()
  const factor = e.deltaY < 0 ? 1.08 : 0.92
  const next = Math.max(0.2, Math.min(3.5, zoom.value * factor))
  zoom.value = Math.round(next * 100) / 100
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      if (props.initialFillColor) {
        fillColor.value = props.initialFillColor === "#000000" ? DARK_FILL : props.initialFillColor
      }
      fitToFrame()
    }
  },
)

onUnmounted(() => {
  window.removeEventListener("mousemove", onMouseMove)
  window.removeEventListener("mouseup", onMouseUp)
})

// 确认导出
async function onConfirm() {
  if (!imgRef.value || naturalWidth.value <= 0 || naturalHeight.value <= 0) return
  saving.value = true

  try {
    const targetWidth = 1920
    const targetHeight = Math.round(1920 / currentAspectRatio.value)

    const canvas = document.createElement("canvas")
    canvas.width = targetWidth
    canvas.height = targetHeight
    const ctx = canvas.getContext("2d")
    if (!ctx) return

    // 1. 填充超出边界底色
    ctx.fillStyle = fillColor.value
    ctx.fillRect(0, 0, targetWidth, targetHeight)

    // 2. 计算绘制目标位置与尺寸
    const m = targetWidth / VIEWPORT_WIDTH
    const drawW = baseDimensions.value.width * zoom.value * m
    const drawH = baseDimensions.value.height * zoom.value * m
    const drawX = (targetWidth - drawW) / 2 + offsetX.value * m
    const drawY = (targetHeight - drawH) / 2 + offsetY.value * m

    ctx.drawImage(imgRef.value, drawX, drawY, drawW, drawH)

    const croppedUrl = canvas.toDataURL("image/jpeg", 0.92)
    emit("apply", {
      croppedUrl,
      originalUrl: props.imageUrl,
      fillColor: fillColor.value,
    })
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <AppDialog :show="show" width="680px" @update:show="(v) => emit('update:show', v)">
    <template #header>
      <div class="flex items-center gap-2">
        <NIcon :size="20" class="text-primary-theme">
          <component :is="mode === 'preview' ? Maximize2 : Crop" />
        </NIcon>
        <span class="text-base font-bold text-c-primary">
          {{
            title ||
            (mode === "preview"
              ? t("discover.imagePreview", "图片大图预览")
              : t("settings.appearance.cropModalTitle"))
          }}
        </span>
      </div>
    </template>

    <div class="flex flex-col gap-4 select-none pb-1">
      <!-- 视口工作区 -->
      <div class="flex justify-center">
        <div
          class="crop-viewport relative overflow-hidden rounded-xl border border-c-default flex items-center justify-center cursor-grab active:cursor-grabbing shadow-sm"
          :style="{
            width: VIEWPORT_WIDTH + 'px',
            height: viewportHeight + 'px',
            backgroundColor: fillColor,
            transition: 'background-color 0.2s ease, height 0.2s ease',
          }"
          @mousedown="onMouseDown"
          @wheel="onWheel"
        >
          <!-- 实际受控图片 -->
          <img
            v-if="imageUrl"
            ref="imgRef"
            :src="imageUrl"
            alt="crop-preview"
            draggable="false"
            class="pointer-events-none transition-transform duration-75 select-none"
            :style="{
              width: baseDimensions.width + 'px',
              height: baseDimensions.height + 'px',
              transform: `translate(${offsetX}px, ${offsetY}px) scale(${zoom})`,
              transformOrigin: 'center center',
            }"
            @load="onImageLoad"
          />

          <!-- 九宫格构图辅助线（仅裁切模式显示） -->
          <div
            v-if="mode === 'crop'"
            class="absolute inset-0 pointer-events-none grid grid-cols-3 grid-rows-3 opacity-15"
          >
            <div class="border-r border-b border-white" />
            <div class="border-r border-b border-white" />
            <div class="border-b border-white" />
            <div class="border-r border-b border-white" />
            <div class="border-r border-b border-white" />
            <div class="border-b border-white" />
            <div class="border-r border-b border-white" />
            <div class="border-r border-b border-white" />
            <div />
          </div>
          <!-- 右上角快捷控制按钮组 -->
          <div
            class="absolute right-2.5 top-2.5 z-10 flex items-center gap-1 p-1 rounded-lg bg-black/50 backdrop-blur-md border border-white/10"
          >
            <NButton
              size="tiny"
              quaternary
              :title="t('settings.appearance.cropFit')"
              @click.stop="fitToFrame"
            >
              <template #icon
                ><NIcon :size="14"><Minimize2 /></NIcon
              ></template>
            </NButton>
            <NButton
              size="tiny"
              quaternary
              :title="t('settings.appearance.cropFill')"
              @click.stop="fillFrame"
            >
              <template #icon
                ><NIcon :size="14"><Maximize2 /></NIcon
              ></template>
            </NButton>
            <NButton
              size="tiny"
              quaternary
              :title="t('settings.appearance.cropCenter')"
              @click.stop="fitToFrame"
            >
              <template #icon
                ><NIcon :size="14"><RotateCcw /></NIcon
              ></template>
            </NButton>
          </div>
        </div>
      </div>

      <!-- 操作控制面板：复用 Naive UI 现有原生组件 -->
      <div class="flex flex-col gap-3.5 px-1 pt-1">
        <!-- 比例与底色行（仅裁切模式显示） -->
        <div v-if="mode === 'crop'" class="flex items-center justify-between gap-4 flex-wrap">
          <!-- 画面比例切换 -->
          <div class="flex items-center gap-2">
            <span class="text-xs text-c-secondary font-medium">{{
              t("settings.appearance.cropRatio")
            }}</span>
            <NRadioGroup v-model:value="selectedRatio" size="small" name="aspect-ratio-group">
              <NRadioButton v-for="opt in ratioOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </NRadioButton>
            </NRadioGroup>
          </div>

          <!-- 留白填充底色选择（仅深色、浅色、自定义） -->
          <div class="flex items-center gap-2">
            <span class="text-xs text-c-secondary font-medium">{{
              t("settings.appearance.cropFillColor")
            }}</span>
            <div class="flex items-center gap-1.5">
              <NButton
                v-for="p in colorPresets"
                :key="p.key"
                size="small"
                :secondary="fillColor === p.color"
                :type="fillColor === p.color ? 'primary' : 'default'"
                @click="fillColor = p.color"
              >
                <template #icon>
                  <span
                    class="w-3 h-3 rounded-full border border-black/20 flex-shrink-0"
                    :style="{ backgroundColor: p.color }"
                  />
                </template>
                {{ t(p.labelKey) }}
              </NButton>

              <!-- 自定义颜色选择 -->
              <label class="relative inline-flex items-center">
                <NButton
                  size="small"
                  :secondary="!colorPresets.some((p) => p.color === fillColor)"
                  :type="!colorPresets.some((p) => p.color === fillColor) ? 'primary' : 'default'"
                >
                  <template #icon>
                    <span
                      class="w-3 h-3 rounded-full border border-black/20 flex-shrink-0"
                      :style="{ backgroundColor: fillColor }"
                    />
                  </template>
                  {{ t("settings.appearance.cropCustomFill") }}
                </NButton>
                <input
                  v-model="fillColor"
                  type="color"
                  class="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                />
              </label>
            </div>
          </div>
        </div>
        <!-- 画面缩放控制行 -->
        <div class="flex items-center gap-3">
          <NIcon :size="16" class="text-c-muted"><ZoomOut /></NIcon>
          <div class="flex-1">
            <NSlider v-model:value="zoom" :min="0.2" :max="3.0" :step="0.02" size="small" />
          </div>
          <NIcon :size="16" class="text-c-muted"><ZoomIn /></NIcon>
          <span class="text-xs font-mono text-c-muted w-12 text-right"
            >{{ Math.round(zoom * 100) }}%</span
          >
        </div>
      </div>
    </div>

    <!-- 弹窗底部操作栏（明确提供取消与确认按钮，保证舒适间隙） -->
    <!-- 弹窗底部操作栏 -->
    <template #footer>
      <div class="flex items-center justify-between w-full pt-1">
        <span class="text-xs text-c-muted">
          {{
            mode === "preview"
              ? "滚轮缩放，按住鼠标拖拽平移，右上角可一键居中适应"
              : t("settings.appearance.cropModalHint")
          }}
        </span>
        <NSpace :size="12">
          <NButton size="small" secondary @click="emit('update:show', false)">
            {{ mode === "preview" ? t("common.close", "关闭") : t("common.cancel") }}
          </NButton>
          <NButton
            v-if="mode === 'crop'"
            size="small"
            type="primary"
            :loading="saving"
            @click="onConfirm"
          >
            {{ t("settings.appearance.cropApply") }}
          </NButton>
        </NSpace>
      </div>
    </template>
  </AppDialog>
</template>

import { ref, onBeforeUnmount } from "vue"

/**
 * 组件生命周期守卫 — 防止切换页面时异步回调继续执行。
 * 在异步函数的关键步骤前调用 `if (!isActive.value) return` 即可安全中止。
 *
 * @example
 * ```ts
 * const { isActive } = useIsActive()
 * async function doSomething() {
 *   if (!isActive.value) return
 *   // ...
 * }
 * ```
 */
export function useIsActive() {
  const isActive = ref(true)
  onBeforeUnmount(() => { isActive.value = false })
  return { isActive }
}

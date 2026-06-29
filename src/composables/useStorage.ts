import { ref, watch, type Ref } from "vue"

/**
 * 统一的 localStorage 持久化 composable
 *
 * - JSON 序列化/反序列化自动处理
 * - try/catch 包裹，损坏数据安全回退到默认值
 * - 通过 `deep: true` watch 自动持久化（对象/数组的深层变更自动保存）
 *
 * @param key    localStorage key（保持一致前缀）
 * @param defaultValue  默认值（localStorage 无数据时使用）
 */
export function useStorage<T>(key: string, defaultValue: T): Ref<T> {
  const data = ref<T>(defaultValue) as Ref<T>

  // 初始化：从 localStorage 读取
  try {
    const raw = localStorage.getItem(key)
    if (raw !== null) {
      data.value = JSON.parse(raw)
    }
  } catch {
    // 数据损坏，静默使用默认值
  }

  // 自动持久化：数据变化时写入 localStorage
  watch(data, (val) => {
    try {
      localStorage.setItem(key, JSON.stringify(val))
    } catch {
      // 配额超限等，静默忽略
    }
  }, { deep: true })

  return data
}

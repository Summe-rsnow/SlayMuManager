interface SchedulerLike {
  yield?: () => Promise<void>
  postTask?: (
    callback: () => void,
    options?: { priority?: "user-blocking" | "user-visible" | "background" },
  ) => Promise<unknown>
}

function getScheduler(): SchedulerLike | undefined {
  return (window as Window & { scheduler?: SchedulerLike }).scheduler
}

/**
 * 让出主线程以防止长任务阻塞输入与渲染（优先使用 scheduler.yield()，不支持时回退到 setTimeout）
 */
export async function yieldToMain(): Promise<void> {
  const scheduler = getScheduler()
  if (scheduler && typeof scheduler.yield === "function") {
    await scheduler.yield()
    return
  }
  await new Promise<void>((resolve) => setTimeout(resolve, 0))
}

/**
 * 以低优先级调度非关键后台任务（如相邻页预取），不抢占当前视图渲染与交互
 */
export function scheduleBackgroundTask(task: () => void): void {
  const scheduler = getScheduler()
  if (scheduler && typeof scheduler.postTask === "function") {
    void scheduler.postTask(task, { priority: "background" }).catch(() => {
      // 忽略被取消的后台任务
    })
    return
  }
  setTimeout(task, 0)
}

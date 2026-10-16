interface PromiseWithResolvers<T> {
  promise: Promise<T>
  resolve: (value: T | PromiseLike<T>) => void
  reject: (reason?: unknown) => void
}

function withResolvers<T>(): PromiseWithResolvers<T> {
  let resolve!: (value: T | PromiseLike<T>) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

const DB_NAME = "slaymgr_wallpaper_store"
const DB_VERSION = 1
const STORE_NAME = "wallpaper"

function openDB(): Promise<IDBDatabase> {
  if (typeof indexedDB === "undefined") {
    return Promise.reject(new Error("IndexedDB is not supported in this environment"))
  }
  const { promise, resolve, reject } = withResolvers<IDBDatabase>()

  const req = indexedDB.open(DB_NAME, DB_VERSION)
  req.onupgradeneeded = () => {
    const db = req.result
    if (!db.objectStoreNames.contains(STORE_NAME)) {
      db.createObjectStore(STORE_NAME)
    }
  }
  req.onsuccess = () => resolve(req.result)
  req.onerror = () => reject(req.error)
  return promise
}

export async function saveWallpaperToIDB(active: string, original?: string): Promise<void> {
  try {
    const db = await openDB()
    const { promise, resolve, reject } = withResolvers<void>()

    const tx = db.transaction(STORE_NAME, "readwrite")
    const store = tx.objectStore(STORE_NAME)
    store.put(active, "active")
    if (original) {
      store.put(original, "original")
    }
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
    return await promise
  } catch (e: unknown) {
    console.error("Failed to save wallpaper to IndexedDB:", e)
  }
}

export async function getWallpaperFromIDB(): Promise<{
  active: string | null
  original: string | null
}> {
  try {
    const db = await openDB()
    const { promise, resolve, reject } = withResolvers<{
      active: string | null
      original: string | null
    }>()

    const tx = db.transaction(STORE_NAME, "readonly")
    const store = tx.objectStore(STORE_NAME)
    const reqActive = store.get("active")
    const reqOriginal = store.get("original")
    tx.oncomplete = () => {
      resolve({
        active: typeof reqActive.result === "string" ? reqActive.result : null,
        original: typeof reqOriginal.result === "string" ? reqOriginal.result : null,
      })
    }
    tx.onerror = () => reject(tx.error)
    return await promise
  } catch {
    return { active: null, original: null }
  }
}

export async function clearWallpaperFromIDB(): Promise<void> {
  try {
    const db = await openDB()
    const { promise, resolve, reject } = withResolvers<void>()

    const tx = db.transaction(STORE_NAME, "readwrite")
    const store = tx.objectStore(STORE_NAME)
    store.delete("active")
    store.delete("original")
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
    return await promise
  } catch {
    // ignore
  }
}

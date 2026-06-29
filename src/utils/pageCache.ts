import type { RemoteMod } from "../types"

interface CacheEntry { items: RemoteMod[]; totalCount: number }

function makeKey(query: string, sortBy: string, page: number, pageSize: number): string {
  return `${query}|${sortBy}|${page}|${pageSize}`
}

const cache = new Map<string, CacheEntry>()

export function getPageCache(
  query: string, sortBy: string, page: number, pageSize: number
): CacheEntry | undefined {
  return cache.get(makeKey(query, sortBy, page, pageSize))
}

export function setPageCache(
  query: string, sortBy: string, page: number, pageSize: number,
  items: RemoteMod[], totalCount: number
) {
  const key = makeKey(query, sortBy, page, pageSize)
  if (cache.size >= 30) {
    const first = cache.keys().next().value
    if (first) cache.delete(first)
  }
  cache.set(key, { items, totalCount })
}

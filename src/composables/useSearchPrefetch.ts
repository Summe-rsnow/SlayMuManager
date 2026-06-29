interface CacheEntry<T> { items: T[]; totalCount: number }

function makeKey(prefix: string, query: string, sortBy: string, page: number, pageSize: number): string {
  return `${prefix}|${query}|${sortBy}|${page}|${pageSize}`
}

export function useSearchPrefetch<T>(prefix: string) {
  const cache = new Map<string, CacheEntry<T>>()

  function getCache(query: string, sortBy: string, page: number, pageSize: number): CacheEntry<T> | undefined {
    return cache.get(makeKey(prefix, query, sortBy, page, pageSize))
  }

  function setCache(query: string, sortBy: string, page: number, pageSize: number, items: T[], totalCount: number) {
    const key = makeKey(prefix, query, sortBy, page, pageSize)
    if (cache.size >= 30) {
      const first = cache.keys().next().value
      if (first) cache.delete(first)
    }
    cache.set(key, { items, totalCount })
  }

  function prefetchAdjacent(
    q: string, sb: string, currentPage: number, ps: number,
    fetchSilent: (q: string, sb: string, p: number, ps: number) => void
  ) {
    const pages = [currentPage - 1, currentPage + 1]
    for (const p of pages) {
      if (p < 1) continue
      if (getCache(q, sb, p, ps)) continue
      fetchSilent(q, sb, p, ps)
    }
  }

  return { getCache, setCache, prefetchAdjacent }
}

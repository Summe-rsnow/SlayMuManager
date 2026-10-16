/**
 * 安全高效的 Steam 创意工坊 BBCode 解析器
 */

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
}

function sanitizeUrl(url: string): string {
  const trimmed = url.trim()
  if (
    trimmed.startsWith("http://") ||
    trimmed.startsWith("https://") ||
    trimmed.startsWith("steam://")
  ) {
    return trimmed
  }
  return "#"
}

export function renderBBCode(raw: string): string {
  if (!raw) return ""

  // 1. 转义 HTML 杜绝 XSS
  let html = escapeHtml(raw)

  // 2. 标题语法 [h1] [h2] [h3]
  html = html.replace(
    /\[h1\]([\s\S]*?)\[\/h1\]/gi,
    '<h1 class="text-base font-bold text-c-primary mt-3 mb-1.5 pb-1 border-b border-c-subtle">$1</h1>',
  )
  html = html.replace(
    /\[h2\]([\s\S]*?)\[\/h2\]/gi,
    '<h2 class="text-sm font-bold text-c-primary mt-2.5 mb-1">$1</h2>',
  )
  html = html.replace(
    /\[h3\]([\s\S]*?)\[\/h3\]/gi,
    '<h3 class="text-xs font-semibold text-c-primary mt-2 mb-1">$1</h3>',
  )

  // 3. 基础文本样式 [b], [i], [u], [strike]
  html = html.replace(
    /\[b\]([\s\S]*?)\[\/b\]/gi,
    '<strong class="font-semibold text-c-primary">$1</strong>',
  )
  html = html.replace(/\[i\]([\s\S]*?)\[\/i\]/gi, '<em class="italic">$1</em>')
  html = html.replace(
    /\[u\]([\s\S]*?)\[\/u\]/gi,
    '<span class="underline underline-offset-2">$1</span>',
  )
  html = html.replace(
    /\[strike\]([\s\S]*?)\[\/strike\]/gi,
    '<del class="line-through text-c-muted">$1</del>',
  )

  // 4. 分割线 [hr] 或 [hr][/hr]
  html = html.replace(/\[hr\](?:\[\/hr\])?/gi, '<hr class="my-3 border-c-default" />')

  // 5. 链接 [url=...]...[/url] 与 [url]...[/url]
  html = html.replace(/\[url=([^\]]+)\]([\s\S]*?)\[\/url\]/gi, (_match, url, text) => {
    const safeUrl = sanitizeUrl(url)
    return `<a class="text-primary-theme hover:underline inline-flex items-center break-all" target="_blank" rel="noopener noreferrer" href="${safeUrl}">${text}</a>`
  })
  html = html.replace(/\[url\]([\s\S]*?)\[\/url\]/gi, (_match, url) => {
    const safeUrl = sanitizeUrl(url)
    return `<a class="text-primary-theme hover:underline inline-flex items-center break-all" target="_blank" rel="noopener noreferrer" href="${safeUrl}">${url}</a>`
  })

  // 6. 引用 [quote]...[/quote]
  html = html.replace(
    /\[quote\]([\s\S]*?)\[\/quote\]/gi,
    '<blockquote class="pl-3 my-2 border-l-2 border-primary-theme/50 text-c-muted italic">$1</blockquote>',
  )

  // 7. 代码块 [code]...[/code]
  html = html.replace(
    /\[code\]([\s\S]*?)\[\/code\]/gi,
    '<pre class="p-2.5 my-2 font-mono text-xs rounded-lg bg-c-secondary/60 text-c-primary overflow-x-auto"><code>$1</code></pre>',
  )

  // 8. 列表语法：无序列表 [list]...[/list] 与 有序列表 [olist]...[/olist]
  // 支持标准 [*] 与容错标记 []
  const parseListItems = (content: string) => {
    const items = content.split(/\[\*\]|\[\]/g)
    return items
      .map((item) => item.trim())
      .filter(Boolean)
      .map((item) => `<li>${item}</li>`)
      .join("")
  }

  html = html.replace(/\[olist\]([\s\S]*?)\[\/olist\]/gi, (_match, content: string) => {
    const lis = parseListItems(content)
    return `<ol class="list-decimal list-inside space-y-1 my-2 text-sm">${lis}</ol>`
  })

  html = html.replace(/\[list\]([\s\S]*?)\[\/list\]/gi, (_match, content: string) => {
    const lis = parseListItems(content)
    return `<ul class="list-disc list-inside space-y-1 my-2 text-sm">${lis}</ul>`
  })

  // 9. 图片 [img]...[/img] 呈现为优化缩略图 (限制最大高度和圆角)
  html = html.replace(/\[img\]([\s\S]*?)\[\/img\]/gi, (_match, src) => {
    const safeSrc = sanitizeUrl(src)
    return `<div class="my-2 max-w-full overflow-hidden rounded-lg border border-c-subtle inline-block"><img src="${safeSrc}" class="max-h-64 object-contain" loading="lazy" /></div>`
  })

  // 10. 换行回车符 -> <br>
  html = html.replace(/\r\n|\r|\n/g, "<br>")

  return html
}

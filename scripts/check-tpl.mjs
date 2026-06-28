import { readFileSync, writeFileSync } from "fs"
const content = readFileSync("src/pages/DiscoverPage.vue", "utf-8")
const match = content.match(/<template>([\s\S]*?)<\/template>/)
const html = match[1]
let depth = 0
const out = []
const lines = html.split("\n")
for (let i = 0; i < lines.length; i++) {
  const ln = lines[i]
  const openCount = (ln.match(/<div\b[^>]*>/g) || []).length
  const closeCount = (ln.match(/<\/div>/g) || []).length
  const selfCloseCount = (ln.match(/<div\b[^>]*\/\s*>/g) || []).length
  depth += openCount - closeCount - selfCloseCount
  if (openCount > 0 || closeCount > 0 || selfCloseCount > 0) {
    out.push(`L${399+i} d=${depth}: ${ln.trim().substring(0,100)}`)
  }
}
out.push(`\nFinal depth: ${depth}`)
out.push(`\nLines with depth < 0:`)
let d = 0
for (let i = 0; i < lines.length; i++) {
  const ln = lines[i]
  const openCount = (ln.match(/<div\b[^>]*>/g) || []).length
  const closeCount = (ln.match(/<\/div>/g) || []).length
  const selfCloseCount = (ln.match(/<div\b[^>]*\/\s*>/g) || []).length
  d += openCount - closeCount - selfCloseCount
  if (d < 0) out.push(`L${399+i} d=${d}: ${ln.trim().substring(0,100)}`)
}
writeFileSync("C:\\Users\\15432\\tpl-check.txt", out.join("\n"), "utf-8")
console.log("Written to tpl-check.txt, lines: " + out.length)

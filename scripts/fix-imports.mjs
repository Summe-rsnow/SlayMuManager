import { readFileSync, writeFileSync, readdirSync } from "fs"
import { join, dirname } from "path"

const src = process.cwd() + "/src"
const files = []

function walk(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory() && e.name !== "assets") walk(p)
    else if (e.isFile() && /\.(vue|ts)$/.test(e.name)) files.push(p)
  }
}
walk(src)

let n = 0
for (const f of files) {
  let c = readFileSync(f, "utf-8")
  const modDir = dirname(f).replace(src, "").replace(/\\/g, "/")
  const orig = c
  c = c.replace(
    /^(import\s+(?:\{[^}]*\}|[^"';\s]+)\s+from\s+)"(\.\.(?:\/[^"]+)*)"/gm,
    (_, prefix, rel) => {
      const parts = [...modDir.split("/").filter(Boolean), ...rel.split("/").filter(Boolean)]
      const out = []
      for (const p of parts) {
        if (p === "..") out.pop()
        else if (p !== ".") out.push(p)
      }
      return prefix + '"@/' + out.join("/") + '"'
    }
  )
  if (c !== orig) {
    writeFileSync(f, c, "utf-8")
    n++
    console.log("  " + f.replace(/\\/g, "/").replace(/^.*?src\//, "src/"))
  }
}
console.log("\nUpdated " + n + " files")

import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import { router } from "./router"
import { setupI18n } from "./i18n"
import { initTheme } from "./theme"
import "virtual:uno.css"
import "./assets/global.css"

// 全局禁用浏览器右键菜单
document.addEventListener("contextmenu", (e) => e.preventDefault())

// 初始化主题
initTheme()

const app = createApp(App)
const pinia = createPinia()
const i18n = setupI18n()

app.use(pinia)
app.use(router)
app.use(i18n)

app.mount("#app")

import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import { router } from "./router"
import { setupI18n } from "./i18n"
import "virtual:uno.css"
import "./assets/global.css"

// 全局禁用浏览器右键菜单
document.addEventListener("contextmenu", (e) => e.preventDefault())

const app = createApp(App)
const pinia = createPinia()
const i18n = setupI18n()

app.use(pinia)
app.use(router)
app.use(i18n)

app.mount("#app")

import { createRouter, createWebHashHistory } from "vue-router"

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "library",
      component: () => import("../pages/LibraryPage.vue"),
    },
    {
      path: "/discover",
      name: "discover",
      component: () => import("../pages/DiscoverPage.vue"),
    },
    {
      path: "/profiles",
      name: "profiles",
      component: () => import("../pages/ProfilesPage.vue"),
    },
    {
      path: "/saves",
      name: "saves",
      component: () => import("../pages/SavesPage.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("../pages/SettingsPage.vue"),
    },
  ],
})

export { router }

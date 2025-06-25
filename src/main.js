import { createApp } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import "bulma/css/bulma.min.css";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import Home from "./views/Home.vue";
import Settings from "./views/Settings.vue";

const routes = [
  { path: "/", component: Home },
  { path: "/settings", component: Settings },
];

const base = window.location.pathname.replace(/\/[^/]*$/, "/");
const router = createRouter({
  history: createWebHashHistory(base),
  routes,
});

createApp(App).use(router).mount("#app");

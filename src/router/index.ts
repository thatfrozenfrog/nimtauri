import { createRouter, createWebHashHistory } from "vue-router";
import BackendDemoView from "@/views/BackendDemoView.vue";
import HomeView from "@/views/HomeView.vue";

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/backend", name: "backend", component: BackendDemoView },
  ],
});

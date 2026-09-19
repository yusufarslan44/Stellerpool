import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: () => import('@/views/HomeView.vue') },
    { path: '/create', name: 'create', component: () => import('@/views/CreateView.vue') },
    { path: '/pool/:id(\\d+)', name: 'pool', component: () => import('@/views/PoolView.vue') },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
  scrollBehavior: () => ({ top: 0 }),
})

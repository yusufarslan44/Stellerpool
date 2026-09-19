import { createRouter, createWebHistory } from 'vue-router'
import { mainnetShowcase } from '@/lib/stellar'

export const router = createRouter({
  history: createWebHistory(),
  routes: mainnetShowcase
    ? [
        { path: '/', name: 'showcase', component: () => import('@/views/ShowcaseView.vue') },
        { path: '/:pathMatch(.*)*', redirect: '/' },
      ]
    : [
        { path: '/', name: 'home', component: () => import('@/views/HomeView.vue') },
        { path: '/create', name: 'create', component: () => import('@/views/CreateView.vue') },
        { path: '/pool/:id(\\d+)', name: 'pool', component: () => import('@/views/PoolView.vue') },
        { path: '/:pathMatch(.*)*', redirect: '/' },
      ],
  scrollBehavior: () => ({ top: 0 }),
})

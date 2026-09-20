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
        { path: '/join', name: 'join', component: () => import('@/views/JoinView.vue') },
        { path: '/create', redirect: (to) => ({ path: '/join', query: to.query }) },
        { path: '/pool/:id(\\d+)', name: 'pool', component: () => import('@/views/PoolView.vue') },
        { path: '/:pathMatch(.*)*', redirect: '/' },
      ],
  scrollBehavior: (to, _from, saved) => {
    if (saved) return saved
    if (to.hash) return { el: to.hash, top: 96, behavior: 'smooth' }
    return { top: 0 }
  },
})

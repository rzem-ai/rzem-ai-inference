import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/generate'
    },
    {
      path: '/generate',
      name: 'generate',
      component: () => import('@/views/GenerateView.vue')
    },
    {
      path: '/refine',
      name: 'refine',
      component: () => import('@/views/RefineView.vue')
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue')
    },
    {
      path: '/manage',
      name: 'manage',
      component: () => import('@/views/ManageView.vue')
    }
  ]
})

export default router

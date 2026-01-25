import { createRouter, createWebHistory } from 'vue-router'
import ModelsView from '@/views/ModelsView.vue'

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
      path: '/gallery',
      name: 'gallery',
      component: () => import('@/views/GalleryView.vue'),
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue')
    },
    {
      path: '/models',
      name: 'models',
      component: ModelsView,
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue')
    }
  ]
})

export default router

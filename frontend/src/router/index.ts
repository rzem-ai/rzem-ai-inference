import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      redirect: { name: 'create' },
    },
    {
      path: '/create',
      name: 'create',
      components: {
        default: () => import('@/pages/create/Main.vue'),
        menu: () => import('@/pages/create/Menu.vue'),
      }
    },
    {
      path: '/gallery',
      name: 'gallery',
      components: {
        default: () => import('@/pages/gallery/Main.vue'),
        menu: () => import('@/pages/gallery/Menu.vue'),
      }
    },
  ],
});

export default router;

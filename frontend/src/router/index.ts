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
      path: '/about',
      name: 'about',
      components: {
        default: () => import('@/pages/AboutPage.vue'),
        menu: () => import('@/pages/AboutPage.vue'),
        history: () => import('@/pages/AboutPage.vue'),
      }
    },
    {
      path: '/create',
      name: 'create',
      components: {
        default: () => import('@/pages/create/Main.vue'), 
        menu: () => import('@/pages/create/Menu.vue'),
      }
    },
  ],
});

export default router;

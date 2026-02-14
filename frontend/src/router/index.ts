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
    {
      path: '/styles',
      components: {
        default: () => import('@/pages/styles/Main.vue'),
        menu: () => import('@/pages/styles/Menu.vue'),
      },
      children: [
        {
          path: '',
          name: 'styles',
          component: () => import('@/pages/styles/StylesList.vue'),
        },
        {
          path: 'new',
          name: 'styles-new',
          component: () => import('@/pages/styles/StyleEditor.vue'),
        },
        {
          path: ':id',
          name: 'styles-edit',
          component: () => import('@/pages/styles/StyleEditor.vue'),
        },
      ],
    },
    {
      path: '/settings',
      components: {
        default: () => import('@/pages/settings/Main.vue'),
        menu: () => import('@/pages/settings/Menu.vue'),
      },
      children: [
        {
          path: '',
          name: 'settings',
          redirect: { name: 'settings-engine' },
        },
        {
          path: 'engine',
          name: 'settings-engine',
          component: () => import('@/pages/settings/InferenceEngine.vue'),
        },
        {
          path: 'servers',
          name: 'settings-servers',
          component: () => import('@/pages/settings/RemoteServers.vue'),
        },
        {
          path: 'api-keys',
          name: 'settings-api-keys',
          component: () => import('@/pages/settings/ApiKeys.vue'),
        },
        {
          path: 'cache',
          name: 'settings-cache',
          component: () => import('@/pages/settings/ModelCache.vue'),
        },
      ],
    },
  ],
});

export default router;

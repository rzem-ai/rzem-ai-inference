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
      path: '/edit',
      name: 'edit',
      components: {
        default: () => import('@/pages/edit/Main.vue'),
        menu: () => import('@/pages/edit/Menu.vue'),
      },
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
      path: '/models',
      name: 'models',
      components: {
        default: () => import('@/pages/models/Main.vue'),
        menu: () => import('@/pages/models/Menu.vue'),
      },
    },
    {
      path: '/workflow',
      name: 'workflow',
      components: {
        default: () => import('@/pages/workflow/Main.vue'),
        menu: () => import('@/pages/workflow/Menu.vue'),
      },
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
          path: 'builder',
          name: 'styles-builder',
          component: () => import('@/pages/styles/StyleBuilder.vue'),
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
        {
          path: 'generation',
          name: 'settings-generation',
          component: () => import('@/pages/settings/Generation.vue'),
        },
        {
          path: 'ai',
          name: 'settings-ai',
          component: () => import('@/pages/settings/AI.vue'),
        },
        {
          path: 'network',
          name: 'settings-network',
          component: () => import('@/pages/settings/Network.vue'),
        },
      ],
    },
  ],
});

export default router;

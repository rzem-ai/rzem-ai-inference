import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/generate',
    },
    {
      path: '/generate',
      name: 'generate',
      component: () => import('@/views/GenerateView.vue'),
    },
    {
      path: '/gallery',
      name: 'gallery',
      component: () => import('@/views/GalleryView.vue'),
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue'),
    },
    {
      path: '/styles',
      name: 'styles',
      component: () => import('@/views/StylesView.vue'),
    },
    {
      path: '/models',
      name: 'models',
      component: () => import('@/views/ModelsView.vue'),
      redirect: '/models/bundles',
      children: [
        {
          path: 'bundles',
          name: 'models-bundles',
          components: {
            default: () => import('@/components/models/BundlesView.vue'),
            sidebar: () => import('@/components/models/BundlesSidebar.vue'),
          },
        },
        {
          path: 'models',
          name: 'models-models',
          components: {
            default: () => import('@/components/models/ModelsView.vue'),
            sidebar: () => import('@/components/models/ModelsSidebar.vue'),
          },
        },
      ],
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      redirect: '/settings/apikeys',
      children: [
        {
          path: 'apikeys',
          name: 'apikeys',
          components: {
            default: () => import('@/components/settings/APIKeys.vue'),
          },
        },
        {
          path: 'cache',
          name: 'cache',
          components: {
            default: () => import('@/components/settings/Cache.vue'),
          },
        },
      ],
    },
  ],
});

export default router;

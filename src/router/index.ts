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
      path: '/models',
      component: () => import('@/views/ModelsView.vue'),
      redirect: '/models/bundles',
      children: [
        {
          path: 'bundles',
          name: 'models-bundles',
          components: {
            default: () => import('@/views/models/BundlesView.vue'),
            sidebar: () => import('@/views/models/BundlesSidebar.vue'),
          },
        },
        {
          path: 'models',
          name: 'models-models',
          components: {
            default: () => import('@/views/models/ModelsView.vue'),
            sidebar: () => import('@/views/models/ModelsSidebar.vue'),
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
            default: () => import('@/views/settings/APIKeys.vue'),
          },
        },
        {
          path: 'cache',
          name: 'cache',
          components: {
            default: () => import('@/views/settings/Cache.vue'),
          },
        },
      ],
    },
  ],
});

export default router;

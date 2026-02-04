import { createRouter, createWebHistory } from 'vue-router';

import GenerateView from '@/views/GenerateView.vue';
import GalleryView from '@/views/GalleryView.vue';
import ModelsView from '@/views/ModelsView.vue';
import BundlesView from '@/components/models/BundlesView.vue';
import BundlesSidebar from '@/components/models/BundlesSidebar.vue';
import SettingsView from '@/views/SettingsView.vue';
import SettingsAPIKeys from '@/components/settings/APIKeys.vue';
import SettingsCache from '@/components/settings/Cache.vue';
import StylesView from '@/views/StylesView.vue';

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
      component: GenerateView,
    },
    {
      path: '/gallery',
      name: 'gallery',
      component: GalleryView,
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue'),
    },
    {
      path: '/styles',
      name: 'styles',
      component: StylesView,
      children: [
        {
          path: '',
          name: 'styles-home',
          component: () => import('@/components/styles/StylesHome.vue'),
        },
        {
          path: 'create',
          name: 'styles-create',
          component: () => import('@/components/styles/StyleCreate.vue'),
        },
        {
          path: 'style/:id',
          name: 'styles-style',
          component: () => import('@/components/styles/StyleEditor.vue'),
        },
      ],
    },
    {
      path: '/models',
      name: 'models',
      component: ModelsView,
      redirect: '/models/bundles',
      children: [
        {
          path: 'bundles',
          name: 'models-bundles',
          components: {
            default: BundlesView,
            sidebar: BundlesSidebar,
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
      component: SettingsView,
      redirect: '/settings/apikeys',
      children: [
        {
          path: 'apikeys',
          name: 'apikeys',
          components: {
            default: SettingsAPIKeys,
          },
        },
        {
          path: 'cache',
          name: 'cache',
          components: {
            default: SettingsCache,
          },
        },
      ],
    },
  ],
});

export default router;

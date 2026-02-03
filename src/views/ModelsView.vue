<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Model Manager</template>
      <template #body>
        <BundlesSidebar />
      </template>
    </WorkspaceActions>

    <!-- Right Panel: Router View -->
    <div class="w-full h-full p-2">
      <RouterView />
    </div>
  </div>

  <Toast />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';

import Toast from 'primevue/toast';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
import BundlesSidebar from '@/components/models/BundlesSidebar.vue';
import { onMounted } from 'vue';
import { useBundlesStore } from '@/stores/bundles';
const bundlesStore = useBundlesStore();
onMounted(() => {
  bundlesStore.initialize();
});
</script>

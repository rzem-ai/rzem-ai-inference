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
    <!-- Main Content -->
    <main class="flex flex-1 h-full p-2 transition-all duration-300 grow">
      <div class="flex flex-col flex-1 h-full gap-2 p-4 overflow-hidden border rounded-lg border-surface-700 bg-surface-950">
        <RouterView />
      </div>
    </main>
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

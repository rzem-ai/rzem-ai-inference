<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Model Manager</template>

      <template #toolbar>
        <!-- Navigation Tabs -->
        <div class="w-full grid grid-cols-2 border rounded-md shadow-xs border-surface-600">
          <ToggleButton :model-value="showBundles" severity="secondary" size="small" class="border-0 rounded-none rounded-l-md" @change="handleToggleBundles()">
            <div class="content-center">
              <fa :icon="['fal', 'box-taped']" size="sm" />
              Bundles
            </div>
          </ToggleButton>
          <ToggleButton :model-value="showModels" severity="secondary" size="small" class="border-0 rounded-none rounded-r-md" @change="handleToggleModels()">
            <div class="content-center">
              <fa :icon="['fal', 'layer-group']" size="sm" />
              Models
            </div>
          </ToggleButton>
        </div>
      </template>

      <template #body>
        <RouterView name="sidebar" />
      </template>
    </WorkspaceActions>

    <!-- Right Panel: Router View -->
    <div class="w-full h-full p-2">
      <RouterView />
    </div>
  </div>

  <!-- Confirmation Dialog -->
  <ConfirmDialog />
  <Toast />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter, RouterView } from 'vue-router';

import ToggleButton from 'primevue/togglebutton';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';

const route = useRoute();
const router = useRouter();

const showBundles = computed(() => route.name == 'models-bundles');

const showModels = computed(() => route.name == 'models-models');

const handleToggleBundles = () => {
  router.push('/models/bundles');
};

const handleToggleModels = () => {
  router.push('/models/Models');
};
</script>

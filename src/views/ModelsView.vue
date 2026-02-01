<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Model Manager</template>

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
import { RouterView } from 'vue-router';

import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
</script>

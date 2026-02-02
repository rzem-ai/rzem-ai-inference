<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Model Manager</template>
      <template #toolbar>
        <div class="grid w-full grid-cols-2 border rounded-md shadow-xs border-surface-600">
          <router-link to="/models/bundles" class="border-0 rounded-none rounded-l-md p-togglebutton p-component p-togglebutton-sm p-inputfield-sm" active-class="p-togglebutton-checked ">
            <div class="content-center p-togglebutton-content">
              <fa :icon="['fal', 'box-taped']" />
              Bundles
            </div>
          </router-link>
          <router-link to="/models/models" class="border-0 rounded-none rounded-r-md p-togglebutton p-component p-togglebutton-sm p-inputfield-sm" active-class="p-togglebutton-checked ">
            <div class="content-center p-togglebutton-content">
              <fa :icon="['fal', 'microchip-ai']" />
              Models
            </div>
          </router-link>
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

  <Toast />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';

import Toast from 'primevue/toast';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
</script>

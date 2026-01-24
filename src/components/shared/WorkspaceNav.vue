<template>
  <div class="flex flex-col w-20 h-full bg-surface-900 min-w-20">
    <div class="flex items-center justify-center h-16 p-2 min-h-16 shrink">
      <div class="flex items-center justify-center w-full h-full border rounded border-surface-600"> rz.ai </div>
    </div>
    <div class="flex flex-col items-center justify-start py-4 grow">
      <WorkspaceNavItem v-for="(workspace, index) in workspaces" :key="workspace.path" :id="'nav_' + index" :workspace="workspace" />
    </div>
    <div class="flex items-center justify-center h-16 min-h-16 shrink">
      
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import WorkspaceNavItem, { type Workspace } from './WorkspaceNavItem.vue';

const workspaces: Workspace[] = [
  { label: 'Generate', path: '/generate', icon: 'sparkles' },
  { label: 'Gallery', path: '/gallery', icon: 'images' },
  { label: 'Compare', path: '/compare', icon: 'code-compare' },
  { label: 'Models', path: '/models', icon: 'database' },
  { label: 'Settings', path: '/manage', icon: 'gear' }
];

onMounted(() => {
  //STUB
});
</script>

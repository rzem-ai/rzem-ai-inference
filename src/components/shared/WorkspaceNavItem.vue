<template>
  <RouterLink :id="id" :to="workspace.path" v-slot="{ isActive }" class="w-full">
    <div
      class="flex flex-col items-center justify-center gap-1 px-2 py-4"
      :class="[
        isActive
          ? 'border-l-4 bg-surface-800 border-blue-500 text-blue-500 hover:text-surface-50'
          : 'border-l-4 bg-surface-900 border-surface-800 text-surface-400 hover:text-surface-50 hover:border-blue-500/50',
      ]">
      <fa :icon="['fal', workspace.icon]" size="xl"/>
      <div class="text-xs" >
        {{ workspace.label }}
      </div>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
export interface Workspace {
  label: string;
  path: string;
  icon: string;
}

defineProps<{
  workspace: Workspace;
  id?: string;
}>();
</script>

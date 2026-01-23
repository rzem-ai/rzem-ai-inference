<template>
  <RouterLink :id="id" :to="workspace.path" v-slot="{ isActive }" class="w-full">
    <div
      class="flex flex-col items-center justify-center gap-1 px-2 py-4"
      :class="[
        isActive
          ? 'border-l-4 bg-gray-800 border-blue-500 text-blue-200 hover:text-gray-50'
          : 'border-l-4 bg-gray-900 border-gray-800 text-gray-400 hover:text-gray-50 hover:border-blue-500/50',
      ]">
      <fa :icon="['fal', workspace.icon]" size="xl"/>
      <div class="text-xs">
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

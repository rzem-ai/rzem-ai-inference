<template>
  <div class="h-full p-2">
    <div class="h-full bg-white rounded-xl shadow-sm flex flex-col overflow-hidden border border-slate-200">
      <!-- -->
      <div class="w-120 flex flex-col gap-1 p-3">
        <div class="px-2 mb-2">
          <span class="text-lg font-semibold text-slate-900">Settings</span>
        </div>

        <RouterLink
          v-for="item in navItems"
          :key="item.route"
          :to="{ name: item.route }"
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="route.name === item.route ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'">
          <component :is="item.icon" :size="14" />
          <span class="font-medium">{{ item.label }}</span>
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { Cpu, KeyRound, HardDrive } from 'lucide-vue-next';

const route = useRoute();

const navItems = [
  { label: 'Inference Engine', icon: Cpu, route: 'settings-engine' },
  { label: 'API Keys', icon: KeyRound, route: 'settings-api-keys' },
  { label: 'Model Cache', icon: HardDrive, route: 'settings-cache' },
];
</script>

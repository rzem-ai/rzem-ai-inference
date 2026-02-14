<template>
  <MenuPanel title="Settings" :icon="Cog">
    <template #title-button>
      <Button class="transition-colors" severity="secondary" variant="outlined" :disabled="true">
        <Plus :size="16" />
      </Button>
    </template>
    <template #content>
      <RouterLink
        v-for="item in navItems"
        :key="item.route"
        :to="{ name: item.route }"
        class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
        :class="route.name === item.route ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'">
        <component :is="item.icon" :size="14" />
        <span class="font-medium">{{ item.label }}</span>
      </RouterLink>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { Cog, Cpu, KeyRound, HardDrive, Wifi, Plus } from 'lucide-vue-next';
import { Button } from 'primevue';
import MenuPanel from '@/components/MenuPanel.vue';

const route = useRoute();

const navItems = [
  { label: 'Inference Engine', icon: Cpu, route: 'settings-engine' },
  { label: 'Remote Servers', icon: Wifi, route: 'settings-servers' },
  { label: 'API Keys', icon: KeyRound, route: 'settings-api-keys' },
  { label: 'Model Cache', icon: HardDrive, route: 'settings-cache' },
];
</script>

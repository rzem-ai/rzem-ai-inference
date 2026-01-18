<template>
  <header>
    <nav class="workspace-nav">
      <div class="nav-items">
        <RouterLink
          v-for="workspace in workspaces"
          :key="workspace.path"
          :to="workspace.path"
          class="nav-item"
          active-class="nav-item-active"
          exact-active-class="nav-item-exact">
          <component :is="workspace.icon" :size="15" />
          <span>{{ workspace.label }}</span>
        </RouterLink>
      </div>
    </nav>
  </header>
</template>

<script setup lang="ts">
import { Sparkles, Layers, Images, Settings, Download } from 'lucide-vue-next';

const workspaces = [
  { label: 'Generate', path: '/generate', icon: Sparkles },
  { label: 'Gallery', path: '/gallery', icon: Sparkles },
  { label: 'Refine', path: '/refine', icon: Layers },
  { label: 'Compare', path: '/compare', icon: Images },
  { label: 'Models', path: '/models', icon: Download },
  { label: 'Settings', path: '/manage', icon: Settings },
];
</script>

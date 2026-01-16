<template>
  <nav class="workspace-nav">
    <div class="nav-items">
      <RouterLink
        v-for="workspace in workspaces"
        :key="workspace.path"
        :to="workspace.path"
        class="nav-item"
        :class="{ active: isActive(workspace.path) }"
      >
        <component :is="workspace.icon" :size="20" />
        <span>{{ workspace.label }}</span>
      </RouterLink>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { Sparkles, Layers, Images, Settings, Download } from 'lucide-vue-next'

const route = useRoute()

const workspaces = [
  { label: 'Generate', path: '/generate', icon: Sparkles },
  { label: 'Refine', path: '/refine', icon: Layers },
  { label: 'Compare', path: '/compare', icon: Images },
  { label: 'Models', path: '/models', icon: Download },
  { label: 'Manage', path: '/manage', icon: Settings }
]

const isActive = (path: string) => {
  return route.path === path
}
</script>

<style scoped>
.workspace-nav {
  display: flex;
  align-items: center;
  padding: 0.5rem 1rem;
  background: #f8f9fa;
  border-bottom: 1px solid #e9ecef;
}

.nav-items {
  display: flex;
  gap: 0.5rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  text-decoration: none;
  color: #495057;
  font-weight: 500;
  transition: all 0.2s;
}

.nav-item:hover {
  background: #e9ecef;
  color: #212529;
}

.nav-item.active {
  background: #007bff;
  color: white;
}
</style>

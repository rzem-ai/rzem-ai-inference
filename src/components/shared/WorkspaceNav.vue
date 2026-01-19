<template>
  <aside class="workspace-sidebar">
    <nav class="sidebar-nav">
      <!-- Logo / Brand -->
      <div class="sidebar-brand">
        <div class="brand-icon">
          <Sparkles :size="24" />
        </div>
        <span class="brand-text">rzem.ai</span>
      </div>

      <!-- Navigation Items -->
      <div class="nav-section">
        <span class="section-label">APPS</span>
        <div class="nav-items">
          <RouterLink
            v-for="workspace in workspaces"
            :key="workspace.path"
            :to="workspace.path"
            class="nav-item"
            active-class="nav-item-active">
            <div class="nav-icon">
              <component :is="workspace.icon" :size="20" />
            </div>
            <span class="nav-label">{{ workspace.label }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Bottom section (could add user profile, dark mode, etc.) -->
      <div class="sidebar-bottom">
        <button class="nav-item" @click="toggleDarkMode">
          <div class="nav-icon">
            <Settings :size="20" />
          </div>
          <span class="nav-label">Settings</span>
        </button>
      </div>
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { Sparkles, Layers, Images, Settings, Download, ImagePlus } from 'lucide-vue-next';

const workspaces = [
  { label: 'Flux.1 Builder', path: '/generate', icon: Sparkles },
  { label: 'Flux.2 Builder', path: '/gallery', icon: ImagePlus },
  { label: 'Avatar Creator', path: '/refine', icon: Layers },
  { label: 'X/Y Plot', path: '/compare', icon: Images },
  { label: 'Storyboarder AI', path: '/models', icon: Download },
];

const toggleDarkMode = () => {
  // Placeholder for dark mode toggle
  console.log('Toggle dark mode');
};
</script>

<style scoped>
@reference "tailwindcss";

.workspace-sidebar {
  @apply fixed left-0 top-0 h-full z-50 transition-all duration-300 ease-in-out;
  width: 60px;
  background: #1a1d26;
  border-right: 1px solid #2d3748;
}

.workspace-sidebar:hover {
  width: 220px;
  box-shadow: 4px 0 12px rgba(0, 0, 0, 0.3);
}

.sidebar-nav {
  @apply flex flex-col h-full;
}

/* Brand Section */
.sidebar-brand {
  @apply flex items-center gap-3 px-3 py-4 border-b;
  border-color: #2d3748;
  min-height: 60px;
}

.brand-icon {
  @apply flex items-center justify-center flex-shrink-0;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.brand-text {
  @apply text-sm font-bold whitespace-nowrap opacity-0 transition-opacity duration-200;
  color: #f1f5f9;
}

.workspace-sidebar:hover .brand-text {
  opacity: 1;
  transition-delay: 50ms;
}

/* Navigation Section */
.nav-section {
  @apply flex-1 py-4;
}

.section-label {
  @apply block px-4 mb-2 text-xs font-semibold tracking-wider opacity-0 transition-opacity duration-200;
  color: #64748b;
}

.workspace-sidebar:hover .section-label {
  opacity: 1;
  transition-delay: 50ms;
}

.nav-items {
  @apply flex flex-col gap-1 px-2;
}

.nav-item {
  @apply flex items-center gap-3 px-2 py-2.5 rounded-lg cursor-pointer transition-all duration-200 no-underline;
  color: #94a3b8;
  border: none;
  background: transparent;
  width: 100%;
  text-align: left;
}

.nav-item:hover {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.nav-item-active {
  background: #3b82f6 !important;
  color: white !important;
}

.nav-icon {
  @apply flex items-center justify-center flex-shrink-0;
  width: 28px;
  height: 28px;
}

.nav-label {
  @apply text-sm whitespace-nowrap opacity-0 transition-opacity duration-200;
  font-weight: 500;
}

.workspace-sidebar:hover .nav-label {
  opacity: 1;
  transition-delay: 50ms;
}

/* Bottom Section */
.sidebar-bottom {
  @apply border-t px-2 py-2;
  border-color: #2d3748;
}
</style>

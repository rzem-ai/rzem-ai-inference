<template>
  <div class="fixed top-0 left-0 flex flex-col h-full transition-all ease-in-out bg-gray-800 border-r border-gray-600 w-18 group z-99 duration-400 hover:w-48">
    <!-- Logo / Brand -->
    <div class="flex items-center w-20 h-20 gap-2 p-6">
      <div class="flex items-center justify-center w-10 h-10 text-gray-100 rounded-xl">
        <Sparkles :size="20" />
      </div>
      <span class="text-lg font-semibold transition-opacity duration-300 opacity-0 whitespace-nowrap group-hover:opacity-100">rzem.ai</span>
    </div>

    <div class="flex flex-col justify-between flex-1">
      <!-- Navigation Items -->
      <div class="overflow-x-hidden overflow-y-auto px-2 py-6 [&::-webkit-scrollbar]:hidden">
        <span class="block px-2 mb-3 text-xs font-semibold tracking-wider text-gray-400 transition-opacity duration-300 opacity-0 group-hover:opacity-100"
          >APPS</span
        >
        <ul class="p-0 m-0 list-none">
          <li v-for="(workspace, index) in workspaces" :key="workspace.path" :id="'nav_' + index" class="relative mx-0 my-2">
            <RouterLink
              :to="workspace.path"
              class="flex items-center w-full h-12 transition-all duration-200 rounded-xl hover:bg-blue-600"
              active-class="bg-blue-500 hover:bg-blue-600"
              exact-active-class="bg-blue-500 hover:bg-blue-600">
              <span class="flex items-center justify-center h-12 text-gray-100 transition-colors duration-300 nav-icon min-w-12">
                <component :is="workspace.icon" :size="20" />
              </span>
              <span
                class="text-sm font-medium text-gray-200 transition-opacity duration-300 opacity-0 pointer-events-none links_name whitespace-nowrap group-hover:opacity-100 group-hover:pointer-events-auto">
                {{ workspace.label }}
              </span>
            </RouterLink>
          </li>
        </ul>
      </div>

      <!-- Bottom section -->
      <div class="p-2">
        <RouterLink
          :to="settings.path"
          class="flex items-center w-full h-12 transition-all duration-200 rounded-xl hover:bg-blue-600"
          active-class="bg-blue-500 hover:bg-blue-600"
          exact-active-class="bg-blue-500 hover:bg-blue-600">
          <span class="flex items-center justify-center h-12 text-gray-100 transition-colors duration-300 nav-icon min-w-12">
            <component :is="settings.icon" :size="20" />
          </span>
          <span
            class="text-sm font-medium text-gray-200 transition-opacity duration-300 opacity-0 pointer-events-none links_name whitespace-nowrap group-hover:opacity-100 group-hover:pointer-events-auto">
            {{ settings.label }}
          </span>
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { Sparkles, Layers, Images, Settings, Download, GalleryVerticalEnd } from 'lucide-vue-next';

const workspaces = [
  { label: 'Generate', path: '/generate', icon: Sparkles },
  { label: 'Gallery', path: '/gallery', icon: GalleryVerticalEnd },
  { label: 'Refine', path: '/refine', icon: Layers },
  { label: 'Compare', path: '/compare', icon: Images },
  { label: 'Models', path: '/models', icon: Download },
];

const settings = { label: 'Settings', path: '/manage', icon: Settings };

onMounted(() => {});
</script>

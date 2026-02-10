<script setup lang="ts">
import Card from "primevue/card";

const techStack = [
  { name: "pywebview", version: "5.x", description: "Lightweight cross-platform webview wrapper" },
  { name: "Vue 3", version: "3.5+", description: "Progressive JavaScript framework" },
  { name: "Tailwind CSS", version: "4.x", description: "Utility-first CSS framework (CSS-first config)" },
  { name: "PrimeVue", version: "4.x", description: "Rich UI component library (Aura theme)" },
  { name: "Pinia", version: "3.x", description: "Intuitive Vue.js store" },
  { name: "Vue Router", version: "4.x", description: "Official client-side router (hash mode)" },
  { name: "Vite", version: "6.x", description: "Next-generation frontend build tool" },
  { name: "TypeScript", version: "5.x", description: "Typed JavaScript superset" },
];
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-2xl font-bold">About This Template</h1>

    <Card>
      <template #title>Architecture</template>
      <template #content>
        <p class="mb-4 text-surface-600">
          This template combines a Python backend with a Vue 3 frontend, connected
          via pywebview's JavaScript API bridge. In development, the frontend is served
          by Vite's dev server with HMR. In production, built assets are loaded directly
          from disk.
        </p>
      </template>
    </Card>

    <Card>
      <template #title>Tech Stack</template>
      <template #content>
        <div class="flex flex-col gap-3">
          <div
            v-for="tech in techStack"
            :key="tech.name"
            class="flex items-start gap-4 border-b border-surface-100 pb-3 last:border-0"
          >
            <div class="min-w-32">
              <span class="font-semibold">{{ tech.name }}</span>
              <span class="ml-2 text-xs text-surface-400">{{ tech.version }}</span>
            </div>
            <span class="text-sm text-surface-600">{{ tech.description }}</span>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>

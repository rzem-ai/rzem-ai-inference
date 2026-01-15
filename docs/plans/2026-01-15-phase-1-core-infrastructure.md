# Phase 1: Core Infrastructure - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the Tauri + Vue3 project with basic Candle integration, modular Rust backend, SQLite database, and workspace navigation UI.

**Architecture:** Tauri 2 backend with modular Rust structure (inference, models, queue, gallery modules) + Vue 3 frontend with PrimeVue components and Pinia stores. SQLite for metadata storage.

**Tech Stack:** Tauri 2, Vue 3, TypeScript, Rust, Candle, SQLite, PrimeVue, TailwindCSS, Pinia

---

## Task 1: Initialize Tauri + Vue Project

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `tailwind.config.js`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`

**Step 1: Create package.json with dependencies**

```bash
cat > package.json << 'EOF'
{
  "name": "flux-generator",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev"
  },
  "dependencies": {
    "@primeuix/themes": "^2.0.2",
    "@tailwindcss/vite": "^4.1.18",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2.6.0",
    "@tauri-apps/plugin-fs": "^2.4.5",
    "@tauri-apps/plugin-opener": "^2",
    "@vueuse/components": "^14.1.0",
    "@vueuse/core": "^14.1.0",
    "lodash": "^4.17.21",
    "lucide-vue-next": "^0.562.0",
    "pinia": "^3.0.4",
    "primevue": "^4.5.4",
    "vue": "^3.5.13",
    "vue-router": "^4.6.4"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.1.18",
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-vue": "^5.2.1",
    "autoprefixer": "^10.4.23",
    "postcss": "^8.5.6",
    "tailwindcss": "^4.1.18",
    "typescript": "~5.6.2",
    "vite": "^6.0.3",
    "vue-tsc": "^2.1.10"
  }
}
EOF
```

**Step 2: Install npm dependencies**

Run: `npm install`
Expected: Dependencies installed successfully

**Step 3: Create TypeScript config**

```bash
cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,

    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",

    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
EOF
```

**Step 4: Create vite config**

```bash
cat > vite.config.ts << 'EOF'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
})
EOF
```

**Step 5: Create minimal Tailwind config**

```bash
cat > tailwind.config.js << 'EOF'
export default {}
EOF
```

**Step 6: Create index.html**

```bash
cat > index.html << 'EOF'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Flux Generator</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
EOF
```

**Step 7: Initialize Tauri project structure**

```bash
mkdir -p src-tauri/src
```

**Step 8: Create Cargo.toml**

```bash
cat > src-tauri/Cargo.toml << 'EOF'
[package]
name = "flux-generator"
version = "0.1.0"
description = "AI Image Generation with Flux"
authors = ["you"]
edition = "2021"

[lib]
name = "flux_generator_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
rusqlite = { version = "0.32", features = ["bundled"] }

# Candle dependencies (CPU only for now, GPU features added later)
candle-core = "0.8"
candle-nn = "0.8"
image = "0.25"
EOF
```

**Step 9: Create basic tauri.conf.json**

```bash
cat > src-tauri/tauri.conf.json << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Flux Generator",
  "version": "0.1.0",
  "identifier": "com.flux.generator",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "app": {
    "windows": [
      {
        "title": "Flux Generator",
        "width": 1400,
        "height": 900,
        "minWidth": 1200,
        "minHeight": 700
      }
    ],
    "security": {
      "csp": null
    }
  }
}
EOF
```

**Step 10: Create build.rs**

```bash
cat > src-tauri/build.rs << 'EOF'
fn main() {
    tauri_build::build()
}
EOF
```

**Step 11: Commit project initialization**

```bash
git add -A
git commit -m "feat: initialize Tauri + Vue project

- Add package.json with dependencies
- Configure TypeScript and Vite
- Set up Tauri configuration
- Add Candle dependencies to Cargo.toml

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Create Rust Module Structure

**Files:**
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/inference/mod.rs`
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/queue/mod.rs`
- Create: `src-tauri/src/gallery/mod.rs`
- Create: `src-tauri/src/utils/mod.rs`

**Step 1: Create module directories**

```bash
mkdir -p src-tauri/src/{inference,models,queue,gallery,utils}
```

**Step 2: Create lib.rs with module declarations**

```bash
cat > src-tauri/src/lib.rs << 'EOF'
mod inference;
mod models;
mod queue;
mod gallery;
mod utils;

use tauri::command;

#[command]
fn health_check() -> String {
    "OK".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

**Step 3: Create inference module stub**

```bash
cat > src-tauri/src/inference/mod.rs << 'EOF'
//! Inference engine for running Flux models with Candle

pub struct InferenceEngine;

impl InferenceEngine {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_engine_creation() {
        let _engine = InferenceEngine::new();
    }
}
EOF
```

**Step 4: Create models module stub**

```bash
cat > src-tauri/src/models/mod.rs << 'EOF'
//! Model management, loading, and caching

pub struct ModelManager;

impl ModelManager {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_manager_creation() {
        let _manager = ModelManager::new();
    }
}
EOF
```

**Step 5: Create queue module stub**

```bash
cat > src-tauri/src/queue/mod.rs << 'EOF'
//! Job queue management and batching

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub prompt: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

pub struct QueueManager {
    jobs: Vec<GenerationJob>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add_job(&mut self, prompt: String) -> String {
        let id = Uuid::new_v4().to_string();
        let job = GenerationJob {
            id: id.clone(),
            prompt,
            status: JobStatus::Queued,
        };
        self.jobs.push(job);
        id
    }

    pub fn get_jobs(&self) -> &[GenerationJob] {
        &self.jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_manager_add_job() {
        let mut manager = QueueManager::new();
        let job_id = manager.add_job("test prompt".to_string());
        assert!(!job_id.is_empty());
        assert_eq!(manager.get_jobs().len(), 1);
    }
}
EOF
```

**Step 6: Create gallery module stub**

```bash
cat > src-tauri/src/gallery/mod.rs << 'EOF'
//! Gallery database and metadata management

use rusqlite::Connection;
use anyhow::Result;

pub struct GalleryDb {
    conn: Connection,
}

impl GalleryDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                prompt TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_db_creation() {
        let db = GalleryDb::new(":memory:").unwrap();
        db.init_schema().unwrap();
    }
}
EOF
```

**Step 7: Create utils module stub**

```bash
cat > src-tauri/src/utils/mod.rs << 'EOF'
//! Utility functions for image processing and system monitoring

use std::path::Path;

pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = env::temp_dir().join("flux_test");
        ensure_dir_exists(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        std::fs::remove_dir(&temp_dir).unwrap();
    }
}
EOF
```

**Step 8: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

**Step 9: Commit Rust module structure**

```bash
git add src-tauri/
git commit -m "feat: create modular Rust backend structure

- Add inference, models, queue, gallery, utils modules
- Implement basic stubs with unit tests
- Add health_check command

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Set Up Vue Frontend Structure

**Files:**
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/style.css`
- Create: `src/vite-env.d.ts`
- Create: `src/router/index.ts`
- Create: `src/types/index.ts`

**Step 1: Create vite-env.d.ts**

```bash
mkdir -p src
cat > src/vite-env.d.ts << 'EOF'
/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
EOF
```

**Step 2: Create types file**

```bash
mkdir -p src/types
cat > src/types/index.ts << 'EOF'
export interface GenerationJob {
  id: string
  prompt: string
  status: 'Queued' | 'Running' | 'Completed' | 'Failed'
}

export interface GenerationParams {
  prompt: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  width: number
  height: number
  seed: number
  model: string
}
EOF
```

**Step 3: Create router**

```bash
mkdir -p src/router
cat > src/router/index.ts << 'EOF'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/generate'
    },
    {
      path: '/generate',
      name: 'generate',
      component: () => import('@/views/GenerateView.vue')
    },
    {
      path: '/refine',
      name: 'refine',
      component: () => import('@/views/RefineView.vue')
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue')
    },
    {
      path: '/manage',
      name: 'manage',
      component: () => import('@/views/ManageView.vue')
    }
  ]
})

export default router
EOF
```

**Step 4: Create global styles**

```bash
cat > src/style.css << 'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  margin: 0;
  padding: 0;
  font-family: system-ui, -apple-system, sans-serif;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}
EOF
```

**Step 5: Create main.ts**

```bash
cat > src/main.ts << 'EOF'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import Aura from '@primeuix/themes/aura'
import router from './router'
import App from './App.vue'
import './style.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: '.dark-mode'
    }
  }
})

app.mount('#app')
EOF
```

**Step 6: Create App.vue**

```bash
cat > src/App.vue << 'EOF'
<template>
  <div class="app-container">
    <WorkspaceNav />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import WorkspaceNav from '@/components/shared/WorkspaceNav.vue'
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
EOF
```

**Step 7: Commit Vue structure**

```bash
git add src/
git commit -m "feat: set up Vue frontend structure

- Add main.ts with PrimeVue and Pinia
- Create router with workspace routes
- Add TypeScript types
- Configure global styles

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Workspace Navigation Component

**Files:**
- Create: `src/components/shared/WorkspaceNav.vue`

**Step 1: Create shared components directory**

```bash
mkdir -p src/components/shared
```

**Step 2: Create WorkspaceNav component**

```bash
cat > src/components/shared/WorkspaceNav.vue << 'EOF'
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
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { Sparkles, Layers, Images, Settings } from 'lucide-vue-next'

const route = useRoute()

const workspaces = [
  { label: 'Generate', path: '/generate', icon: Sparkles },
  { label: 'Refine', path: '/refine', icon: Layers },
  { label: 'Compare', path: '/compare', icon: Images },
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
EOF
```

**Step 3: Test navigation renders**

Run: `npm run dev`
Expected: Dev server starts, navigation bar visible

**Step 4: Commit workspace navigation**

```bash
git add src/components/
git commit -m "feat: add workspace navigation component

- Create WorkspaceNav with 4 workspaces
- Use lucide-vue-next icons
- Style active/hover states

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Create Workspace View Stubs

**Files:**
- Create: `src/views/GenerateView.vue`
- Create: `src/views/RefineView.vue`
- Create: `src/views/CompareView.vue`
- Create: `src/views/ManageView.vue`

**Step 1: Create views directory**

```bash
mkdir -p src/views
```

**Step 2: Create GenerateView stub**

```bash
cat > src/views/GenerateView.vue << 'EOF'
<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Generate</h1>
      <p>Text-to-image, Image-to-image, and Inpainting</p>
    </div>
    <div class="workspace-content">
      <div class="panel left-panel">
        <h2>Controls</h2>
        <p>Prompt and generation parameters will go here</p>
      </div>
      <div class="panel center-panel">
        <h2>Queue</h2>
        <p>Generation queue and history will go here</p>
      </div>
      <div class="panel right-panel">
        <h2>Canvas</h2>
        <p>Image preview and editing will go here</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.panel {
  padding: 1rem;
  overflow-y: auto;
  border-right: 1px solid #e9ecef;
}

.panel:last-child {
  border-right: none;
}

.left-panel {
  width: 35%;
}

.center-panel {
  width: 25%;
}

.right-panel {
  width: 40%;
}

.panel h2 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  font-weight: 600;
}
</style>
EOF
```

**Step 3: Create RefineView stub**

```bash
cat > src/views/RefineView.vue << 'EOF'
<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Refine</h1>
      <p>Model Hub and LoRA Library</p>
    </div>
    <div class="workspace-content">
      <p>Model and LoRA management will go here</p>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}
</style>
EOF
```

**Step 4: Create CompareView stub**

```bash
cat > src/views/CompareView.vue << 'EOF'
<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Compare</h1>
      <p>Smart Gallery and Image Comparison</p>
    </div>
    <div class="workspace-content">
      <p>Gallery and comparison tools will go here</p>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}
</style>
EOF
```

**Step 5: Create ManageView stub**

```bash
cat > src/views/ManageView.vue << 'EOF'
<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Manage</h1>
      <p>Settings, Performance, and Connection</p>
    </div>
    <div class="workspace-content">
      <p>Settings and system management will go here</p>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}
</style>
EOF
```

**Step 6: Test all views render**

Run: `npm run dev` and navigate to each workspace
Expected: All 4 workspaces render with headers

**Step 7: Commit workspace views**

```bash
git add src/views/
git commit -m "feat: create workspace view stubs

- Add GenerateView with 3-panel layout
- Add RefineView stub
- Add CompareView stub
- Add ManageView stub

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Create Pinia Stores

**Files:**
- Create: `src/stores/generation.ts`
- Create: `src/stores/settings.ts`

**Step 1: Create stores directory**

```bash
mkdir -p src/stores
```

**Step 2: Create generation store**

```bash
cat > src/stores/generation.ts << 'EOF'
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GenerationJob, GenerationParams } from '@/types'

export const useGenerationStore = defineStore('generation', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const currentParams = ref<GenerationParams>({
    prompt: '',
    negativePrompt: '',
    steps: 20,
    cfgScale: 7.5,
    width: 1024,
    height: 1024,
    seed: -1,
    model: 'flux-schnell'
  })

  // Getters
  const queuedJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Queued')
  )

  const runningJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Running')
  )

  const completedJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Completed')
  )

  // Actions
  function addJob(job: GenerationJob) {
    jobs.value.push(job)
  }

  function updateJobStatus(id: string, status: GenerationJob['status']) {
    const job = jobs.value.find(j => j.id === id)
    if (job) {
      job.status = status
    }
  }

  function clearCompleted() {
    jobs.value = jobs.value.filter(job => job.status !== 'Completed')
  }

  return {
    jobs,
    currentParams,
    queuedJobs,
    runningJobs,
    completedJobs,
    addJob,
    updateJobStatus,
    clearCompleted
  }
})
EOF
```

**Step 3: Create settings store**

```bash
cat > src/stores/settings.ts << 'EOF'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ConnectionMode = 'local' | 'server' | 'client'

export const useSettingsStore = defineStore('settings', () => {
  // State
  const connectionMode = ref<ConnectionMode>('local')
  const serverUrl = ref('')
  const serverPort = ref(7860)
  const apiToken = ref('')
  const outputPath = ref('')
  const modelCachePath = ref('')

  // Actions
  function setConnectionMode(mode: ConnectionMode) {
    connectionMode.value = mode
  }

  function setServerUrl(url: string) {
    serverUrl.value = url
  }

  function setApiToken(token: string) {
    apiToken.value = token
  }

  return {
    connectionMode,
    serverUrl,
    serverPort,
    apiToken,
    outputPath,
    modelCachePath,
    setConnectionMode,
    setServerUrl,
    setApiToken
  }
})
EOF
```

**Step 4: Commit Pinia stores**

```bash
git add src/stores/
git commit -m "feat: create Pinia stores for state management

- Add generation store for jobs and params
- Add settings store for connection mode
- Use Composition API pattern

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Connect Frontend to Backend (Health Check)

**Files:**
- Modify: `src/views/ManageView.vue`

**Step 1: Add health check UI to ManageView**

```bash
cat > src/views/ManageView.vue << 'EOF'
<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Manage</h1>
      <p>Settings, Performance, and Connection</p>
    </div>
    <div class="workspace-content">
      <div class="section">
        <h2>System Status</h2>
        <button @click="checkHealth" class="btn-primary">
          Check Backend Health
        </button>
        <p v-if="healthStatus" class="status-message">
          Status: {{ healthStatus }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const healthStatus = ref<string>('')

async function checkHealth() {
  try {
    const result = await invoke<string>('health_check')
    healthStatus.value = result
  } catch (error) {
    healthStatus.value = `Error: ${error}`
  }
}
</script>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.section {
  margin-bottom: 2rem;
}

.section h2 {
  margin: 0 0 1rem 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.btn-primary {
  padding: 0.5rem 1rem;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 0.375rem;
  cursor: pointer;
  font-weight: 500;
}

.btn-primary:hover {
  background: #0056b3;
}

.status-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background: #d4edda;
  border: 1px solid #c3e6cb;
  border-radius: 0.375rem;
  color: #155724;
}
</style>
EOF
```

**Step 2: Test health check**

Run: `npm run tauri:dev`
Expected: App launches, health check button returns "OK"

**Step 3: Commit frontend-backend connection**

```bash
git add src/views/ManageView.vue
git commit -m "feat: connect frontend to backend with health check

- Add health check button in ManageView
- Test Tauri command invocation
- Verify frontend-backend communication

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Initialize SQLite Database

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Expand gallery schema**

```bash
cat > src-tauri/src/gallery/mod.rs << 'EOF'
//! Gallery database and metadata management

use rusqlite::{Connection, params};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub file_path: String,
    pub prompt: String,
    pub created_at: i64,
}

pub struct GalleryDb {
    conn: Connection,
}

impl GalleryDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn init_schema(&self) -> Result<()> {
        // Main images table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                thumbnail_path TEXT,
                created_at INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                is_favorite INTEGER DEFAULT 0,

                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                model_name TEXT NOT NULL,
                steps INTEGER,
                cfg_scale REAL,
                seed INTEGER,
                sampler TEXT,

                server_id TEXT,
                generation_time_ms INTEGER
            )",
            [],
        )?;

        // Tags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;

        // Image-tags junction
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS image_tags (
                image_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                FOREIGN KEY(image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY(image_id, tag_id)
            )",
            [],
        )?;

        // Full-text search
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS images_fts USING fts5(
                image_id UNINDEXED,
                prompt,
                negative_prompt
            )",
            [],
        )?;

        Ok(())
    }

    pub fn insert_image(&self, metadata: &ImageMetadata) -> Result<()> {
        self.conn.execute(
            "INSERT INTO images (id, file_path, prompt, created_at, width, height, file_size, model_name)
             VALUES (?1, ?2, ?3, ?4, 1024, 1024, 0, 'flux-schnell')",
            params![metadata.id, metadata.file_path, metadata.prompt, metadata.created_at],
        )?;

        // Insert into FTS table
        self.conn.execute(
            "INSERT INTO images_fts (image_id, prompt, negative_prompt)
             VALUES (?1, ?2, '')",
            params![metadata.id, metadata.prompt],
        )?;

        Ok(())
    }

    pub fn get_recent_images(&self, limit: usize) -> Result<Vec<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, prompt, created_at
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let images = stmt.query_map(params![limit], |row| {
            Ok(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                prompt: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_db_init_and_insert() {
        let db = GalleryDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        let metadata = ImageMetadata {
            id: "test-id".to_string(),
            file_path: "/path/to/image.png".to_string(),
            prompt: "test prompt".to_string(),
            created_at: 1234567890,
        };

        db.insert_image(&metadata).unwrap();
        let images = db.get_recent_images(10).unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "test-id");
    }
}
EOF
```

**Step 2: Run tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass, including new gallery test

**Step 3: Add database initialization command**

Update `src-tauri/src/lib.rs`:

```bash
cat > src-tauri/src/lib.rs << 'EOF'
mod inference;
mod models;
mod queue;
mod gallery;
mod utils;

use tauri::command;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    gallery_db: Mutex<Option<gallery::GalleryDb>>,
}

#[command]
fn health_check() -> String {
    "OK".to_string()
}

#[command]
fn init_database(app_state: State<AppState>, db_path: String) -> Result<String, String> {
    let db = gallery::GalleryDb::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    db.init_schema()
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    *app_state.gallery_db.lock().unwrap() = Some(db);

    Ok("Database initialized".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        gallery_db: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
            init_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

**Step 4: Test database initialization**

Run: `cd src-tauri && cargo test`
Expected: Builds successfully

**Step 5: Commit database implementation**

```bash
git add src-tauri/
git commit -m "feat: implement SQLite database with full schema

- Expand gallery module with complete schema
- Add images, tags, image_tags, FTS tables
- Implement insert and query functions
- Add init_database Tauri command

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Add Application Initialization

**Files:**
- Modify: `src/App.vue`
- Create: `src/composables/useAppInit.ts`

**Step 1: Create composables directory and useAppInit**

```bash
mkdir -p src/composables
cat > src/composables/useAppInit.ts << 'EOF'
import { onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { homeDir } from '@tauri-apps/api/path'
import { join } from '@tauri-apps/api/path'

export function useAppInit() {
  onMounted(async () => {
    try {
      // Get home directory
      const home = await homeDir()
      const dbPath = await join(home, '.flux-generator', 'gallery.db')

      // Initialize database
      await invoke('init_database', { dbPath })
      console.log('Database initialized successfully')
    } catch (error) {
      console.error('Failed to initialize app:', error)
    }
  })
}
EOF
```

**Step 2: Update App.vue to use initialization**

```bash
cat > src/App.vue << 'EOF'
<template>
  <div class="app-container">
    <WorkspaceNav />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import WorkspaceNav from '@/components/shared/WorkspaceNav.vue'
import { useAppInit } from '@/composables/useAppInit'

// Initialize app on mount
useAppInit()
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
EOF
```

**Step 3: Test app initialization**

Run: `npm run tauri:dev`
Expected: App launches and initializes database (check console logs)

**Step 4: Commit app initialization**

```bash
git add src/
git commit -m "feat: add application initialization on startup

- Create useAppInit composable
- Initialize database on app mount
- Set up .flux-generator directory structure

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Add Basic Candle Integration Test

**Files:**
- Create: `src-tauri/src/inference/engine.rs`
- Modify: `src-tauri/src/inference/mod.rs`

**Step 1: Create inference engine module**

```bash
cat > src-tauri/src/inference/engine.rs << 'EOF'
//! Core inference engine using Candle

use anyhow::Result;
use candle_core::{Device, Tensor};

pub struct InferenceEngine {
    device: Device,
}

impl InferenceEngine {
    pub fn new() -> Result<Self> {
        // Try to use CUDA, fall back to CPU
        let device = Device::cuda_if_available(0)?;
        Ok(Self { device })
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    /// Test function to verify Candle is working
    pub fn test_tensor_ops(&self) -> Result<Vec<f32>> {
        // Create a simple tensor and perform operations
        let tensor = Tensor::new(&[1.0f32, 2.0, 3.0, 4.0], &self.device)?;
        let result = (tensor * 2.0)?;
        let data = result.to_vec1()?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = InferenceEngine::new().unwrap();
        // Just verify it can be created
        let _device = engine.get_device();
    }

    #[test]
    fn test_tensor_operations() {
        let engine = InferenceEngine::new().unwrap();
        let result = engine.test_tensor_ops().unwrap();
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);
    }
}
EOF
```

**Step 2: Update inference mod.rs**

```bash
cat > src-tauri/src/inference/mod.rs << 'EOF'
//! Inference engine for running Flux models with Candle

mod engine;

pub use engine::InferenceEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
EOF
```

**Step 3: Run Candle tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass, including Candle tensor ops

**Step 4: Commit Candle integration**

```bash
git add src-tauri/src/inference/
git commit -m "feat: add basic Candle integration with tests

- Create InferenceEngine with device detection
- Add test tensor operations to verify Candle works
- Automatic CUDA/CPU fallback

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 1 Complete!

**Verification Steps:**

1. Run all tests: `cd src-tauri && cargo test`
2. Start dev server: `npm run tauri:dev`
3. Navigate through all 4 workspaces
4. Click health check button in Manage workspace
5. Check console for database initialization

**What We Built:**

✅ Tauri + Vue project structure
✅ Modular Rust backend (inference, models, queue, gallery, utils)
✅ Basic Candle integration with tests
✅ SQLite database with full schema
✅ 4 workspace views with navigation
✅ Pinia stores for state management
✅ Frontend-backend communication
✅ App initialization on startup

**Next Steps:**

Phase 2 will implement actual image generation with Flux Schnell, progressive preview, and queue management. That will require:

1. Downloading Flux model from HuggingFace
2. Implementing text-to-image pipeline
3. Progress callback system
4. Queue worker with threading
5. Image saving and gallery integration

---

## Notes for Implementation

- **YAGNI**: We're only building what's needed for Phase 1. No premature optimization.
- **TDD**: Every module has tests. Run `cargo test` frequently.
- **Commits**: Small, frequent commits with descriptive messages.
- **Template**: Follow patterns from `/home/alex/Dev/Work/rzem-ai-mj-lora` for consistency.
- **Candle**: Currently CPU-only. GPU features (CUDA/Metal) will be added in Phase 2.
- **Database**: Schema is complete but many features unused yet. That's intentional.


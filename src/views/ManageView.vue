<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const healthStatus = ref<string>('');

async function checkHealth() {
  try {
    const result = await invoke<string>('health_check');
    healthStatus.value = `Status: ${result}`;
  } catch (error) {
    healthStatus.value = `Error: ${error}`;
  }
}
</script>

<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Manage</h1>
      <p>Settings, Performance, and Connection</p>
    </div>
    <div class="workspace-content">
      <div class="section">
        <h2>System Status</h2>
        <button @click="checkHealth" class="btn-primary">Check Backend Health</button>
        <p v-if="healthStatus" class="status-message">{{ healthStatus }}</p>
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
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.section {
  margin-bottom: 2rem;
}

.btn-primary {
  background: #007bff;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.25rem;
  cursor: pointer;
  font-size: 1rem;
}

.btn-primary:hover {
  background: #0056b3;
}

.status-message {
  margin-top: 1rem;
  padding: 0.75rem 1.25rem;
  background: #d4edda;
  border: 1px solid #c3e6cb;
  color: #155724;
  border-radius: 0.25rem;
}
</style>

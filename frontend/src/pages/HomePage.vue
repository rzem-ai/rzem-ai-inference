<script setup lang="ts">
import { onMounted } from "vue";
import Card from "primevue/card";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Tag from "primevue/tag";
import ProgressSpinner from "primevue/progressspinner";
import { usePywebview } from "@/composables/usePywebview";
import { useAppStore } from "@/stores/app";

const { api, isReady, isNative } = usePywebview();
const store = useAppStore();

onMounted(async () => {
  // Wait for bridge to be ready, then fetch initial data
  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      await store.fetchSystemInfo(api.value);
      await store.fetchCounter(api.value);
    }
  }, 50);
});
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="flex items-center gap-3">
      <h1 class="text-2xl font-bold">Demo Dashboard</h1>
      <Tag
        :value="isNative ? 'Native (pywebview)' : 'Browser (mock)'"
        :severity="isNative ? 'success' : 'warn'"
      />
    </div>

    <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
      <!-- System Info Card -->
      <Card>
        <template #title>
          <div class="flex items-center gap-2">
            <i class="pi pi-desktop" />
            System Info
          </div>
        </template>
        <template #content>
          <div v-if="store.systemInfo" class="flex flex-col gap-2 text-sm">
            <div
              v-for="(value, key) in store.systemInfo"
              :key="key"
              class="flex justify-between border-b border-surface-100 pb-1"
            >
              <span class="font-medium text-surface-500">{{ key }}</span>
              <span>{{ value }}</span>
            </div>
          </div>
          <div v-else class="flex justify-center p-4">
            <ProgressSpinner style="width: 2rem; height: 2rem" />
          </div>
        </template>
      </Card>

      <!-- Greeting Card -->
      <Card>
        <template #title>
          <div class="flex items-center gap-2">
            <i class="pi pi-user" />
            Greeting
          </div>
        </template>
        <template #content>
          <div class="flex flex-col gap-3">
            <div class="flex gap-2">
              <InputText
                v-model="store.userName"
                placeholder="Enter your name"
                class="flex-1"
                @keyup.enter="store.sendGreeting(api)"
              />
              <Button
                label="Greet"
                icon="pi pi-send"
                :loading="store.isLoading"
                @click="store.sendGreeting(api)"
              />
            </div>
            <p v-if="store.greeting" class="rounded-lg bg-primary/10 p-3 text-sm">
              {{ store.greeting }}
            </p>
          </div>
        </template>
      </Card>

      <!-- Counter Card -->
      <Card>
        <template #title>
          <div class="flex items-center gap-2">
            <i class="pi pi-calculator" />
            Counter
          </div>
        </template>
        <template #content>
          <div class="flex items-center gap-4">
            <span class="text-4xl font-bold text-primary">{{ store.counter }}</span>
            <Button
              label="Increment"
              icon="pi pi-plus"
              @click="store.incrementCounter(api)"
            />
          </div>
        </template>
      </Card>

      <!-- Component Showcase Card -->
      <Card>
        <template #title>
          <div class="flex items-center gap-2">
            <i class="pi pi-palette" />
            Component Showcase
          </div>
        </template>
        <template #content>
          <div class="flex flex-wrap gap-2">
            <Button label="Primary" />
            <Button label="Secondary" severity="secondary" />
            <Button label="Success" severity="success" />
            <Button label="Warning" severity="warn" />
            <Button label="Danger" severity="danger" />
            <Button label="Info" severity="info" />
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

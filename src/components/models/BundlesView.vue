<template>
  <div class="flex h-full">
    <!-- Details Panel -->
    <div class="flex-1 p-4 overflow-y-auto">
      <div v-if="selectedBundle">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-surface-100">{{ selectedBundle.displayName }}</h2>
          <div class="flex gap-2">
            <Button v-if="!selectedBundle.isActive" severity="secondary" size="small" @click="bundlesStore.setActiveBundle(selectedBundle.id)">
              Set Active
            </Button>
            <Button severity="danger" size="small" @click="deleteBundle(selectedBundle.id)">
              <template #icon><fa :icon="['fal', 'trash-can']" /></template>
            </Button>
          </div>
        </div>

        <p v-if="selectedBundle.description" class="mb-4 text-sm text-surface-400">{{ selectedBundle.description }}</p>

        <!-- Components -->
        <div class="mb-4">
          <h3 class="mb-2 text-sm font-semibold text-surface-300">Components</h3>
          <div class="space-y-2">
            <div v-for="item in selectedBundle.items" :key="item.id" class="flex items-center justify-between p-2 rounded bg-surface-800">
              <div class="flex items-center gap-2">
                <span class="w-20 text-xs uppercase text-surface-500">{{ item.role }}</span>
                <span class="text-sm text-surface-200">{{ item.modelDisplayName }}</span>
              </div>
              <div class="flex items-center gap-2">
                <Tag v-if="item.modelQuantization" :value="item.modelQuantization" severity="info" class="text-xs" />
                <span v-if="item.modelVramMb" class="text-xs text-surface-500">{{ bundlesStore.formatVram(item.modelVramMb) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Tags -->
        <div class="mb-4">
          <h3 class="mb-2 text-sm font-semibold text-surface-300">Tags</h3>
          <div class="flex flex-wrap gap-1">
            <span v-for="tag in selectedBundle.tags" :key="tag" class="flex items-center gap-1 px-2 py-1 text-xs rounded bg-surface-700 text-surface-300">
              {{ tag }}
              <fa :icon="['fal', 'xmark']" size="xs" class="cursor-pointer hover:text-red-400" @click="bundlesStore.deleteBundle" />
            </span>
          </div>
        </div>

        <!-- Examples -->
        <div>
          <h3 class="mb-2 text-sm font-semibold text-surface-300">Examples</h3>
          <div v-if="selectedBundle.examples.length === 0" class="text-sm text-surface-500">No examples added.</div>
          <div v-else class="space-y-1">
            <div v-for="ex in selectedBundle.examples" :key="ex.id" class="p-2 text-sm rounded text-surface-400 bg-surface-800">
              <span class="mr-2 text-xs uppercase text-surface-500">{{ ex.exampleType }}</span>
              {{ ex.content }}
            </div>
          </div>
        </div>
      </div>

      <div v-else class="flex items-center justify-center h-full">
        <p class="text-sm text-surface-500">Select a bundle to view details.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useBundlesStore } from '@/stores/bundles';
import { Button, Tag } from 'primevue';

const bundlesStore = useBundlesStore();

const selectedbundle_id = ref<string | null>(null);

const selectedBundle = computed(() => {
  if (!selectedbundle_id.value) return null;
  return bundlesStore.bundles.find((b) => b.id === selectedbundle_id.value) ?? null;
});

async function deleteBundle(id: string) {
  await bundlesStore.deleteBundle(id);
  if (selectedbundle_id.value === id) selectedbundle_id.value = null;
}
</script>

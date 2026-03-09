<template>
  <div class="flex flex-col h-full p-4 gap-2 overflow-auto">
    <!-- Header -->
    <div class="flex items-center justify-between shrink-0">
      <div class="flex items-center gap-2">
        <Grid3x3 :size="16" class="text-surface-500" />
        <span class="text-sm font-medium text-surface-600">
          {{ store.gridConfig?.xAxis.label }} vs {{ store.gridConfig?.yAxis.label }}
        </span>
        <span class="text-xs text-surface-400">
          {{ store.gridCompleted }} / {{ store.gridTotal }} completed
        </span>
      </div>
      <Button size="small" severity="secondary" variant="text" @click="store.clearGrid()">
        <X :size="14" />
        Close Grid
      </Button>
    </div>

    <!-- Grid -->
    <div v-if="store.gridConfig" class="flex-1 overflow-auto">
      <div
        class="grid gap-1"
        :style="{
          gridTemplateColumns: `auto repeat(${xValues.length}, minmax(80px, 1fr))`,
          gridTemplateRows: `auto auto repeat(${yValues.length}, minmax(80px, 1fr))`,
        }">
        <!-- Top-left corner: Y axis label -->
        <div class="flex items-end justify-end pr-2 pb-1">
          <span class="text-xs text-surface-400 font-medium">{{ store.gridConfig.yAxis.label }}</span>
        </div>
        <!-- X axis header labels -->
        <div
          v-for="(xVal, xi) in xValues"
          :key="'xh-' + xi"
          class="text-center pb-1">
          <span class="text-xs font-medium text-surface-500">{{ xVal }}</span>
        </div>

        <!-- Data rows -->
        <template v-for="(yVal, yi) in yValues" :key="'row-' + yi">
          <!-- Row header -->
          <div class="flex items-center justify-end pr-2">
            <span class="text-xs font-medium text-surface-500">{{ yVal }}</span>
          </div>
          <!-- Image cells -->
          <div
            v-for="(xVal, xi) in xValues"
            :key="'cell-' + xi + '-' + yi"
            class="relative aspect-square rounded-lg overflow-hidden cursor-pointer border border-surface-200 bg-surface-50 transition-all hover:border-surface-400"
            @click="selectCell(xi, yi)">
            <!-- Completed image -->
            <img
              v-if="getCellImage(xi, yi)?.dataUrl"
              :src="getCellImage(xi, yi)!.dataUrl"
              :alt="`${store.gridConfig!.xAxis.param}=${xVal}, ${store.gridConfig!.yAxis.param}=${yVal}`"
              class="w-full h-full object-cover" />
            <!-- Active cell (currently generating) -->
            <div
              v-else-if="isCellActive(xi, yi)"
              class="w-full h-full flex items-center justify-center bg-surface-100">
              <div class="flex flex-col items-center gap-1">
                <div class="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                <span v-if="store.progress" class="text-xs text-surface-400">
                  {{ store.progress.step }}/{{ store.progress.totalSteps }}
                </span>
              </div>
            </div>
            <!-- Empty placeholder -->
            <div v-else class="w-full h-full flex items-center justify-center">
              <div class="w-2 h-2 rounded-full bg-surface-200" />
            </div>
          </div>
        </template>
      </div>

      <!-- X axis label below the grid -->
      <div class="text-center mt-2">
        <span class="text-xs text-surface-400 font-medium">{{ store.gridConfig.xAxis.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useInferenceStore } from '@/stores/inference';
import type { GeneratedImage } from '@/types/inference';

const store = useInferenceStore();

const xValues = computed(() => store.gridConfig?.xAxis.values ?? []);
const yValues = computed(() => store.gridConfig?.yAxis.values ?? []);

function getCellImage(x: number, y: number): GeneratedImage | undefined {
  return store.gridResults.get(`${x},${y}`);
}

function isCellActive(x: number, y: number): boolean {
  if (!store.isGenerating || !store.currentJobId) return false;
  const cell = store.gridJobMap.get(store.currentJobId);
  return cell?.x === x && cell?.y === y;
}

function selectCell(x: number, y: number) {
  const img = getCellImage(x, y);
  if (!img) return;
  const idx = store.generatedImages.findIndex(i => i.jobId === img.jobId);
  if (idx >= 0) store.selectImage(idx);
}
</script>

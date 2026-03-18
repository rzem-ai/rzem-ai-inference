<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :draggable="false"
    :closable="true"
    header="X/Y Parameter Grid"
    class="w-140 max-w-[95vw]">
    <div class="flex flex-col gap-4">
      <p class="text-xs text-surface-500">
        Pick two parameters and provide values for each axis. All combinations will be generated in a grid.
      </p>

      <!-- X Axis -->
      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-slate-600">X Axis</label>
        <Select
          v-model="xParam"
          :options="availableXParams"
          option-label="label"
          option-value="value"
          placeholder="Select parameter..." />
        <!-- Numeric input -->
        <InputText
          v-if="xParamDef?.type === 'numeric'"
          v-model="xRawInput"
          placeholder="e.g. 10, 20, 30 or 10-40:10"
          class="font-mono text-sm" />
        <!-- Categorical chips -->
        <div v-if="xParamDef?.type === 'categorical'" class="flex flex-wrap gap-1">
          <button
            v-for="opt in xParamDef.options"
            :key="opt"
            class="px-2 py-1 rounded-full text-xs transition-colors cursor-pointer"
            :class="xSelectedChips.has(opt) ? 'bg-blue-600 text-white' : 'bg-surface-100 text-surface-600 hover:bg-surface-200'"
            @click="toggleChip('x', opt)">
            {{ opt }}
          </button>
        </div>
        <span v-if="xValues.length" class="text-xs text-surface-400">
          {{ xValues.length }} value{{ xValues.length !== 1 ? 's' : '' }}: {{ formatValues(xValues) }}
        </span>
      </div>

      <!-- Y Axis -->
      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-slate-600">Y Axis</label>
        <Select
          v-model="yParam"
          :options="availableYParams"
          option-label="label"
          option-value="value"
          placeholder="Select parameter..." />
        <!-- Numeric input -->
        <InputText
          v-if="yParamDef?.type === 'numeric'"
          v-model="yRawInput"
          placeholder="e.g. 1-7:1.5 or 1, 3.5, 7"
          class="font-mono text-sm" />
        <!-- Categorical chips -->
        <div v-if="yParamDef?.type === 'categorical'" class="flex flex-wrap gap-1">
          <button
            v-for="opt in yParamDef.options"
            :key="opt"
            class="px-2 py-1 rounded-full text-xs transition-colors cursor-pointer"
            :class="ySelectedChips.has(opt) ? 'bg-blue-600 text-white' : 'bg-surface-100 text-surface-600 hover:bg-surface-200'"
            @click="toggleChip('y', opt)">
            {{ opt }}
          </button>
        </div>
        <span v-if="yValues.length" class="text-xs text-surface-400">
          {{ yValues.length }} value{{ yValues.length !== 1 ? 's' : '' }}: {{ formatValues(yValues) }}
        </span>
      </div>

      <!-- Grid preview -->
      <div v-if="xValues.length && yValues.length" class="text-sm text-surface-600 bg-surface-50 rounded-lg px-3 py-2">
        {{ xValues.length }} x {{ yValues.length }} = <strong>{{ xValues.length * yValues.length }}</strong> images
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button
          severity="primary"
          :disabled="!canSubmit"
          @click="submitGrid">
          <Grid3x3 :size="14" />
          Generate Grid
        </Button>
        <Button severity="secondary" variant="outlined" @click="visibleModel = false">
          Cancel
        </Button>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue';
import { useInferenceStore } from '@/stores/inference';
import type { GridConfig } from '@/types/inference';

const visibleModel = defineModel<boolean>('visible', { required: true });

const store = useInferenceStore();

// ── Axis parameter definitions ──

interface AxisParamDef {
  value: string;
  label: string;
  type: 'numeric' | 'categorical';
  options?: string[];
}

const AXIS_PARAMS: AxisParamDef[] = [
  { value: 'steps', label: 'Steps', type: 'numeric' },
  { value: 'cfg_scale', label: 'CFG Scale', type: 'numeric' },
  { value: 'seed', label: 'Seed', type: 'numeric' },
  { value: 'sampler', label: 'Sampler', type: 'categorical', options: ['euler', 'euler_a', 'dpm++_2m', 'dpm++_2s', 'dpm++_sde', 'heun', 'lms'] },
  { value: 'scheduler', label: 'Scheduler', type: 'categorical', options: ['simple', 'normal', 'beta', 'karras', 'exponential', 'sgm_uniform', 'ddim_uniform'] },
];

const xParam = ref<string | null>(null);
const yParam = ref<string | null>(null);
const xRawInput = ref('');
const yRawInput = ref('');
const xSelectedChips = reactive(new Set<string>());
const ySelectedChips = reactive(new Set<string>());

const xParamDef = computed(() => AXIS_PARAMS.find(p => p.value === xParam.value));
const yParamDef = computed(() => AXIS_PARAMS.find(p => p.value === yParam.value));

// Prevent selecting the same param for both axes
const availableXParams = computed(() => AXIS_PARAMS.filter(p => p.value !== yParam.value));
const availableYParams = computed(() => AXIS_PARAMS.filter(p => p.value !== xParam.value));

// ── Value parsing ──

function parseNumericValues(input: string): number[] {
  const trimmed = input.trim();
  if (!trimmed) return [];

  // Range syntax: start-end:step
  const rangeMatch = trimmed.match(/^(-?\d+\.?\d*)\s*-\s*(-?\d+\.?\d*)\s*:\s*(\d+\.?\d*)$/);
  if (rangeMatch) {
    const start = parseFloat(rangeMatch[1]);
    const end = parseFloat(rangeMatch[2]);
    const step = parseFloat(rangeMatch[3]);
    if (step <= 0 || isNaN(start) || isNaN(end)) return [];
    const values: number[] = [];
    for (let v = start; v <= end + step * 0.001; v += step) {
      values.push(Math.round(v * 1000) / 1000);
    }
    return values;
  }

  // Comma-separated values
  return trimmed
    .split(',')
    .map(s => parseFloat(s.trim()))
    .filter(n => !isNaN(n));
}

const xValues = computed<(number | string)[]>(() => {
  if (!xParamDef.value) return [];
  if (xParamDef.value.type === 'categorical') return Array.from(xSelectedChips);
  return parseNumericValues(xRawInput.value);
});

const yValues = computed<(number | string)[]>(() => {
  if (!yParamDef.value) return [];
  if (yParamDef.value.type === 'categorical') return Array.from(ySelectedChips);
  return parseNumericValues(yRawInput.value);
});

function toggleChip(axis: 'x' | 'y', value: string) {
  const set = axis === 'x' ? xSelectedChips : ySelectedChips;
  if (set.has(value)) {
    set.delete(value);
  } else {
    set.add(value);
  }
}

function formatValues(values: (number | string)[]): string {
  return values.slice(0, 8).join(', ') + (values.length > 8 ? '...' : '');
}

// ── Submit ──

const canSubmit = computed(() => {
  return xParam.value && yParam.value && xValues.value.length > 0 && yValues.value.length > 0 && (store.engineReady || store.isCloudBundle) && !store.isGenerating;
});

function submitGrid() {
  if (!canSubmit.value || !xParamDef.value || !yParamDef.value) return;

  const config: GridConfig = {
    xAxis: { param: xParamDef.value.value, values: xValues.value, label: xParamDef.value.label },
    yAxis: { param: yParamDef.value.value, values: yValues.value, label: yParamDef.value.label },
  };

  store.submitGrid(config);
  visibleModel.value = false;
}
</script>

<!-- Dev controls (Ctrl+Shift+D) -->
<template>
  <div class="border-t border-slate-200 pt-3 flex flex-col gap-3">
    <div class="flex items-center gap-1">
      <Terminal :size="12" class="text-orange-500" />
      <span class="text-[10px] font-bold text-orange-500 uppercase">Dev Mode</span>
    </div>

    <!-- Debug events -->
    <div class="flex flex-col gap-1">
      <label class="text-[10px] font-medium text-slate-500">Debug Events</label>
      <Select
        v-model="debugEvent"
        :options="debugEventOptions"
        option-label="label"
        option-value="value"
        placeholder="Inject event..."
        size="small"
        fluid
        @change="onDebugEvent" />
    </div>

    <!-- Engine controls -->
    <div class="flex gap-1">
      <button
        v-if="!store.engineReady"
        class="flex-1 text-[11px] py-1.5 rounded bg-green-500 text-white hover:bg-green-600 transition-colors disabled:opacity-50"
        :disabled="store.engineStarting"
        @click="store.startEngine()">
        {{ store.engineStarting ? 'Starting...' : 'Start Engine' }}
      </button>
      <button v-else class="flex-1 text-[11px] py-1.5 rounded bg-red-500 text-white hover:bg-red-600 transition-colors" @click="store.stopEngine()">
        Stop Engine
      </button>
      <div class="text-[10px] px-2 py-1.5 rounded font-medium" :class="store.engineReady ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'">
        {{ store.engineReady ? 'Ready' : 'Stopped' }}
      </div>
    </div>

    <!-- Model path overrides -->
    <div class="flex flex-col gap-1.5">
      <label class="text-[10px] font-medium text-slate-500">Transformer</label>
      <input v-model="store.params.transformer_model" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">VAE</label>
      <input v-model="store.params.vae_model" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">CLIP Tokenizer</label>
      <input v-model="store.params.clip_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">CLIP Encoder</label>
      <input v-model="store.params.clip_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">T5 Tokenizer</label>
      <input v-model="store.params.t5_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">T5 Encoder</label>
      <input v-model="store.params.t5_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">Qwen3 Tokenizer</label>
      <input v-model="store.params.qwen3_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
      <label class="text-[10px] font-medium text-slate-500">Qwen3 Encoder</label>
      <input v-model="store.params.qwen3_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Terminal } from 'lucide-vue-next';
import Select from 'primevue/select';

import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

const debugEvent = ref<string | null>(null);

const debugEventOptions = [
  { label: 'default (reset)', value: 'default' },
  { label: 'model_loading', value: 'model_loading' },
  { label: 'model_loaded', value: 'model_loaded' },
  { label: 'model_unloaded', value: 'model_unloaded' },
  { label: 'job_queued', value: 'job_queued' },
  { label: 'job_started', value: 'job_started' },
  { label: 'job_progress', value: 'job_progress' },
  { label: 'job_progress_0', value: 'job_progress_0' },
  { label: 'job_progress_5', value: 'job_progress_5' },
  { label: 'job_progress_10', value: 'job_progress_10' },
  { label: 'job_progress_15', value: 'job_progress_15' },
  { label: 'job_progress_20', value: 'job_progress_20' },
  { label: 'job_progress_25', value: 'job_progress_25' },
  { label: 'job_progress_28', value: 'job_progress_28' },
  { label: 'job_completed', value: 'job_completed' },
  { label: 'job_failed', value: 'job_failed' },
  { label: 'job_cancelled', value: 'job_cancelled' },
];

function onDebugEvent(e: { value: string }) {
  if (e.value) {
    store.injectDebugEvent(e.value);
  }
}
</script>

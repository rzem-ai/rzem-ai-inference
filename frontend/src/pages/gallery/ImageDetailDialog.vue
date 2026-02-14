<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :draggable="false"
    :closable="true"
    header="Image Details"
    class="w-[700px] max-w-[95vw]">
    <div class="flex flex-col gap-4">
      <!-- Prompt -->
      <div>
        <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">Prompt</label>
        <p class="mt-1 text-sm leading-relaxed">{{ image.prompt }}</p>
      </div>

      <!-- Negative prompt -->
      <div v-if="image.negative_prompt">
        <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">Negative Prompt</label>
        <p class="mt-1 text-sm leading-relaxed text-surface-500">{{ image.negative_prompt }}</p>
      </div>

      <!-- Parameters grid -->
      <div class="grid grid-cols-3 gap-3">
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Size</span>
          <span class="block text-sm mt-0.5 font-mono">{{ image.width }} x {{ image.height }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Steps</span>
          <span class="block text-sm mt-0.5 font-mono">{{ image.steps }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">CFG Scale</span>
          <span class="block text-sm mt-0.5 font-mono">{{ image.cfg_scale }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Seed</span>
          <span class="block text-sm mt-0.5 font-mono">{{ image.seed }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Sampler</span>
          <span class="block text-sm mt-0.5 font-mono">{{ parsedConfig?.sampler ?? '—' }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Scheduler</span>
          <span class="block text-sm mt-0.5 font-mono">{{ parsedConfig?.scheduler ?? '—' }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2 col-span-3">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Model</span>
          <span class="block text-sm mt-0.5 font-mono">{{ modelDisplayName }}</span>
        </div>
        <div v-if="image.generation_time_ms" class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Generation Time</span>
          <span class="block text-sm mt-0.5 font-mono">{{ formatDuration(image.generation_time_ms) }}</span>
        </div>
        <div v-if="image.file_size" class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">File Size</span>
          <span class="block text-sm mt-0.5 font-mono">{{ formatFileSize(image.file_size) }}</span>
        </div>
        <div class="rounded-lg bg-surface-50 px-3 py-2">
          <span class="block text-[10px] font-medium text-surface-400 uppercase tracking-wide">Created</span>
          <span class="block text-sm mt-0.5 font-mono">{{ formatDate(image.created_at) }}</span>
        </div>
      </div>

      <!-- LoRAs -->
      <div v-if="parsedLoras.length > 0">
        <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">LoRAs</label>
        <div class="mt-1 flex flex-wrap gap-1.5">
          <span
            v-for="lora in parsedLoras"
            :key="lora.model_file"
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-surface-100 text-xs">
            {{ loraDisplayName(lora.model_file) }}
            <span class="text-surface-400">{{ lora.strength }}</span>
          </span>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button severity="primary" @click="usePrompt">
          <RotateCcw :size="14" />
          Use Prompt
        </Button>
        <Button severity="secondary" variant="outlined" @click="visibleModel = false">
          Close
        </Button>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { RotateCcw } from 'lucide-vue-next';
import { useRouter } from 'vue-router';
import { Dialog, Button } from 'primevue';
import { useInferenceStore, ASPECT_RATIOS } from '@/stores/inference';
import type { GalleryImage, LoraParam } from '@/types/inference';

const props = defineProps<{
  image: GalleryImage;
}>();

const visibleModel = defineModel<boolean>('visible', { required: true });

const emit = defineEmits<{
  closeAll: [];
}>();

const router = useRouter();
const inference = useInferenceStore();

// Parse the JSON model_config snapshot
const parsedConfig = computed(() => {
  if (!props.image.model_config) return null;
  try {
    return JSON.parse(props.image.model_config);
  } catch {
    return null;
  }
});

// Parse LoRAs JSON
const parsedLoras = computed((): LoraParam[] => {
  if (!props.image.loras) return [];
  try {
    return JSON.parse(props.image.loras);
  } catch {
    return [];
  }
});

// Extract a readable model name from the full HF path
const modelDisplayName = computed(() => {
  const model = parsedConfig.value?.transformer_model ?? parsedConfig.value?.model;
  if (!model) return '—';
  const parts = model.split('/');
  if (parts.length >= 2) {
    const name = parts[parts.length - 1];
    return name.endsWith('.gguf') ? name : parts.slice(-2).join('/');
  }
  return model;
});

function loraDisplayName(path: string): string {
  const parts = path.split('/');
  return parts[parts.length - 1];
}

function formatDuration(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = (seconds % 60).toFixed(0);
  return `${minutes}m ${remainingSeconds}s`;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

async function usePrompt() {
  const img = props.image;
  const config = parsedConfig.value;

  // Restore prompt and generation params
  inference.params.prompt = img.prompt;
  inference.params.width = img.width;
  inference.params.height = img.height;
  inference.params.steps = img.steps;
  inference.params.cfg_scale = img.cfg_scale;
  inference.params.seed = img.seed;

  // Restore sampler/scheduler from model_config snapshot
  if (config?.sampler) inference.params.sampler = config.sampler;
  if (config?.scheduler) inference.params.scheduler = config.scheduler;

  // Restore model settings from config snapshot
  if (config?.transformer_model) inference.params.transformer_model = config.transformer_model;
  if (config?.transformer_type) inference.params.transformer_type = config.transformer_type;
  if (config?.vae_model) inference.params.vae_model = config.vae_model;

  // Restore LoRAs
  inference.params.loras = parsedLoras.value.map(l => ({
    model_file: l.model_file,
    strength: l.strength,
  }));

  // Try to match and apply the original bundle
  if (img.bundle_id) {
    const bundle = inference.bundles.find(b => b.id === img.bundle_id);
    if (bundle) {
      inference.applyBundle(bundle);
      // Re-apply image-specific overrides that applyBundle may have changed
      inference.params.prompt = img.prompt;
      inference.params.width = img.width;
      inference.params.height = img.height;
      inference.params.seed = img.seed;
    } else {
      inference.selectedBundleId = img.bundle_id;
    }
  }

  // Find matching aspect ratio
  const matchingRatio = ASPECT_RATIOS.find(
    r => r.width === img.width && r.height === img.height,
  );
  inference.activeAspectRatio = matchingRatio?.label ?? null;

  // Close everything and navigate
  visibleModel.value = false;
  emit('closeAll');
  await router.push({ name: 'create' });
}
</script>


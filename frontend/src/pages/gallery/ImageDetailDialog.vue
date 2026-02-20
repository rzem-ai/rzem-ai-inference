<template>
  <Dialog v-model:visible="visibleModel" modal :draggable="false" :closable="true" header="Image Details" class="w-250 max-w-[95vw]">
    <div class="flex flex-col gap-4">
      <!-- Prompt -->
      <div class="rounded-lg bg-surface-50 px-3 py-2">
        <label class="text-base font-semibold text-surface-900 uppercase tracking-wide">Prompt</label>
        <div class="text-base p-2 leading-relaxed font-mono">{{ image.prompt }}</div>
      </div>

      <!-- Negative prompt -->
      <div v-if="image.negative_prompt">
        <label class="text-sm font-semibold text-surface-800 uppercase tracking-wide">Negative Prompt</label>
        <p class="mt-1 text-base leading-relaxed text-surface-500">{{ image.negative_prompt }}</p>
      </div>

      <!-- Parameters grid -->
      <div class="grid grid-cols-3 gap-3">
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-600 uppercase tracking-wide">Size</div>
          <div class="text-base font-mono">{{ image.width }} x {{ image.height }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Steps</div>
          <div class="text-base mt-1 font-mono">{{ image.steps }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">CFG Scale</div>
          <div class="text-base mt-1 font-mono">{{ image.cfg_scale }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Seed</div>
          <div class="text-base mt-1 font-mono">{{ image.seed }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Sampler</div>
          <div class="text-base mt-1 font-mono">{{ parsedConfig?.sampler ?? '—' }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Scheduler</div>
          <div class="text-base mt-1 font-mono">{{ parsedConfig?.scheduler ?? '—' }}</div>
        </div>
        <div v-if="image.generation_time_ms" class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Generation Time</div>
          <div class="text-base mt-1 font-mono">{{ formatDuration(image.generation_time_ms) }}</div>
        </div>
        <div v-if="image.file_size" class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">File Size</div>
          <div class="text-base mt-1 font-mono">{{ formatFileSize(image.file_size) }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2">
          <div class="text-base font-semibold text-surface-400 uppercase tracking-wide">Created</div>
          <div class="text-base mt-1 font-mono">{{ formatDate(image.created_at) }}</div>
        </div>
        <div class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2 col-div-3 col-span-3">
          <div class="text-base font-medium text-surface-400 uppercase tracking-wide">Model</div>
          <div class="text-base mt-1 font-mono">{{ modelDisplayName }}</div>
        </div>
        <!-- LoRAs -->
        <div v-if="parsedLoras.length > 0" class="flex flex-col gap-1 rounded-lg bg-surface-50 px-3 py-2 col-div-3 col-span-3">
          <div class="text-base font-medium text-surface-400 uppercase tracking-wide">LoRAs</div>
          <div class="flex flex-wrap gap-2">
            <Tag v-for="lora in parsedLoras" :key="lora.model_file">
              {{ loraDisplayName(lora.model_file) }}
              <div class="text-surface-400">{{ lora.strength }}</div>
            </Tag>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button severity="primary" @click="usePrompt">
          <ClipboardCheck :size="14" />
          Use Prompt
        </Button>
        <Button severity="secondary" variant="outlined" @click="visibleModel = false"> Close </Button>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { ClipboardCheck } from 'lucide-vue-next';
import { useRouter } from 'vue-router';
import { Dialog, Button, Tag } from 'primevue';
import { useInferenceStore, ASPECT_RATIOS } from '@/stores/inference';
import { usePywebview } from '@/composables/usePywebview';
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
const { api } = usePywebview();

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
  const params: Record<string, any> = {
    prompt: img.prompt,
    width: img.width,
    height: img.height,
    steps: img.steps,
    cfg_scale: img.cfg_scale,
    seed: img.seed,
  };

  // Restore sampler/scheduler from model_config snapshot
  if (config?.sampler) params.sampler = config.sampler;
  if (config?.scheduler) params.scheduler = config.scheduler;

  // Restore model settings from config snapshot
  if (config?.transformer_model) params.transformer_model = config.transformer_model;
  if (config?.transformer_type) params.transformer_type = config.transformer_type;
  if (config?.vae_model) params.vae_model = config.vae_model;

  // Restore LoRAs
  params.loras = parsedLoras.value.map((l) => ({
    model_file: l.model_file,
    strength: l.strength,
  }));

  inference.applyParams(params);

  // Try to match and apply the original bundle
  if (img.bundle_id) {
    const bundle = inference.bundles.find((b) => b.id === img.bundle_id);
    if (bundle) {
      inference.applyBundle(bundle);
      // Re-apply image-specific overrides that applyBundle may have changed
      inference.applyParams({
        prompt: img.prompt,
        width: img.width,
        height: img.height,
        seed: img.seed,
      });
    } else {
      inference.setSelectedBundleId(img.bundle_id);
    }
  }

  // Restore style if one was used
  const styleId = config?.style_id;
  if (styleId) {
    const res = await api.value.get_style({ style_id: styleId });
    if (res.status === 'success' && res.style) {
      inference.applyStyle(
        res.style.id,
        res.style.prompt_template,
        res.style.negative_prompt,
        res.loras ?? [],
      );
      // Use the stored raw prompt (pre-template) so it doesn't get double-expanded
      if (img.raw_prompt) {
        inference.applyParams({ prompt: img.raw_prompt });
      }
    }
  } else {
    inference.clearStyle();
  }

  // Find matching aspect ratio
  const matchingRatio = ASPECT_RATIOS.find((r) => r.width === img.width && r.height === img.height);
  inference.setActiveAspectRatio(matchingRatio?.label ?? null);

  // Close everything and navigate
  visibleModel.value = false;
  emit('closeAll');
  await router.push({ name: 'create' });
}
</script>

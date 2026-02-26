<template>
  <div class="flex flex-col h-full overflow-y-auto p-4 gap-4">
    <!-- Empty state -->
    <div v-if="!bundle" class="flex flex-col items-center justify-center h-full text-slate-400 gap-3">
      <Box :size="48" stroke-width="1" />
      <div class="text-lg font-medium">Select a model</div>
      <div class="text-sm">Choose a model from the sidebar to view its details</div>
    </div>

    <!-- Bundle detail -->
    <template v-else>
      <!-- Header -->
      <div class="flex items-start justify-between gap-4">
        <div class="flex flex-col gap-1">
          <div class="text-2xl font-bold text-slate-900">{{ bundle.label }}</div>
          <div class="text-sm text-slate-500">{{ bundle.description }}</div>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <Button
            :severity="bundle.favorite ? 'warn' : 'secondary'"
            size="small"
            outlined
            :title="bundle.favorite ? 'Remove from favorites' : 'Add to favorites'"
            @click="modelsStore.toggleFavorite(bundle.id)">
            <Star :size="16" :class="bundle.favorite ? 'fill-amber-400 text-amber-400' : ''" />
          </Button>
          <Button
            :severity="bundle.hidden ? 'danger' : 'secondary'"
            size="small"
            outlined
            :title="bundle.hidden ? 'Show in dropdowns' : 'Hide from dropdowns'"
            @click="modelsStore.toggleHidden(bundle.id)">
            <EyeOff v-if="bundle.hidden" :size="16" />
            <Eye v-else :size="16" />
          </Button>
        </div>
      </div>

      <!-- Badges -->
      <div class="flex items-center gap-2 flex-wrap">
        <Tag :severity="tierSeverity(bundle.tier)" class="uppercase">{{ bundle.tier }}</Tag>
        <Tag v-if="bundle.source === 'cloud'" severity="info">
          <div class="flex items-center gap-1"><Cloud :size="12" /> Cloud</div>
        </Tag>
        <Tag v-else severity="secondary">
          <div class="flex items-center gap-1"><Gpu :size="12" /> Local</div>
        </Tag>
        <Tag v-if="bundle.vram_estimate_gb > 0" severity="secondary">~{{ bundle.vram_estimate_gb }} GB VRAM</Tag>
        <Tag v-if="bundle.hidden" severity="warn">Hidden</Tag>
      </div>

      <!-- Model Components -->
      <div>
        <div class="text-lg font-semibold text-slate-900 mb-3">Model Components</div>
        <div class="bg-white rounded-xl border border-slate-200 p-4 flex flex-col gap-3">
          <div class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">Transformer</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ bundle.transformer_model }}</div>
          </div>
          <div class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">Type</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ typeLabel }}</div>
          </div>
          <div class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">VAE</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ bundle.vae_model }}</div>
          </div>
          <div v-if="bundle.clip_encoder" class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">CLIP Encoder</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ bundle.clip_encoder }}</div>
          </div>
          <div v-if="bundle.t5_encoder" class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">T5 Encoder</div>
            <div class="text-sm font-mono text-slate-900 break-all">
              {{ bundle.t5_encoder }}
              <Tag v-if="t5QuantLabel" severity="info" class="ml-2 align-middle">{{ t5QuantLabel }}</Tag>
            </div>
          </div>
          <div v-if="bundle.qwen3_encoder" class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">Qwen3 Encoder</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ bundle.qwen3_encoder }}</div>
          </div>
          <div v-if="bundle.fal_endpoint" class="flex items-baseline gap-3">
            <div class="text-sm font-medium text-slate-500 w-40 shrink-0">FAL Endpoint</div>
            <div class="text-sm font-mono text-slate-900 break-all">{{ bundle.fal_endpoint }}</div>
          </div>
        </div>
      </div>

      <!-- Default Parameters -->
      <div>
        <div class="text-lg font-semibold text-slate-900 mb-3">Default Parameters</div>
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="grid grid-cols-4 gap-4">
            <div>
              <div class="text-sm font-medium text-slate-500 mb-1">Steps</div>
              <div class="text-base font-semibold text-slate-900">{{ bundle.steps }}</div>
            </div>
            <div>
              <div class="text-sm font-medium text-slate-500 mb-1">CFG Scale</div>
              <div class="text-base font-semibold text-slate-900">{{ bundle.cfg_scale }}</div>
            </div>
            <div>
              <div class="text-sm font-medium text-slate-500 mb-1">Sampler</div>
              <div class="text-base font-semibold text-slate-900">{{ bundle.sampler }}</div>
            </div>
            <div>
              <div class="text-sm font-medium text-slate-500 mb-1">Scheduler</div>
              <div class="text-base font-semibold text-slate-900">{{ bundle.scheduler }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Aspect Ratios (FAL Cloud only) -->
      <div v-if="bundle.fal_aspectratio?.length">
        <div class="text-lg font-semibold text-slate-900 mb-3">Supported Aspect Ratios</div>
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="flex flex-wrap gap-2">
            <Tag v-for="ar in bundle.fal_aspectratio" :key="ar" severity="secondary">{{ ar }}</Tag>
          </div>
        </div>
      </div>

      <!-- Guide (rendered markdown) -->
      <div v-if="guideHtml">
        <div class="text-lg font-semibold text-slate-900 mb-3">Guide</div>
        <div class="bg-white rounded-xl border border-slate-200 p-4 prose prose-sm prose-slate max-w-none" v-html="guideHtml" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useModelsStore } from '@/stores/models';

const modelsStore = useModelsStore();
const { selectedBundle: bundle } = storeToRefs(modelsStore);

const guideHtml = computed(() => modelsStore.guideHtml);
const typeLabel = computed(() => {
  if (!bundle.value) return '';
  return modelsStore.getTypeLabel(bundle.value.transformer_type);
});

const t5QuantLabel = computed(() => {
  const cfg = bundle.value?.t5_encoder_config;
  if (!cfg || typeof cfg === 'string') return null;
  if (cfg.load_in_4bit) return `NF4${cfg.bnb_4bit_quant_type === 'fp4' ? ' (FP4)' : ''}`;
  if (cfg.load_in_8bit) return 'INT8';
  return null;
});

function tierSeverity(tier: string): string {
  switch (tier) {
    case 'performance': return 'success';
    case 'balanced': return 'info';
    case 'quality': return 'danger';
    default: return 'secondary';
  }
}

onMounted(() => {
  modelsStore.loadBundleTypes();
  modelsStore.loadBundles();
});
</script>

<style scoped>
.prose :deep(h2) {
  font-size: 1rem;
  font-weight: 600;
  margin-top: 1.25rem;
  margin-bottom: 0.5rem;
  color: var(--p-slate-800);
}

.prose :deep(h2:first-child) {
  margin-top: 0;
}

.prose :deep(p) {
  margin-top: 0.25rem;
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

.prose :deep(ul) {
  padding-left: 1.25rem;
  margin-top: 0.25rem;
  margin-bottom: 0.5rem;
}

.prose :deep(li) {
  margin-bottom: 0.25rem;
  line-height: 1.5;
}

.prose :deep(strong) {
  color: var(--p-slate-900);
}

.prose :deep(code) {
  font-size: 0.8rem;
  background: var(--p-slate-100);
  padding: 0.1rem 0.3rem;
  border-radius: 0.25rem;
}
</style>

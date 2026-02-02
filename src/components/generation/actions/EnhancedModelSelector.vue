<template>
  <div class="space-y-3">
    <!-- Model/Bundle Selector -->
    <div class="flex flex-col">
      <div class="flex items-center justify-between pr-2 mb-1">
        <label class="text-sm font-medium tracking-wide text-surface-300">Model Configuration</label>
        <fa
          :icon="['fal', 'circle-info']"
          size="sm"
          class="transition-colors text-surface-400 hover:text-blue-500 cursor-help"
          v-tooltip.top="'Choose a bundle for preset configuration, or select individual model with custom components'" />
      </div>

      <Select
        v-model="selectedOption"
        :options="allOptions"
        option-label="label"
        option-value="value"
        option-group-label="label"
        option-group-children="items"
        placeholder="Select model or bundle"
        size="small"
        class="w-full">
        <template #value="slotProps">
          <div class="flex items-center justify-between w-full gap-2">
            <div class="flex items-center gap-2">
              <fa :icon="['fal', getOptionIcon(slotProps.value)]" />
              <span class="font-medium">{{ getOptionLabel(slotProps.value) }}</span>
            </div>
            <div class="flex items-center gap-2">
              <Tag v-if="isBundle(slotProps.value)" value="Bundle" severity="info" class="text-xs" />
              <span v-if="getOptionVram(slotProps.value)" class="text-xs text-surface-400">
                {{ getOptionVram(slotProps.value) }}
              </span>
            </div>
          </div>
        </template>

        <template #optiongroup="slotProps">
          <div class="flex items-center gap-2 py-2 font-semibold text-surface-300 bg-surface-800/50">
            <fa :icon="['far', slotProps.option.icon]" />
            <span class="text-sm tracking-wider uppercase"> {{ slotProps.option.label }} </span>
          </div>
        </template>

        <template #option="slotProps">
          <div class="flex items-center justify-between w-full px-2">
            <div class="flex items-center gap-2">
              <fa :icon="['fal', getOptionIcon(slotProps.option.value)]" />
              <span class="font-normal">{{ slotProps.option.label }}</span>
            </div>
            <div class="flex items-center gap-2">
              <Tag v-if="slotProps.option.isBundle" value="Bundle" severity="info" class="text-xs" />
              <Tag v-if="slotProps.option.isActive" value="Active" severity="success" class="text-xs" />
              <Tag v-if="slotProps.option.quantization" :value="slotProps.option.quantization" severity="warn" class="text-xs" />
              <span v-if="slotProps.option.vram" class="text-xs text-surface-400">
                {{ slotProps.option.vram }}
              </span>
            </div>
          </div>
        </template>
      </Select>

      <!-- Bundle Description -->
      <div v-if="selectedBundle" class="px-1 mt-2">
        <Message class="">
          <template #icon><fa :icon="['fal', 'circle-info']" /></template>
          <div class="flex flex-row">
            <div v-if="selectedBundle.description" class="text-sm text-surface-300">
              {{ selectedBundle.description }}
            </div>
            <div class="flex flex-col text-xs text-surface-400">
              <div class="text-nowrap">
                Components: <span class="font-semibold text-mono text-surface-300">{{ selectedBundle.components.length }}</span>
              </div>
              <div class="text-nowrap" v-if="selectedBundle.totalVramMb">
                VRAM: <span class="font-semibold text-mono text-surface-300">{{ bundlesStore.formatVram(selectedBundle.totalVramMb) }}</span>
              </div>
            </div>
          </div>
        </Message>
      </div>
    </div>

    <!-- Component Selectors (shown when individual model selected) -->
    <template v-if="isIndividualModel">
      <div class="p-3 border-2 border-dashed rounded-lg border-surface-700">
        <div class="flex items-center gap-2 mb-3">
          <fa :icon="['fal', 'cog']" size="sm" />
          <span class="text-sm font-medium text-surface-300">Component Configuration</span>
        </div>

        <!-- T5 Encoder Selector -->
        <div class="mb-3">
          <label class="block mb-1 text-xs font-medium text-surface-400">
            T5 Text Encoder
            <span class="text-red-400">*</span>
          </label>
          <Select
            v-model="selectedT5"
            :options="compatibleT5Components"
            option-label="name"
            option-value="id"
            placeholder="Select T5 encoder..."
            size="small"
            class="w-full">
            <template #value="slotProps">
              <div v-if="slotProps.value" class="flex items-center justify-between w-full gap-2">
                <div class="flex items-center gap-2">
                  <i
                    :class="
                      getComponentById(slotProps.value, bundlesStore.t5Components)?.isAvailable
                        ? 'pi pi-check-circle text-green-400 text-xs'
                        : 'pi pi-times-circle text-red-400 text-xs'
                    "></i>
                  <span class="text-sm">{{ getComponentById(slotProps.value, bundlesStore.t5Components)?.name }}</span>
                </div>
                <Tag
                  v-if="getComponentById(slotProps.value, bundlesStore.t5Components)?.quantization"
                  :value="getComponentById(slotProps.value, bundlesStore.t5Components)!.quantization"
                  severity="info"
                  class="text-xs" />
              </div>
              <span v-else class="text-surface-500">Select T5 encoder...</span>
            </template>
            <template #option="slotProps">
              <div class="flex items-center justify-between w-full gap-2">
                <div class="flex items-center gap-2">
                  <i :class="slotProps.option.isAvailable ? 'pi pi-check-circle text-green-400 text-xs' : 'pi pi-times-circle text-red-400 text-xs'"></i>
                  <span class="text-sm">{{ slotProps.option.name }}</span>
                </div>
                <div class="flex items-center gap-2">
                  <Tag v-if="slotProps.option.quantization" :value="slotProps.option.quantization" severity="info" class="text-xs" />
                  <span v-if="slotProps.option.vramMb" class="text-xs text-surface-500">
                    {{ slotProps.option.vramMb >= 1000 ? `${(slotProps.option.vramMb / 1000).toFixed(1)} GB` : `${slotProps.option.vramMb} MB` }}
                  </span>
                </div>
              </div>
            </template>
          </Select>
        </div>

        <!-- CLIP Encoder Selector -->
        <div class="mb-3">
          <label class="block mb-1 text-xs font-medium text-surface-400">
            CLIP Text Encoder
            <span class="text-red-400">*</span>
          </label>
          <Select
            v-model="selectedClip"
            :options="compatibleClipComponents"
            option-label="name"
            option-value="id"
            placeholder="Select CLIP encoder..."
            size="small"
            class="w-full">
            <template #value="slotProps">
              <div v-if="slotProps.value" class="flex items-center gap-2">
                <i
                  :class="
                    getComponentById(slotProps.value, bundlesStore.clipComponents)?.isAvailable
                      ? 'pi pi-check-circle text-green-400 text-xs'
                      : 'pi pi-times-circle text-red-400 text-xs'
                  "></i>
                <span class="text-sm">{{ getComponentById(slotProps.value, bundlesStore.clipComponents)?.name }}</span>
              </div>
              <span v-else class="text-surface-500">Select CLIP encoder...</span>
            </template>
            <template #option="slotProps">
              <div class="flex items-center justify-between w-full gap-2">
                <div class="flex items-center gap-2">
                  <i :class="slotProps.option.isAvailable ? 'pi pi-check-circle text-green-400 text-xs' : 'pi pi-times-circle text-red-400 text-xs'"></i>
                  <span class="text-sm">{{ slotProps.option.name }}</span>
                </div>
                <span v-if="slotProps.option.vramMb" class="text-xs text-surface-500">
                  {{ slotProps.option.vramMb >= 1000 ? `${(slotProps.option.vramMb / 1000).toFixed(1)} GB` : `${slotProps.option.vramMb} MB` }}
                </span>
              </div>
            </template>
          </Select>
        </div>

        <!-- VAE Decoder Selector -->
        <div>
          <label class="block mb-1 text-xs font-medium text-surface-400">
            VAE Decoder
            <span class="text-red-400">*</span>
          </label>
          <Select
            v-model="selectedVae"
            :options="compatibleVaeComponents"
            option-label="name"
            option-value="id"
            placeholder="Select VAE decoder..."
            size="small"
            class="w-full">
            <template #value="slotProps">
              <div v-if="slotProps.value" class="flex items-center gap-2">
                <i
                  :class="
                    getComponentById(slotProps.value, bundlesStore.vaeComponents)?.isAvailable
                      ? 'pi pi-check-circle text-green-400 text-xs'
                      : 'pi pi-times-circle text-red-400 text-xs'
                  "></i>
                <span class="text-sm">{{ getComponentById(slotProps.value, bundlesStore.vaeComponents)?.name }}</span>
              </div>
              <span v-else class="text-surface-500">Select VAE decoder...</span>
            </template>
            <template #option="slotProps">
              <div class="flex items-center justify-between w-full gap-2">
                <div class="flex items-center gap-2">
                  <i :class="slotProps.option.isAvailable ? 'pi pi-check-circle text-green-400 text-xs' : 'pi pi-times-circle text-red-400 text-xs'"></i>
                  <span class="text-sm">{{ slotProps.option.name }}</span>
                </div>
                <span v-if="slotProps.option.vramMb" class="text-xs text-surface-500">
                  {{ slotProps.option.vramMb >= 1000 ? `${(slotProps.option.vramMb / 1000).toFixed(1)} GB` : `${slotProps.option.vramMb} MB` }}
                </span>
              </div>
            </template>
          </Select>
        </div>

        <!-- Validation Warning -->
        <Message v-if="!areComponentsValid" severity="warn" size="small" class="mt-3">
          <template #icon>
            <fa :icon="['fal', 'triangle-exclamation']" size="lg" />
          </template>
          Please select all required components (T5, CLIP, VAE)
        </Message>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { useBundlesStore } from '@/stores/bundles';
import type { BundleInfo, ComponentRecord } from '@/stores/bundles';
import { Select, Tag, Message } from 'primevue';

const generationStore = useGenerationStore();
const bundlesStore = useBundlesStore();

// Selected option (either bundle ID or model ID)
const selectedOption = computed({
  get: () => {
    // If bundle is selected, return bundle:id
    if (generationStore.currentParams.bundleId) {
      return `bundle:${generationStore.currentParams.bundleId}`;
    }

    // Otherwise return model ID (may be undefined initially)
    return generationStore.currentParams.modelComponentId || undefined;
  },
  set: (value: string | undefined) => {
    if (!value) return;

    if (value.startsWith('bundle:')) {
      // Bundle selected
      const bundleId = value.substring(7);
      const bundle = bundlesStore.bundles.find((b) => b.id === bundleId);

      if (bundle) {
        generationStore.currentParams.bundleId = bundleId;
        generationStore.currentParams.modelComponentId = inferModelIdFromBundle(bundle);
        generationStore.currentParams.t5ComponentId = inferT5EncoderIdFromBundle(bundle);
        generationStore.currentParams.clipComponentId = inferClipEncoderIdFromBundle(bundle);
        generationStore.currentParams.vaeComponentId = inferVaeIdFromBundle(bundle);
      }
    } else {
      // Individual model selected
      generationStore.currentParams.modelComponentId = value;
      generationStore.currentParams.bundleId = undefined;
      // Try to auto-select compatible components
      autoSelectComponents(value);
    }
  },
});

// Component selectors (only for individual models)
const selectedT5 = computed({
  get: () => generationStore.currentParams.t5ComponentId || '',
  set: (value: string) => {
    generationStore.currentParams.t5ComponentId = value;
  },
});

const selectedClip = computed({
  get: () => generationStore.currentParams.clipComponentId || '',
  set: (value: string) => {
    generationStore.currentParams.clipComponentId = value;
  },
});

const selectedVae = computed({
  get: () => generationStore.currentParams.vaeComponentId || '',
  set: (value: string) => {
    generationStore.currentParams.vaeComponentId = value;
  },
});

// Check if current selection is an individual model (not bundle)
const isIndividualModel = computed(() => {
  return !generationStore.currentParams.bundleId;
});

// Get selected bundle
const selectedBundle = computed(() => {
  if (!generationStore.currentParams.bundleId) return null;
  return bundlesStore.bundles.find((b) => b.id === generationStore.currentParams.bundleId);
});

// Validate component selections
const areComponentsValid = computed(() => {
  if (!isIndividualModel.value) return true;
  return !!(selectedT5.value && selectedClip.value && selectedVae.value);
});

// Validation for complete configuration (exposed to parent)
const isValidConfiguration = computed(() => {
  // Bundle mode: bundleId must be set
  if (generationStore.currentParams.bundleId) {
    return true;
  }

  // Individual mode: all component IDs must be set
  return !!(
    generationStore.currentParams.modelComponentId &&
    generationStore.currentParams.t5ComponentId &&
    generationStore.currentParams.clipComponentId &&
    generationStore.currentParams.vaeComponentId
  );
});

// Expose validation state to parent
defineExpose({
  isValidConfiguration,
});

// Build options list with bundles first, then individual transformer models
interface SelectOption {
  label: string;
  value: string;
  isBundle: boolean;
  isActive?: boolean;
  vram?: string;
  quantization?: string;
}

interface OptionGroup {
  label: string;
  icon: string;
  items: SelectOption[];
}

const allOptions = computed(() => {
  const groups: OptionGroup[] = [];

  // Group 1: Bundles (pre-configured complete setups)
  if (bundlesStore.bundles.length > 0) {
    const bundleOptions: SelectOption[] = bundlesStore.bundles
      .filter((b) => b.isComplete) // Only show complete bundles
      .map((b) => ({
        label: b.name,
        value: `bundle:${b.id}`,
        isBundle: true,
        isActive: b.isActive,
        vram: b.totalVramMb ? bundlesStore.formatVram(b.totalVramMb) : undefined,
      }));

    if (bundleOptions.length > 0) {
      groups.push({
        label: 'Model Bundles',
        icon: 'box-taped',
        items: bundleOptions,
      });
    }
  }

  // Group 2: Individual Transformer Models (FLUX models that can be paired with custom components)
  const transformerOptions: SelectOption[] = bundlesStore.transformerComponents
    .filter((c) => c.isAvailable)
    .map((c) => ({
      label: c.name,
      value: c.id,
      isBundle: false,
      vram: c.vramMb ? bundlesStore.formatVram(c.vramMb) : undefined,
      quantization: c.quantization ?? undefined,
    }));

  if (transformerOptions.length > 0) {
    groups.push({
      label: 'FLUX Models',
      icon: 'microchip-ai',
      items: transformerOptions,
    });
  }

  return groups;
});

// Get compatible components based on selected model
const compatibleT5Components = computed(() => {
  return filterCompatibleComponents(bundlesStore.t5Components, generationStore.currentParams.modelComponentId ?? '');
});

const compatibleClipComponents = computed(() => {
  return filterCompatibleComponents(bundlesStore.clipComponents, generationStore.currentParams.modelComponentId ?? '');
});

const compatibleVaeComponents = computed(() => {
  return filterCompatibleComponents(bundlesStore.vaeComponents, generationStore.currentParams.modelComponentId ?? '');
});

// Filter components by compatibility with selected model
function filterCompatibleComponents(components: ComponentRecord[], modelId: string): ComponentRecord[] {
  if (!modelId) return components;

  // Determine model family from model ID
  const modelFamily = getModelFamily(modelId);

  // Filter components that match the model family
  return components.filter((comp) => {
    // Check if component is available
    if (!comp.isAvailable) return false;

    // Architecture-based compatibility
    const compArch = comp.architecture?.toLowerCase() || '';

    // For FLUX models
    if (modelFamily === 'flux') {
      // T5 and CLIP are shared across FLUX models
      if (comp.componentType === 't5_encoder' || comp.componentType === 'clip_encoder') {
        return compArch.includes('t5') || compArch.includes('clip') || compArch.includes('flux');
      }
      // VAE decoder should be FLUX VAE
      if (comp.componentType === 'vae') {
        return compArch.includes('flux') || compArch.includes('vae');
      }
    }

    // For Z-Image models
    if (modelFamily === 'zindex') {
      // Z-Image may have different text encoders
      return true; // For now, allow all available components
    }

    // Default: allow all available components
    return true;
  });
}

// Get model family from model ID
function getModelFamily(modelId: string): string {
  if (modelId.includes('zimage') || modelId.toLowerCase().includes('z-image')) {
    return 'zindex';
  }
  return 'flux'; // Default to FLUX
}

// Auto-select compatible components when model changes
function autoSelectComponents(_modelId: string) {
  // Find best matching components (filtered by model in computed properties)
  const t5Options = compatibleT5Components.value;
  const clipOptions = compatibleClipComponents.value;
  const vaeOptions = compatibleVaeComponents.value;

  // Auto-select first available component of each type
  if (t5Options.length > 0 && !selectedT5.value) {
    selectedT5.value = t5Options[0].id;
  }

  if (clipOptions.length > 0 && !selectedClip.value) {
    selectedClip.value = clipOptions[0].id;
  }

  if (vaeOptions.length > 0 && !selectedVae.value) {
    selectedVae.value = vaeOptions[0].id;
  }
}

// Helper functions
function isBundle(value: string | undefined): boolean {
  return value ? value.startsWith('bundle:') : false;
}

function getOptionIcon(value: string | undefined): string {
  // if (!value) return 'pi pi-question text-surface-400';
  // if (isBundle(value)) {
  //   return 'pi pi-box text-blue-400';
  // }
  // return 'pi pi-microchip text-purple-400';

  if (!value) {
    return 'square-question';
  }

  if (isBundle(value)) {
    return 'box-taped';
  }

  return 'layer-group';
}

function getOptionLabel(value: string | undefined): string {
  if (!value) return 'Select...';
  if (isBundle(value)) {
    const bundleId = value.substring(7);
    const bundle = bundlesStore.bundles.find((b) => b.id === bundleId);
    return bundle?.name || value;
  }
  // Look up in transformer components
  const transformer = bundlesStore.transformerComponents.find((c) => c.id === value);
  return transformer?.name || value;
}

function getOptionVram(value: string | undefined): string | undefined {
  if (!value) return undefined;
  if (isBundle(value)) {
    const bundleId = value.substring(7);
    const bundle = bundlesStore.bundles.find((b) => b.id === bundleId);
    return bundle?.totalVramMb ? bundlesStore.formatVram(bundle.totalVramMb) : undefined;
  }
  // Look up in transformer components
  const transformer = bundlesStore.transformerComponents.find((c) => c.id === value);
  return transformer?.vramMb ? bundlesStore.formatVram(transformer.vramMb) : undefined;
}

function getComponentById(id: string, components: ComponentRecord[]): ComponentRecord | null {
  return components.find((c) => c.id === id) || null;
}

// Infer model type from bundle
function inferModelIdFromBundle(bundle: BundleInfo): string {
  // Check transformer component architecture
  const transformer = bundle.components.find((c: any) => c.role === 'transformer');
  return transformer ? transformer.id : 'schnell';
}

function inferClipEncoderIdFromBundle(bundle: BundleInfo): string {
  // Check transformer component architecture
  const transformer = bundle.components.find((c: any) => c.role === 'clip' || c.role === 'clip_encoder');
  return transformer ? transformer.id : 'clip-l';
}

function inferT5EncoderIdFromBundle(bundle: BundleInfo): string {
  // Check transformer component architecture
  const transformer = bundle.components.find((c: any) => c.role === 't5' || c.role === 't5_encoder');
  return transformer ? transformer.id : 't5-v1_1-xxl';
}

function inferVaeIdFromBundle(bundle: BundleInfo): string {
  // Check transformer component architecture
  const transformer = bundle.components.find((c: any) => c.role === 'vae');
  return transformer ? transformer.id : 'flux-vae';
}

// Initialize bundles store on mount
onMounted(async () => {
  await bundlesStore.initialize();

  // If no selection yet, try to use active bundle, or first available transformer
  if (!selectedOption.value) {
    if (bundlesStore.activeBundle) {
      selectedOption.value = `bundle:${bundlesStore.activeBundle.id}`;
    } else if (bundlesStore.transformerComponents.length > 0) {
      // Fall back to first available transformer
      const firstTransformer = bundlesStore.transformerComponents.find((c) => c.isAvailable);
      if (firstTransformer) {
        selectedOption.value = firstTransformer.id;
      }
    }
  }
});
</script>

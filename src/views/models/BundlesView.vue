<template>
  <div class="flex-1 h-full p-6 overflow-y-auto rounded-xl bg-surface-950">
    <template v-if="selectedBundle">
      <!-- Bundle Header -->
      <div class="flex items-start justify-between pb-6 mb-6 border-b border-surface-800">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <h2 class="m-0 text-xl font-semibold text-surface-50">{{ selectedBundle.name }}</h2>
            <Tag v-if="selectedBundle.isActive" value="Active" severity="success" />
            <Tag v-if="!selectedBundle.isComplete" value="Incomplete" severity="warning" />
          </div>
          <p v-if="selectedBundle.description" class="m-0 text-sm text-surface-400">
            {{ selectedBundle.description }}
          </p>
        </div>

        <div class="flex gap-2">
          <Button
            v-if="!selectedBundle.isActive && selectedBundle.isComplete"
            label="Activate"
            icon="pi pi-check"
            size="small"
            @click="handleActivateBundle(selectedBundle)" />
          <Button
            v-if="selectedBundle.bundleType === 'user_created'"
            icon="pi pi-trash"
            size="small"
            severity="danger"
            outlined
            @click="handleDeleteBundle(selectedBundle)" />
        </div>
      </div>

      <!-- Bundle Info -->
      <div class="mb-6">
        <h3 class="mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Bundle Information</h3>
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Type</span>
            <span class="text-sm font-medium text-surface-200">{{ formatBundleType(selectedBundle.bundleType) }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Model Family</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedBundle.modelFamily.toUpperCase() }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Total VRAM</span>
            <span class="text-sm font-medium text-surface-200">
              {{ bundlesStore.formatVram(selectedBundle.totalVramMb) }}
            </span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Components</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedBundle.components.length }}</span>
          </div>
        </div>
      </div>

      <!-- Components List -->
      <div class="mb-6">
        <h3 class="mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Components</h3>
        <div class="space-y-2">
          <div v-for="comp in selectedBundle.components" :key="comp.id" class="p-3 rounded-lg bg-surface-800">
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <fa v-if="comp.isAvailable" :icon="['fal', 'file-check']" size="xl" class="text-green-400" />
                <fa v-else :icon="['fal', 'file-slash']" size="xl" class="text-red-400" />
                <span class="text-sm font-medium text-surface-200">{{ comp.name }}</span>
                <Tag v-if="comp.quantization" :value="comp.quantization" severity="info" class="text-xs" />
              </div>
            </div>
            <div class="grid grid-cols-3 gap-3 text-xs text-surface-500">
              <div>
                <span>Role:</span>
                <span class="ml-1 text-surface-300">{{ formatComponentRole(comp.role) }}</span>
              </div>
              <div>
                <span>Format:</span>
                <span class="ml-1 text-surface-300">{{ comp.format }}</span>
              </div>
              <div v-if="comp.vramMb">
                <span>VRAM:</span>
                <span class="ml-1 text-surface-300">{{ bundlesStore.formatVram(comp.vramMb) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Default Settings (if available) -->
      <div v-if="selectedBundle.defaultSteps || selectedBundle.defaultGuidance" class="mb-6">
        <h3 class="mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Default Settings</h3>
        <div class="grid grid-cols-2 gap-4 p-4 rounded-lg bg-surface-800">
          <div v-if="selectedBundle.defaultSteps" class="flex items-center justify-between">
            <span class="text-sm text-surface-400">Steps</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedBundle.defaultSteps }}</span>
          </div>
          <div v-if="selectedBundle.defaultGuidance" class="flex items-center justify-between">
            <span class="text-sm text-surface-400">Guidance</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedBundle.defaultGuidance }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- Empty State -->
    <div v-else class="flex flex-col items-center justify-center h-full gap-4 text-surface-500">
      <i class="text-6xl pi pi-box"></i>
      <p class="m-0 text-sm">Select a bundle to view details</p>
      <Button label="Create Bundle" icon="pi pi-plus" size="small" @click="showBundleCreator = true" />
    </div>
  </div>

  <!-- Bundle Creator Dialog -->
  <BundleCreator v-model:visible="showBundleCreator" :bundle="editingBundle" @created="handleBundleCreated" @updated="handleBundleUpdated" />

  <!-- Confirmation Dialog -->
  <ConfirmDialog />
  <Toast />
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import { useBundlesStore } from '@/stores/bundles';
import type { BundleInfo } from '@/stores/bundles';
import { useToast } from 'primevue/usetoast';
import { useConfirm } from 'primevue/useconfirm';
import BundleCreator from '@/components/models/BundleCreator.vue';

const bundlesStore = useBundlesStore();
const toast = useToast();
const confirm = useConfirm();

const showBundleCreator = ref(false);
const editingBundle = ref<BundleInfo | undefined>(undefined);

const selectedBundle = computed(() => bundlesStore.activeBundle);

// Expose selectedBundle for parent to set
defineExpose({ selectedBundle });

async function handleActivateBundle(bundle: BundleInfo) {
  if (!bundle.isComplete) {
    toast.add({
      severity: 'warn',
      summary: 'Incomplete Bundle',
      detail: 'This bundle is missing required components',
      life: 3000,
    });
    return;
  }

  try {
    await bundlesStore.setActiveBundle(bundle.id);

    toast.add({
      severity: 'success',
      summary: 'Bundle Activated',
      detail: `${bundle.name} is now active`,
      life: 3000,
    });
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: 'Activation Failed',
      detail: String(err),
      life: 5000,
    });
  }
}

function handleDeleteBundle(bundle: BundleInfo) {
  confirm.require({
    message: `Delete bundle "${bundle.name}"? This cannot be undone.`,
    header: 'Confirm Deletion',
    icon: 'pi pi-exclamation-triangle',
    acceptClass: 'p-button-danger',
    accept: async () => {
      try {
        await bundlesStore.deleteBundle(bundle.id);

        toast.add({
          severity: 'success',
          summary: 'Bundle Deleted',
          detail: `${bundle.name} has been deleted`,
          life: 3000,
        });

        //selectedBundle.value = null;
      } catch (err) {
        toast.add({
          severity: 'error',
          summary: 'Deletion Failed',
          detail: String(err),
          life: 5000,
        });
      }
    },
  });
}

function handleBundleCreated() {
  editingBundle.value = undefined;
  toast.add({
    severity: 'success',
    summary: 'Bundle Created',
    detail: 'New bundle has been created',
    life: 3000,
  });
}

function handleBundleUpdated() {
  editingBundle.value = undefined;
  toast.add({
    severity: 'success',
    summary: 'Bundle Updated',
    detail: 'Bundle has been updated',
    life: 3000,
  });
}

function formatBundleType(type: string): string {
  switch (type) {
    case 'auto_discovered':
      return 'Auto-Discovered';
    case 'user_created':
      return 'User Created';
    case 'system':
      return 'System';
    default:
      return type;
  }
}

function formatComponentRole(role: string): string {
  const roleMap: Record<string, string> = {
    transformer: 'Transformer',
    t5: 'T5 Encoder',
    clip: 'CLIP Encoder',
    vae: 'VAE Decoder',
    clip_tokenizer: 'CLIP Tokenizer',
    t5_tokenizer: 'T5 Tokenizer',
  };
  return roleMap[role] || role;
}

onMounted(() => {
  // Auto-select first bundle
  if (bundlesStore.bundles.length > 0) {
    //selectedBundle.value = bundlesStore.bundles[0];
  }
});
</script>

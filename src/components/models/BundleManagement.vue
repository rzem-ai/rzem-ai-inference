<template>
  <div class="bundle-management">
    <!-- Bundle Section Header -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-3">
        <div>
          <h2 class="text-xl font-semibold text-surface-50">Model Bundles</h2>
          <p class="text-sm text-surface-400 mt-1">
            Bundles group model components together for easy switching and management
          </p>
        </div>
        <div class="flex gap-2">
          <Button
            label="Scan Models"
            icon="pi pi-refresh"
            size="small"
            severity="secondary"
            outlined
            :loading="bundlesStore.isScanning"
            @click="handleScan"
          />
          <Button
            label="Create Bundle"
            icon="pi pi-plus"
            size="small"
            @click="showCreator = true"
          />
        </div>
      </div>

      <!-- Active Bundle Status -->
      <div v-if="bundlesStore.activeBundle" class="p-4 mb-4 border rounded-lg border-green-800/50 bg-green-900/10">
        <div class="flex items-start gap-3">
          <i class="pi pi-check-circle text-green-400 text-xl mt-0.5"></i>
          <div class="flex-1">
            <p class="m-0 text-sm font-medium text-green-300">Active Bundle</p>
            <p class="m-0 mt-1 text-lg font-semibold text-surface-100">
              {{ bundlesStore.activeBundle.name }}
            </p>
            <p v-if="bundlesStore.activeBundle.description" class="m-0 mt-1 text-sm text-surface-400">
              {{ bundlesStore.activeBundle.description }}
            </p>
            <div class="flex gap-3 mt-2 text-xs text-surface-500">
              <span>{{ bundlesStore.activeBundle.components.length }} components</span>
              <span v-if="bundlesStore.activeBundle.totalVramMb">
                {{ bundlesStore.formatVram(bundlesStore.activeBundle.totalVramMb) }} VRAM
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- No Active Bundle Warning -->
      <Message v-else severity="warn" :closable="false">
        No bundle active. Using legacy hardcoded paths. Activate a bundle for better control.
      </Message>

      <!-- Scan Result Message -->
      <Message v-if="scanMessage" :severity="scanSeverity" @close="scanMessage = null">
        {{ scanMessage }}
      </Message>

      <!-- Error Message -->
      <Message v-if="bundlesStore.error" severity="error" @close="bundlesStore.clearError()">
        {{ bundlesStore.error }}
      </Message>
    </div>

    <!-- Tabs for Bundle Organization -->
    <TabView class="bundle-tabs">
      <!-- All Bundles Tab -->
      <TabPanel value="0" header="All Bundles">
        <BundleSelector @editBundle="handleEditBundle" />
      </TabPanel>

      <!-- FLUX Bundles Tab -->
      <TabPanel value="1" header="FLUX Bundles">
        <div v-if="bundlesStore.fluxBundles.length === 0" class="text-center py-8 text-surface-500">
          <p>No FLUX bundles found</p>
        </div>
        <div v-else class="space-y-3">
          <BundleCard
            v-for="bundle in bundlesStore.fluxBundles"
            :key="bundle.id"
            :bundle="bundle"
            @activate="handleActivate"
            @edit="handleEditBundle"
            @delete="handleDeleteBundle"
          />
        </div>
      </TabPanel>

      <!-- Z-Index Bundles Tab -->
      <TabPanel value="2" header="Z-Index Bundles">
        <div v-if="bundlesStore.zindexBundles.length === 0" class="text-center py-8 text-surface-500">
          <p>No Z-Index bundles found</p>
        </div>
        <div v-else class="space-y-3">
          <BundleCard
            v-for="bundle in bundlesStore.zindexBundles"
            :key="bundle.id"
            :bundle="bundle"
            @activate="handleActivate"
            @edit="handleEditBundle"
            @delete="handleDeleteBundle"
          />
        </div>
      </TabPanel>

      <!-- Components Tab -->
      <TabPanel value="3" header="Components">
        <ComponentList />
      </TabPanel>
    </TabView>

    <!-- Bundle Creator Dialog -->
    <BundleCreator
      v-model:visible="showCreator"
      :bundle="editingBundle"
      @created="handleBundleCreated"
      @updated="handleBundleUpdated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBundlesStore } from '@/stores/bundles'
import type { BundleInfo } from '@/stores/bundles'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import Button from 'primevue/button'
import Message from 'primevue/message'
import TabView from 'primevue/tabview'
import TabPanel from 'primevue/tabpanel'
import BundleSelector from './BundleSelector.vue'
import BundleCreator from './BundleCreator.vue'
import BundleCard from './BundleCard.vue'
import ComponentList from './ComponentList.vue'

const bundlesStore = useBundlesStore()
const toast = useToast()
const confirm = useConfirm()

const showCreator = ref(false)
const editingBundle = ref<BundleInfo | undefined>(undefined)
const scanMessage = ref<string | null>(null)
const scanSeverity = ref<'success' | 'info' | 'warn' | 'error'>('success')

onMounted(async () => {
  await bundlesStore.initialize()
})

async function handleScan() {
  scanMessage.value = null

  try {
    const result = await bundlesStore.scanModels()

    if (result.componentsAdded > 0 || result.bundlesCreated > 0) {
      scanMessage.value = `Found ${result.componentsFound} components, added ${result.componentsAdded} new, created ${result.bundlesCreated} bundles`
      scanSeverity.value = 'success'
    } else {
      scanMessage.value = 'Scan complete. No new models found.'
      scanSeverity.value = 'info'
    }

    toast.add({
      severity: 'success',
      summary: 'Scan Complete',
      detail: scanMessage.value,
      life: 3000,
    })
  } catch (err) {
    scanSeverity.value = 'error'
    scanMessage.value = `Scan failed: ${err}`
  }
}

async function handleActivate(bundle: BundleInfo) {
  if (bundle.isActive) return

  if (!bundle.isComplete) {
    toast.add({
      severity: 'warn',
      summary: 'Incomplete Bundle',
      detail: 'This bundle is missing required components',
      life: 3000,
    })
    return
  }

  try {
    await bundlesStore.setActiveBundle(bundle.id)

    toast.add({
      severity: 'success',
      summary: 'Bundle Activated',
      detail: `${bundle.name} is now active`,
      life: 3000,
    })
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: 'Activation Failed',
      detail: String(err),
      life: 5000,
    })
  }
}

function handleEditBundle(bundle: BundleInfo) {
  editingBundle.value = bundle
  showCreator.value = true
}

function handleDeleteBundle(bundle: BundleInfo) {
  confirm.require({
    message: `Delete bundle "${bundle.name}"? This cannot be undone.`,
    header: 'Confirm Deletion',
    icon: 'pi pi-exclamation-triangle',
    acceptClass: 'p-button-danger',
    accept: async () => {
      try {
        await bundlesStore.deleteBundle(bundle.id)

        toast.add({
          severity: 'success',
          summary: 'Bundle Deleted',
          detail: `${bundle.name} has been deleted`,
          life: 3000,
        })
      } catch (err) {
        toast.add({
          severity: 'error',
          summary: 'Deletion Failed',
          detail: String(err),
          life: 5000,
        })
      }
    },
  })
}

function handleBundleCreated() {
  editingBundle.value = undefined
  toast.add({
    severity: 'success',
    summary: 'Bundle Created',
    detail: 'New bundle has been created successfully',
    life: 3000,
  })
}

function handleBundleUpdated() {
  editingBundle.value = undefined
  toast.add({
    severity: 'success',
    summary: 'Bundle Updated',
    detail: 'Bundle has been updated successfully',
    life: 3000,
  })
}
</script>

<style scoped>
.bundle-tabs :deep(.p-tabview-nav) {
  background: transparent;
  border-bottom: 1px solid rgb(var(--surface-800));
}

.bundle-tabs :deep(.p-tabview-panels) {
  background: transparent;
  padding: 1.5rem 0;
}
</style>

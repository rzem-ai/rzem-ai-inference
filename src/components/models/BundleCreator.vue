<template>
  <Dialog
    v-model:visible="isVisible"
    modal
    :header="editMode ? 'Edit Bundle' : 'Create Custom Bundle'"
    :style="{ width: '800px' }"
    @hide="handleClose"
  >
    <div class="space-y-4">
      <!-- Bundle Name -->
      <div>
        <label class="block text-sm font-medium mb-2">Bundle Name</label>
        <InputText
          v-model="formData.name"
          placeholder="e.g., My Custom FLUX Bundle"
          class="w-full"
          :invalid="!!errors.name"
        />
        <small v-if="errors.name" class="text-red-500">{{ errors.name }}</small>
      </div>

      <!-- Description -->
      <div>
        <label class="block text-sm font-medium mb-2">Description (optional)</label>
        <Textarea
          v-model="formData.description"
          placeholder="Describe this bundle..."
          rows="2"
          class="w-full"
        />
      </div>

      <!-- Model Family -->
      <div>
        <label class="block text-sm font-medium mb-2">Model Family</label>
        <Select
          v-model="formData.modelFamily"
          :options="modelFamilies"
          option-label="label"
          option-value="value"
          placeholder="Select model family"
          class="w-full"
          :invalid="!!errors.modelFamily"
        />
        <small v-if="errors.modelFamily" class="text-red-500">{{ errors.modelFamily }}</small>
      </div>

      <!-- Component Selection -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <label class="block text-sm font-medium">Components</label>
          <small class="text-surface-500">{{ selectedCount }}/4 required components selected</small>
        </div>

        <!-- Transformer -->
        <ComponentPicker
          label="Transformer"
          role="transformer"
          :components="bundlesStore.transformerComponents"
          :selected-id="formData.componentMap.transformer"
          :required="true"
          @update:selected-id="(val) => formData.componentMap.transformer = val || ''"
        />

        <!-- T5 Encoder -->
        <ComponentPicker
          label="T5 Text Encoder"
          role="t5"
          :components="bundlesStore.t5Components"
          :selected-id="formData.componentMap.t5"
          :required="true"
          @update:selected-id="(val) => formData.componentMap.t5 = val || ''"
        />

        <!-- CLIP Encoder -->
        <ComponentPicker
          label="CLIP Text Encoder"
          role="clip"
          :components="bundlesStore.clipComponents"
          :selected-id="formData.componentMap.clip"
          :required="true"
          @update:selected-id="(val) => formData.componentMap.clip = val || ''"
        />

        <!-- VAE Decoder -->
        <ComponentPicker
          label="VAE Decoder"
          role="vae"
          :components="bundlesStore.vaeComponents"
          :selected-id="formData.componentMap.vae"
          :required="true"
          @update:selected-id="(val) => formData.componentMap.vae = val || ''"
        />
      </div>

      <!-- Validation Summary -->
      <Message v-if="!isValid" severity="warn">
        Please select all required components (Transformer, T5, CLIP, VAE)
      </Message>
    </div>

    <!-- Footer Actions -->
    <template #footer>
      <div class="flex justify-end gap-2">
        <Button
          label="Cancel"
          severity="secondary"
          text
          @click="handleClose"
        />
        <Button
          :label="editMode ? 'Update' : 'Create Bundle'"
          icon="pi pi-check"
          :disabled="!isValid"
          :loading="isCreating"
          @click="handleSubmit"
        />
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBundlesStore } from '@/stores/bundles'
import type { BundleInfo } from '@/stores/bundles'
import { useToast } from 'primevue/usetoast'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import Select from 'primevue/select'
import Message from 'primevue/message'
import ComponentPicker from './ComponentPicker.vue'

interface Props {
  visible: boolean
  bundle?: BundleInfo
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  created: [bundleId: string]
  updated: []
}>()

const bundlesStore = useBundlesStore()
const toast = useToast()

const isVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
})

const editMode = computed(() => !!props.bundle)

const formData = ref({
  name: '',
  description: '',
  modelFamily: 'flux',
  componentMap: {
    transformer: '',
    t5: '',
    clip: '',
    vae: '',
  },
})

const errors = ref({
  name: '',
  modelFamily: '',
})

const isCreating = ref(false)
const scanMessage = ref<string | null>(null)

const modelFamilies = [
  { label: 'FLUX', value: 'flux' },
  { label: 'Z-Index', value: 'zindex' },
]

const selectedCount = computed(() => {
  return Object.values(formData.value.componentMap).filter(id => id).length
})

const isValid = computed(() => {
  return (
    formData.value.name.trim().length > 0 &&
    formData.value.modelFamily &&
    formData.value.componentMap.transformer &&
    formData.value.componentMap.t5 &&
    formData.value.componentMap.clip &&
    formData.value.componentMap.vae
  )
})

// Watch for bundle changes (edit mode)
watch(() => props.bundle, (bundle) => {
  if (bundle) {
    formData.value.name = bundle.name
    formData.value.description = bundle.description || ''
    formData.value.modelFamily = bundle.modelFamily

    // Map components to roles
    const componentMap = {
      transformer: '',
      t5: '',
      clip: '',
      vae: '',
    }

    for (const comp of bundle.components) {
      if (comp.role in componentMap) {
        componentMap[comp.role as keyof typeof componentMap] = comp.id
      }
    }

    formData.value.componentMap = componentMap
  }
}, { immediate: true })

function resetForm() {
  formData.value = {
    name: '',
    description: '',
    modelFamily: 'flux',
    componentMap: {
      transformer: '',
      t5: '',
      clip: '',
      vae: '',
    },
  }
  errors.value = {
    name: '',
    modelFamily: '',
  }
  scanMessage.value = null
}

function validateForm(): boolean {
  errors.value = {
    name: '',
    modelFamily: '',
  }

  if (!formData.value.name.trim()) {
    errors.value.name = 'Bundle name is required'
    return false
  }

  if (!formData.value.modelFamily) {
    errors.value.modelFamily = 'Model family is required'
    return false
  }

  return true
}

async function handleSubmit() {
  if (!validateForm() || !isValid.value) {
    return
  }

  isCreating.value = true

  try {
    if (editMode.value) {
      // Update existing bundle
      await bundlesStore.updateBundle(
        props.bundle!.id,
        formData.value.name,
        formData.value.description,
      )

      toast.add({
        severity: 'success',
        summary: 'Bundle Updated',
        detail: `${formData.value.name} has been updated`,
        life: 3000,
      })

      emit('updated')
    } else {
      // Create new bundle
      const bundleId = await bundlesStore.createBundle({
        name: formData.value.name,
        description: formData.value.description || undefined,
        modelFamily: formData.value.modelFamily,
        componentMap: formData.value.componentMap,
      })

      toast.add({
        severity: 'success',
        summary: 'Bundle Created',
        detail: `${formData.value.name} has been created`,
        life: 3000,
      })

      emit('created', bundleId)
    }

    handleClose()
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: editMode.value ? 'Update Failed' : 'Creation Failed',
      detail: String(err),
      life: 5000,
    })
  } finally {
    isCreating.value = false
  }
}

function handleClose() {
  isVisible.value = false
  resetForm()
}
</script>

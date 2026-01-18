<template>
  <div class="preset-selector">
    <div class="field">
      <label for="preset">Preset</label>
      <div class="preset-controls">
        <Select
          id="preset"
          v-model="selectedPresetId"
          :options="presetsStore.presets"
          option-label="name"
          option-value="id"
          placeholder="Select a preset"
          class="flex-1"
          @change="handleLoadPreset" />
        <Button icon="pi pi-save" severity="secondary" @click="showSaveDialog = true" title="Save current settings as preset" />
      </div>
    </div>

    <Dialog v-model:visible="showSaveDialog" modal header="Save Preset" :style="{ width: '350px' }">
      <div class="save-dialog-content">
        <label for="preset-name">Preset Name</label>
        <InputText id="preset-name" v-model="newPresetName" placeholder="My Preset" class="w-full" @keyup.enter="handleSavePreset" />
      </div>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showSaveDialog = false" />
        <Button label="Save" @click="handleSavePreset" :disabled="!newPresetName.trim()" />
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { usePresetsStore } from '@/stores/presets';
import Select from 'primevue/select';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Dialog from 'primevue/dialog';

const presetsStore = usePresetsStore();

const selectedPresetId = ref<string | null>(null);
const showSaveDialog = ref(false);
const newPresetName = ref('');

const handleLoadPreset = () => {
  if (selectedPresetId.value) {
    presetsStore.loadPreset(selectedPresetId.value);
  }
};

const handleSavePreset = () => {
  if (newPresetName.value.trim()) {
    presetsStore.savePreset(newPresetName.value.trim());
    newPresetName.value = '';
    showSaveDialog.value = false;
  }
};
</script>

<style scoped>
@reference "tailwindcss";

.preset-selector {
  @apply flex flex-col gap-2;
}

.preset-controls {
  @apply flex gap-2 items-center;
}

.save-dialog-content {
  @apply flex flex-col gap-3 py-4;
}
</style>

<template>
  <div class="flex flex-col">
    <button
      class="flex items-center justify-between w-full text-xs font-medium tracking-wide text-gray-400 transition-colors cursor-pointer hover:text-gray-300">
      <span>Presets</span>
    </button>

    <div class="overflow-hidden">
      <div class="flex items-center gap-2">
        <Select
          id="preset"
          v-model="selectedPresetId"
          :options="presetsStore.presets"
          option-label="name"
          option-value="id"
          placeholder="Select a preset"
          class="flex-1"
          size="small"
          @change="handleLoadPreset" />
        <Button @click="showSaveDialog = true" title="Save current settings as preset">
          <BookMarked class="w-4 h-4" />
        </Button>
      </div>
    </div>

    <Dialog v-model:visible="showSaveDialog" modal header="Save Preset" :style="{ width: '350px' }">
      <div class="flex flex-col gap-3 py-4">
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
import { BookMarked } from 'lucide-vue-next';

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

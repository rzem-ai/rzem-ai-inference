<template>
  <div class="px-2 py-4 h-full">
    <Panel class="menu-panel">
      <template #default>
        <div>
          <!-- Prompt -->
          <EditorContent :editor="editor" class="w-full border rounded border-blue-500" />

          <!-- Engine Controls -->
          <div v-if="showEngine" class="flex items-center gap-2">
            <Button
              v-if="!store.engineReady"
              label="Start Engine"
              icon="pi pi-play"
              size="small"
              :loading="store.engineStarting"
              @click="store.startEngine()"
              class="flex-1" />
            <Button v-else label="Stop Engine" icon="pi pi-stop" size="small" severity="danger" @click="store.stopEngine()" class="flex-1" />
            <Tag :value="store.engineReady ? 'Ready' : 'Stopped'" :severity="store.engineReady ? 'success' : 'secondary'" />
          </div>

          <!-- Model Status -->
          <div v-if="store.modelStatus" class="text-xs text-surface-500 px-1">
            {{ store.modelStatus }}
          </div>

          <!-- Error -->
          <div v-if="store.error" class="text-xs text-red-500 px-1 bg-red-50 rounded p-2">
            {{ store.error }}
          </div>

          <!-- Model Selection -->

          <div class="flex flex-col gap-2">
            <Select
              v-model="store.selectedPresetId"
              :options="store.presets"
              option-label="label"
              option-value="id"
              placeholder="Select model preset"
              fluid
              @change="onPresetChange" />
            <p v-if="store.selectedPreset" class="text-xs text-surface-500">
              {{ store.selectedPreset.description }}
            </p>

            <div class="flex items-center gap-2 mt-1">
              <ToggleSwitch v-model="showAdvanced" />
              <span class="text-xs text-surface-500">Advanced model config</span>
            </div>

            <template v-if="showAdvanced">
              <div class="flex flex-col gap-2 mt-1">
                <label class="text-xs font-medium text-surface-600">Transformer</label>
                <InputText v-model="store.params.transformer_model" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">VAE</label>
                <InputText v-model="store.params.vae_model" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">CLIP Tokenizer</label>
                <InputText v-model="store.params.clip_tokenizer" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">CLIP Encoder</label>
                <InputText v-model="store.params.clip_encoder" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">T5 Tokenizer</label>
                <InputText v-model="store.params.t5_tokenizer" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">T5 Encoder</label>
                <InputText v-model="store.params.t5_encoder" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">Qwen3 Tokenizer</label>
                <InputText v-model="store.params.qwen3_tokenizer" size="small" class="w-full" />

                <label class="text-xs font-medium text-surface-600">Qwen3 Encoder</label>
                <InputText v-model="store.params.qwen3_encoder" size="small" class="w-full" />
              </div>
            </template>
          </div>

          <!-- Generation Parameters -->
          <Fieldset legend="Parameters" :toggleable="true">
            <div class="grid grid-cols-2 gap-4 items-center justify-between">
              <!-- Steps -->
              <label class="text-xs font-medium text-surface-600">Steps</label>
              <InputNumber v-model="store.params.steps" :min="1" :max="100" size="small" fluid />

              <!-- CFG Scale -->

              <label class="text-xs font-medium text-surface-600 grow">CFG Scale</label>
              <InputNumber
                v-model="store.params.cfg_scale"
                :min="0"
                :max="30"
                :min-fraction-digits="1"
                :max-fraction-digits="1"
                :step="0.1"
                size="small"
                fluid />

              <!-- Width -->

              <label class="text-xs font-medium text-surface-600">Width</label>
              <InputNumber v-model="store.params.width" :min="256" :max="2048" :step="64" size="small" fluid />

              <!-- Height -->

              <label class="text-xs font-medium text-surface-600">Height</label>
              <InputNumber v-model="store.params.height" :min="256" :max="2048" :step="64" size="small" fluid />

              <!-- Seed -->
              <div class="flex flex-col items-start">
                <label class="text-xs font-medium text-surface-600">Seed</label>
                <p class="text-xs text-surface-400">-1 for random</p>
              </div>
              <InputNumber v-model="store.params.seed" :min="-1" size="small" fluid />

              <!-- Sampler -->
              <label class="text-xs font-medium text-surface-600">Sampler</label>
              <Select v-model="store.params.sampler" :options="samplerOptions" size="small" fluid />

              <!-- Scheduler -->
              <label class="text-xs font-medium text-surface-600">Scheduler</label>
              <Select v-model="store.params.scheduler" :options="schedulerOptions" size="small" fluid />
            </div>
          </Fieldset>

          <!-- LoRAs -->
          <Fieldset legend="LoRAs" :toggleable="true" :collapsed="true">
            <div class="flex flex-col gap-2">
              <div v-for="(lora, index) in store.params.loras" :key="index" class="flex flex-col gap-1 p-2 bg-surface-100 rounded">
                <div class="flex items-center gap-1">
                  <InputText v-model="lora.model_file" placeholder="LoRA model path" size="small" class="flex-1" />
                  <Button icon="pi pi-times" size="small" severity="danger" text rounded @click="store.removeLora(index)" />
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-xs text-surface-500 w-16">Strength</span>
                  <Slider v-model="lora.strength" :min="0" :max="2" :step="0.05" class="flex-1" />
                  <span class="text-xs text-surface-600 w-8 text-right">
                    {{ lora.strength.toFixed(2) }}
                  </span>
                </div>
              </div>
              <Button label="Add LoRA" icon="pi pi-plus" size="small" severity="secondary" outlined @click="store.addLora()" />
            </div>
          </Fieldset>
        </div>
      </template>
      <template #footer>
        <!-- Generate Button -->
        <Button
          v-if="!store.isGenerating"
          label="Generate"
          icon="pi pi-sparkles"
          class="w-full"
          :disabled="!store.engineReady || !store.params.prompt.trim()"
          @click="store.submitJob()" />
        <Button v-else label="Cancel" icon="pi pi-stop" class="w-full" severity="danger" @click="store.cancelJob()" />
      </template>
    </Panel>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import Button from 'primevue/button';
import Fieldset from 'primevue/fieldset';
import InputNumber from 'primevue/inputnumber';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import Slider from 'primevue/slider';
import Tag from 'primevue/tag';
import Textarea from 'primevue/textarea';
import ToggleSwitch from 'primevue/toggleswitch';
import { usePywebview } from '@/composables/usePywebview';
import { useInferenceStore } from '@/stores/inference';
import { Panel } from 'primevue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';

const { api, isReady } = usePywebview();
const store = useInferenceStore();

const showEngine = ref(false);
const showAdvanced = ref(false);

const samplerOptions = ['euler', 'euler_a', 'dpm++_2m', 'dpm++_2s', 'dpm++_sde', 'heun', 'lms'];
const schedulerOptions = ['normal', 'karras', 'exponential', 'sgm_uniform', 'simple', 'ddim_uniform'];

function onPresetChange(e: any) {
  const preset = store.presets.find((p) => p.id === e.value);
  if (preset) store.applyPreset(preset);
}

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      // Disable most formatting for simple text input
      heading: false,
      codeBlock: false,
      horizontalRule: false,
      blockquote: false,
      bulletList: false,
      orderedList: false,
      listItem: false,
      code: false,
      bold: false,
      italic: false,
      strike: false,
    }),
    Placeholder.configure({
      placeholder: 'Describe what you want to generate...',
    }),
  ],
  content: store.params.prompt,
  editorProps: {
    attributes: {
      class: 'focus:outline-none',
    },
    handleKeyDown: (_view, event) => {
      // Emit submit on Enter (without shift)
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        //emit('submit');
        return true;
      }
      return false;
    },
  },
  onUpdate: ({ editor }) => {
    const text = editor.getText();
    //emit('update:modelValue', text);
  },
});

// Watch for external value changes and sync to editor
watch(
  () => store.params.prompt,
  (newValue) => {
    if (editor.value && newValue !== editor.value.getText()) {
      editor.value.commands.setContent(newValue || '');
    }
  },
);

onMounted(async () => {
  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      store.setApi(api.value);
      await store.loadPresets();
      if (store.presets.length && !store.selectedPresetId) {
        store.applyPreset(store.presets[0]);
      }
      await store.startEngine();
    }
  }, 50);
});
</script>

<style scoped>
.menu-panel {
  height: 100%;
  display: flex !important;
  flex-direction: column;
}

.menu-panel :deep(.p-panel-header) {
  flex-shrink: 0;
}

.menu-panel :deep(.p-panel-content-container) {
  flex: 1;
  min-height: 0;
  display: flex !important;
  flex-direction: column;
}

.menu-panel :deep(.p-panel-content-wrapper) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.menu-panel :deep(.p-panel-content) {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.menu-panel :deep(.p-panel-footer) {
  flex-shrink: 0;
}
</style>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :draggable="false"
    :closable="true"
    header="Batch Generation"
    class="w-175 max-w-[95vw]">
    <Tabs :value="activeTab" @update:value="activeTab = $event">
      <TabList>
        <Tab value="prompts">
          <div class="flex items-center gap-1">
            <List :size="14" />
            Prompts
          </div>
        </Tab>
        <Tab value="data">
          <div class="flex items-center gap-1">
            <Table :size="14" />
            Data + Template
          </div>
        </Tab>
      </TabList>

      <TabPanels>
        <!-- Tab 1: One prompt per line -->
        <TabPanel value="prompts">
          <div class="flex flex-col gap-3 pt-3">
            <p class="text-xs text-surface-500">Enter one prompt per line. Each line generates a separate image using the current settings.</p>
            <Textarea
              v-model="promptsText"
              :rows="10"
              placeholder="A portrait of a warrior in golden armor&#10;A landscape of rolling hills at sunset&#10;A cyberpunk cityscape at night"
              class="w-full font-mono text-sm" />
            <div class="flex items-center justify-between">
              <span class="text-xs text-surface-400">{{ promptCount }} prompt{{ promptCount !== 1 ? 's' : '' }}</span>
            </div>
          </div>
        </TabPanel>

        <!-- Tab 2: CSV data + template -->
        <TabPanel value="data">
          <div class="flex flex-col gap-3 pt-3">
            <p class="text-xs text-surface-500">Paste CSV data below, then write a template using <code class="bg-surface-100 px-1 rounded text-xs" v-text="braces('column_name')"></code> placeholders.</p>

            <!-- CSV input -->
            <div>
              <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">CSV Data</label>
              <Textarea
                v-model="csvText"
                :rows="5"
                placeholder="subject,style,color&#10;cat,watercolor,blue&#10;dog,oil painting,red"
                class="w-full font-mono text-xs mt-1" />
            </div>

            <!-- Parse button + status -->
            <div class="flex items-center gap-2">
              <Button size="small" severity="secondary" :disabled="!csvText.trim()" @click="parseData">
                <Database :size="14" />
                Parse Data
              </Button>
              <span v-if="parsedColumns.length" class="text-xs text-green-600">
                {{ parsedRows.length }} rows, {{ parsedColumns.length }} columns
              </span>
              <span v-if="parseError" class="text-xs text-red-500">{{ parseError }}</span>
            </div>

            <!-- Column chips -->
            <div v-if="parsedColumns.length" class="flex flex-wrap gap-1">
              <button
                v-for="col in parsedColumns"
                :key="col"
                class="px-2 py-1 rounded-full bg-blue-50 text-blue-600 text-xs hover:bg-blue-100 transition-colors cursor-pointer"
                @click="insertVariable(col)"
                v-text="braces(col)">
              </button>
            </div>

            <!-- Template input -->
            <div>
              <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">Template</label>
              <Textarea
                ref="templateRef"
                v-model="templateText"
                :rows="3"
                :placeholder="templatePlaceholder"
                class="w-full font-mono text-sm mt-1" />
            </div>

            <!-- Preview -->
            <div v-if="renderedPrompts.length">
              <label class="text-xs font-medium text-surface-500 uppercase tracking-wide">Preview (first {{ Math.min(renderedPrompts.length, 5) }})</label>
              <div class="mt-1 flex flex-col gap-1">
                <div
                  v-for="(p, i) in renderedPrompts.slice(0, 5)"
                  :key="i"
                  class="text-xs font-mono px-2 py-1 rounded bg-surface-50"
                  :class="renderErrors[i] ? 'border border-red-200 text-red-500' : ''">
                  <span class="text-surface-400 mr-1">{{ i + 1 }}.</span>
                  {{ renderErrors[i] || p }}
                </div>
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button
          severity="primary"
          :disabled="!canSubmit"
          @click="submitBatch">
          <Sparkles :size="14" />
          Generate {{ finalPrompts.length }} Image{{ finalPrompts.length !== 1 ? 's' : '' }}
        </Button>
        <Button severity="secondary" variant="outlined" @click="visibleModel = false">
          Cancel
        </Button>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { usePywebview } from '@/composables/usePywebview';
import { useInferenceStore } from '@/stores/inference';

const visibleModel = defineModel<boolean>('visible', { required: true });
const emit = defineEmits<{ submitted: [] }>();

const { api } = usePywebview();
const store = useInferenceStore();

const activeTab = ref('prompts');

// ── Prompts tab ──
const promptsText = ref('');
const promptCount = computed(() => {
  const lines = promptsText.value.split('\n').filter(l => l.trim());
  return lines.length;
});

// ── Data tab ──
const csvText = ref('');
const templateText = ref('');
const templateRef = ref<InstanceType<typeof Textarea> | null>(null);
const parsedColumns = ref<string[]>([]);
const parsedRows = ref<Record<string, string>[]>([]);
const parseError = ref('');
const renderedPrompts = ref<string[]>([]);
const renderErrors = ref<Record<number, string>>({});

const templatePlaceholder = computed(() => {
  if (parsedColumns.value.length) {
    return 'A ' + braces(parsedColumns.value[0]) + ' in the style of ' + braces(parsedColumns.value[1] ?? 'style');
  }
  return 'A ' + braces('subject') + ' in the style of ' + braces('style');
});

async function parseData() {
  if (!api.value) return;
  parseError.value = '';
  parsedColumns.value = [];
  parsedRows.value = [];
  renderedPrompts.value = [];
  renderErrors.value = {};

  const res = await api.value.batch_parse_data({ content: csvText.value });
  if (res.status === 'success') {
    parsedColumns.value = res.columns ?? [];
    parsedRows.value = res.rows ?? [];
    // Auto-render if template already has content
    if (templateText.value.trim()) {
      await renderTemplate();
    }
  } else {
    parseError.value = res.message ?? 'Failed to parse data';
  }
}

async function renderTemplate() {
  if (!api.value || !parsedRows.value.length || !templateText.value.trim()) return;

  const res = await api.value.batch_render_template({
    template: templateText.value,
    rows: parsedRows.value,
  });
  if (res.status === 'success') {
    renderedPrompts.value = res.rendered ?? [];
    renderErrors.value = {};
    for (const err of res.errors ?? []) {
      renderErrors.value[err.row] = err.error;
    }
  }
}

function braces(name: string): string {
  return '{{' + name + '}}';
}

function insertVariable(col: string) {
  templateText.value += '{{' + col + '}}';
}

// Auto-render when template text changes and data is loaded
watch(templateText, () => {
  if (parsedRows.value.length && templateText.value.trim()) {
    renderTemplate();
  }
});

// ── Shared logic ──

const finalPrompts = computed(() => {
  if (activeTab.value === 'prompts') {
    return promptsText.value.split('\n').filter(l => l.trim());
  }
  return renderedPrompts.value.filter(p => p.trim());
});

const canSubmit = computed(() => {
  return finalPrompts.value.length > 0 && store.engineReady && !store.isGenerating;
});

function submitBatch() {
  if (!canSubmit.value) return;
  store.submitBatch(finalPrompts.value);
  visibleModel.value = false;
  emit('submitted');
}
</script>

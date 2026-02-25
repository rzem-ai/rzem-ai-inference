<template>
  <div class="flex h-full flex-col gap-2">
    <!-- Toolbar -->
    <Toolbar>
      <template #start>
        <Button @click="goBack" severity="secondary">
          <ArrowLeft :size="14" />
          <span>Back to Styles</span>
        </Button>
      </template>

      <template #center class="w-[50%]"> </template>

      <template #end>
        <Button :disabled="!canSave" @click="onSave">
          <Save :size="14" />
          <div>{{ isNew ? 'Create' : 'Save' }}</div>
        </Button>
      </template>
    </Toolbar>

    <!-- Editor content -->
    <div class="max-w-8xl mx-auto min-h-0 w-7xl flex-1 overflow-hidden">
      <div class="grid h-full grid-cols-2 gap-8">
        <!-- Center column: Core details -->
        <div class="flex min-h-0 max-w-160 flex-1 flex-col gap-4 overflow-y-auto">
          <!-- Thumbnail -->
          <div
            class="relative h-100 w-full cursor-pointer overflow-hidden rounded-lg border-2 transition-colors"
            :class="
              isThumbnailDragging
                ? 'border-blue-400 bg-blue-50'
                : thumbnailDataUrl
                  ? 'border-surface-200 hover:border-blue-300'
                  : 'border-surface-200 bg-surface-100 border-dashed hover:border-blue-300 hover:bg-blue-50/30'
            "
            @click="onBrowseThumbnail"
            @dragenter.prevent="onThumbnailDragEnter"
            @dragleave.prevent="onThumbnailDragLeave"
            @dragover.prevent
            @drop.prevent="onThumbnailDrop">
            <img v-if="thumbnailDataUrl" :src="thumbnailDataUrl" class="absolute inset-0 h-full w-full object-cover" />
            <div
              v-if="!thumbnailDataUrl || isThumbnailDragging"
              class="pointer-events-none absolute inset-0 flex items-center justify-center"
              :class="thumbnailDataUrl ? 'bg-black/40' : ''">
              <div class="text-center" :class="thumbnailDataUrl || isThumbnailDragging ? 'text-white' : 'text-surface-300'">
                <ImagePlus :size="32" class="mx-auto mb-1" />
                <p class="text-xs">{{ isThumbnailDragging ? 'Drop image here' : 'Drop or click to set thumbnail' }}</p>
              </div>
            </div>
          </div>

          <!-- Name -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Name</label>
            <InputText v-model="form.name" placeholder="Style name" />
          </div>

          <!-- Collection (category) -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Collection</label>
            <InputText v-model="form.category" placeholder="e.g. Photography, Illustration, Painting" />
          </div>

          <!-- Base Model -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Base Model</label>
            <Select
              v-model="selectedBundleId"
              :options="inferenceStore.bundlesByType"
              option-group-label="label"
              option-group-children="items"
              option-label="label"
              option-value="id"
              placeholder="Select base model"
              show-clear
              fluid />
          </div>

          <!-- Description -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Description</label>
            <TextEditor
              v-model="form.description"
              placeholder="Brief description of this style"
              editor-class=" bg-white border border-surface-300  text-surface-900 min-h-40 max-h-80 transition-colors " />
          </div>

          <!-- Tags -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Tags</label>
            <div class="flex flex-col gap-2">
              <!-- Tag input with suggestions -->
              <div class="relative">
                <InputText
                  v-model="tagInput"
                  placeholder="Type to add a tag..."
                  fluid
                  @focus="showTagSuggestions = true"
                  @keydown.enter.prevent="onTagEnter"
                  @keydown.escape="showTagSuggestions = false" />
                <ul
                  v-if="showTagSuggestions && tagSuggestions.length"
                  class="border-surface-200 absolute z-10 mt-1 max-h-32 w-full overflow-y-auto rounded-lg border bg-white shadow-lg">
                  <li
                    v-for="tag in tagSuggestions"
                    :key="tag.id"
                    class="text-surface-700 cursor-pointer px-3 py-1 text-sm transition-colors hover:bg-blue-50"
                    @mousedown.prevent="addExistingTag(tag)">
                    {{ tag.name }}
                  </li>
                </ul>
                <p v-if="tagInput.trim() && !tagSuggestions.length && showTagSuggestions" class="text-surface-400 mt-1 text-xs">
                  Press Enter to create "{{ tagInput.trim() }}"
                </p>
              </div>
              <div class="flex flex-wrap gap-2">
                <Chip
                  v-for="tag in allEditorTags"
                  :key="tag.id"
                  :label="tag.name"
                  @remove="removeTag(tag.id)"
                  removable
                  class="border-surface-300 rounded-xl border px-2 py-0" />
              </div>
            </div>
          </div>
        </div>

        <!-- Right column: Tags, LoRAs, and Examples -->
        <div class="flex min-h-0 max-w-160 flex-1 flex-col gap-6 overflow-y-auto">
          <!-- Positive prompt template -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">Prompt Template</label>
            <TextEditor
              v-model="form.promptTemplate"
              placeholder="In style of Moebius {prompt}"
              editor-class=" bg-white border border-surface-300 text-sm text-surface-900 min-h-15 max-h-25 focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-400 transition-colors cursor-text" />
            <p class="text-xs text-blue-600">Use {prompt} as a placeholder for the user's prompt</p>
          </div>

          <!-- LoRAs -->
          <div class="flex flex-col gap-0">
            <label class="text-surface-700 text-lg font-semibold">LoRA Models</label>
            <div class="border-surface-200 flex flex-col gap-2 rounded-lg border bg-white p-3">
              <!-- Associated LoRAs -->
              <div v-if="stylesStore.editorLoras.length" class="flex flex-col gap-2">
                <div v-for="lora in stylesStore.editorLoras" :key="lora.id" class="border-surface-200 flex items-center gap-2 rounded-lg border bg-white p-3">
                  <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-blue-50">
                    <Layers :size="16" class="text-blue-600" />
                  </div>
                  <div class="min-w-0 flex-1">
                    <p class="text-surface-800 truncate text-base font-semibold">{{ lora.lora_name }}</p>
                    <p class="text-surface-400 truncate text-sm">{{ lora.lora_path }}</p>
                  </div>
                  <div class="w-15">
                    <InputNumber
                      :model-value="lora.strength"
                      @update:model-value="(val) => onLoraStrengthChange(lora, val ?? 1)"
                      :min="0"
                      :max="2"
                      :step="0.05"
                      :min-fraction-digits="2"
                      :max-fraction-digits="2"
                      size="small"
                      input-class="text-right"
                      fluid />
                  </div>
                </div>
              </div>

              <!-- Pending LoRAs (not yet saved to style) -->
              <div v-if="pendingLoras.length" class="flex flex-col gap-2">
                <div v-for="lora in pendingLoras" :key="lora.id" class="flex items-center gap-2 rounded-lg border border-blue-200 bg-blue-50 p-3">
                  <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-blue-100">
                    <Layers :size="16" class="text-blue-600" />
                  </div>
                  <div class="min-w-0 flex-1">
                    <p class="text-surface-800 truncate text-base font-semibold">{{ lora.name }}</p>
                    <p class="text-surface-400 truncate text-sm">{{ lora.path }}</p>
                  </div>
                  <button class="text-surface-400 shrink-0 transition-colors hover:text-red-500" @click="removePendingLora(lora.id)">
                    <X :size="14" />
                  </button>
                </div>
              </div>

              <!-- Drop zone / browse button -->
              <div
                class="flex h-20 w-full cursor-pointer flex-col items-center justify-center rounded-lg border-[1.5px] border-dashed transition-colors"
                :class="
                  isDragging
                    ? 'border-blue-400 bg-blue-50 text-blue-500'
                    : 'border-surface-300 bg-surface-50 text-surface-400 hover:border-blue-300 hover:bg-blue-50/50 hover:text-blue-500'
                "
                @click="onBrowseLoras"
                @dragenter.prevent="onDragEnter"
                @dragleave.prevent="onDragLeave"
                @dragover.prevent
                @drop.prevent="onDrop">
                <div class="pointer-events-none text-center">
                  <Plus :size="20" class="mx-auto mb-1" />
                  <p class="text-xs font-normal">{{ isDragging ? 'Drop LoRA files here' : 'Drop or click to add LoRA files' }}</p>
                </div>
              </div>
            </div>
          </div>

          <!-- Example Prompts -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <label class="text-surface-700 text-lg font-semibold">Example Prompts ( {{ stylesStore.editorExamples.length }} )</label>
              <Button severity="secondary" variant="outlined" v-if="!isNew" @click="showAddExample = true">
                <Plus :size="13" />
                Add Example
              </Button>
            </div>

            <!-- Example cards -->
            <template v-if="stylesStore.editorExamples.length">
              <div v-for="example in parsedExamples" :key="example.id" class="border-surface-300 overflow-hidden rounded-lg border bg-white">
                <!-- Image (click for full resolution) -->
                <div v-if="example.imageDataUrl" class="h-32 cursor-pointer overflow-hidden" @click="onOpenExampleImage(example)">
                  <img :src="example.imageDataUrl" class="h-full w-full object-cover" />
                </div>
                <!-- Body -->
                <div class="flex flex-col gap-2 p-3">
                  <p class="text-surface-800 line-clamp-2 text-base font-medium">{{ example.prompt }}</p>
                  <div class="flex flex-wrap gap-1">
                    <!-- -->
                    <div v-if="example.seed != null" class="bg-surface-100 inline-flex items-center gap-1 rounded px-1 py-1">
                      <div class="text-surface-400 text-sm font-medium">Seed</div>
                      <div class="text-surface-600 font-mono text-base font-medium">{{ example.seed }}</div>
                    </div>

                    <!-- -->
                    <div v-if="example.width && example.height" class="bg-surface-100 inline-flex items-center gap-1 rounded px-1 py-1">
                      <div class="text-surface-400 text-sm font-medium">Size</div>
                      <div class="text-surface-600 font-mono text-base font-medium">{{ example.width }}×{{ example.height }}</div>
                    </div>

                    <!-- -->
                    <div v-if="example.steps != null" class="bg-surface-100 inline-flex items-center gap-1 rounded px-1 py-1">
                      <div class="text-surface-400 text-sm font-medium">Steps</div>
                      <div class="text-surface-600 font-mono text-base font-medium">{{ example.steps }}</div>
                    </div>

                    <!-- -->
                    <div v-if="example.cfg_scale != null" class="bg-surface-100 inline-flex items-center gap-1 rounded px-1 py-1">
                      <div class="text-surface-400 text-sm font-medium">CFG</div>
                      <div class="text-surface-600 font-mono text-base font-medium">{{ example.cfg_scale }}</div>
                    </div>
                  </div>

                  <!-- -->
                  <Button @click="useExample(example)" severity="primary" variant="outlined">
                    <Play :size="11" :fill="'currentColor'" />
                    Use This
                  </Button>
                </div>
              </div>
            </template>

            <p v-else class="text-surface-400 py-3 text-center text-xs">
              {{ isNew ? 'Save the style first to add examples' : 'No examples yet' }}
            </p>
          </div>

          <!-- Add Example Dialog -->
          <Dialog v-model:visible="showAddExample" header="Add Example Prompt" modal :style="{ width: '28rem' }">
            <div class="flex flex-col gap-3 pt-2">
              <div class="flex flex-col gap-1">
                <label class="text-surface-600 text-xs font-medium">Prompt</label>
                <textarea
                  v-model="exampleForm.prompt"
                  rows="3"
                  placeholder="Example prompt text"
                  class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 resize-none rounded-lg border px-3 py-2 text-sm transition-colors outline-none focus:border-blue-400" />
              </div>
              <div class="flex flex-col gap-1">
                <label class="text-surface-600 text-xs font-medium">Image (optional)</label>
                <button
                  class="bg-surface-50 border-surface-200 text-surface-600 flex h-9 items-center gap-2 rounded-lg border px-3 text-xs transition-colors hover:border-blue-400"
                  @click="onBrowseExampleImage">
                  <ImagePlus :size="14" />
                  {{ exampleForm.imagePath ? exampleForm.imagePath.split('/').pop() : 'Browse image...' }}
                </button>
              </div>
              <div class="grid grid-cols-2 gap-2">
                <div class="flex flex-col gap-1">
                  <label class="text-surface-600 text-xs font-medium">Seed</label>
                  <input
                    v-model.number="exampleForm.seed"
                    type="number"
                    placeholder="Random"
                    class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 h-8 rounded-lg border px-3 text-xs outline-none focus:border-blue-400" />
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-surface-600 text-xs font-medium">Steps</label>
                  <input
                    v-model.number="exampleForm.steps"
                    type="number"
                    placeholder="—"
                    class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 h-8 rounded-lg border px-3 text-xs outline-none focus:border-blue-400" />
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-surface-600 text-xs font-medium">Width</label>
                  <input
                    v-model.number="exampleForm.width"
                    type="number"
                    placeholder="—"
                    class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 h-8 rounded-lg border px-3 text-xs outline-none focus:border-blue-400" />
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-surface-600 text-xs font-medium">Height</label>
                  <input
                    v-model.number="exampleForm.height"
                    type="number"
                    placeholder="—"
                    class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 h-8 rounded-lg border px-3 text-xs outline-none focus:border-blue-400" />
                </div>
              </div>
              <div class="flex flex-col gap-1">
                <label class="text-surface-600 text-xs font-medium">CFG Scale</label>
                <input
                  v-model.number="exampleForm.cfgScale"
                  type="number"
                  step="0.1"
                  placeholder="—"
                  class="bg-surface-50 border-surface-200 text-surface-700 placeholder-surface-400 h-8 rounded-lg border px-3 text-xs outline-none focus:border-blue-400" />
              </div>
            </div>
            <template #footer>
              <div class="flex justify-end gap-2">
                <Button severity="secondary" @click="showAddExample = false">Cancel</Button>
                <Button :disabled="!exampleForm.prompt.trim()" @click="onAddExample">Add Example</Button>
              </div>
            </template>
          </Dialog>
        </div>
      </div>
    </div>

    <!-- Full-resolution image overlay -->
    <Teleport to="body">
      <Transition name="overlay-fade">
        <div
          v-if="overlayImageUrl || overlayLoading"
          class="fixed inset-0 z-50 flex items-center justify-center"
          @click="closeOverlay"
          @keydown.escape="closeOverlay"
          tabindex="0"
          ref="overlayRef">
          <div class="absolute inset-0 bg-black/90" />
          <button
            class="absolute top-4 right-4 z-10 rounded-full bg-white/10 p-2 text-white/70 backdrop-blur-sm transition-colors hover:bg-white/20 hover:text-white"
            @click="closeOverlay">
            <X :size="20" />
          </button>
          <img
            v-if="overlayImageUrl"
            :src="overlayImageUrl"
            class="relative z-1 max-h-[90vh] max-w-[90vw] rounded-lg object-contain select-none"
            draggable="false"
            @click.stop />
          <div v-else class="relative z-1 h-8 w-8 animate-spin rounded-full border-2 border-white/40 border-t-white" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useStylesStore } from '@/stores/styles';
import { useInferenceStore } from '@/stores/inference';
import { usePywebview } from '@/composables/usePywebview';
import type { LoRA, Tag } from '@/types/inference';
import TextEditor from '@/components/TextEditor.vue';

const LORA_EXTENSIONS = ['.safetensors', '.ckpt', '.pt'];
const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.webp', '.bmp'];

const { api } = usePywebview();
const route = useRoute();
const router = useRouter();
const stylesStore = useStylesStore();
const inferenceStore = useInferenceStore();

const isNew = computed(() => route.name === 'styles-new');
const styleId = computed(() => route.params.id as string | undefined);

const form = ref({
  name: '',
  description: '',
  promptTemplate: '',
  negativePrompt: '',
  category: '',
});

const selectedBundleId = ref<string | null>(null);
const pendingLoras = ref<LoRA[]>([]);
const pendingTags = ref<Tag[]>([]);
const isDragging = ref(false);
let dragCounter = 0;

// Thumbnail
const thumbnailPath = ref<string | null>(null);
const thumbnailDataUrl = ref<string | null>(null);
const isThumbnailDragging = ref(false);
let thumbDragCounter = 0;

// Tag input
const tagInput = ref('');
const showTagSuggestions = ref(false);

const allEditorTags = computed(() => [...stylesStore.editorTags, ...pendingTags.value]);

const assignedTagIds = computed(() => new Set(allEditorTags.value.map((t) => t.id)));

const tagSuggestions = computed(() => {
  const q = tagInput.value.trim().toLowerCase();
  return stylesStore.tags.filter((t) => !assignedTagIds.value.has(t.id)).filter((t) => !q || t.name.toLowerCase().includes(q));
});

const canSave = computed(() => form.value.name.trim() && form.value.promptTemplate.trim());

// Load existing style when editing
onMounted(async () => {
  stylesStore.loadTags();
  if (!inferenceStore.bundles.length) inferenceStore.loadBundles();
  if (!isNew.value && styleId.value) {
    await stylesStore.loadStyleForEditor(styleId.value);
  } else {
    stylesStore.clearEditor();
  }
});

// Populate form when editor style loads, reset when cleared
watch(
  () => stylesStore.editorStyle,
  async (style) => {
    if (style) {
      form.value = {
        name: style.name,
        description: style.description ?? '',
        promptTemplate: style.prompt_template,
        negativePrompt: style.negative_prompt ?? '',
        category: style.category ?? '',
      };
      selectedBundleId.value = style.bundle_id ?? null;
      if (style.thumbnail_path) {
        thumbnailPath.value = style.thumbnail_path;
        await loadThumbnail(style.thumbnail_path);
      }
    } else {
      form.value = { name: '', description: '', promptTemplate: '', negativePrompt: '', category: '' };
      selectedBundleId.value = null;
      thumbnailPath.value = null;
      thumbnailDataUrl.value = null;
      pendingLoras.value = [];
      pendingTags.value = [];
    }
  },
  { immediate: true },
);

function goBack() {
  router.push({ name: 'styles' });
}

async function onSave() {
  if (!canSave.value) return;

  if (isNew.value) {
    const id = crypto.randomUUID();
    const res = await stylesStore.createStyle({
      id,
      name: form.value.name.trim(),
      promptTemplate: form.value.promptTemplate.trim(),
      description: form.value.description.trim() || undefined,
      negativePrompt: form.value.negativePrompt.trim() || undefined,
      category: form.value.category.trim() || undefined,
      thumbnailPath: thumbnailPath.value || undefined,
      bundleId: selectedBundleId.value || undefined,
    });
    if (res?.status === 'success') {
      if (pendingLoras.value.length) {
        const loras = pendingLoras.value.map((l, i) => ({
          lora_id: l.id,
          strength: l.strength,
          priority: i,
        }));
        await stylesStore.saveStyleLoras(id, loras);
      }
      if (pendingTags.value.length) {
        await stylesStore.saveStyleTags(
          id,
          pendingTags.value.map((t) => t.id),
        );
      }
      router.push({ name: 'styles' });
    }
  } else if (styleId.value) {
    await stylesStore.updateStyle(styleId.value, {
      name: form.value.name.trim(),
      prompt_template: form.value.promptTemplate.trim(),
      description: form.value.description.trim() || null,
      negative_prompt: form.value.negativePrompt.trim() || null,
      category: form.value.category.trim() || null,
      thumbnail_path: thumbnailPath.value || null,
      bundle_id: selectedBundleId.value || null,
    });
    router.push({ name: 'styles' });
  }
}

function removeTag(tagId: number) {
  // Remove from pending if it's there
  const wasPending = pendingTags.value.some((t) => t.id === tagId);
  if (wasPending) {
    pendingTags.value = pendingTags.value.filter((t) => t.id !== tagId);
    return;
  }
  if (!styleId.value) return;
  const remaining = stylesStore.editorTags.filter((t) => t.id !== tagId).map((t) => t.id);
  stylesStore.saveStyleTags(styleId.value, remaining);
}

async function addTag(tag: Tag) {
  if (assignedTagIds.value.has(tag.id)) return;

  if (styleId.value) {
    const currentIds = stylesStore.editorTags.map((t) => t.id);
    await stylesStore.saveStyleTags(styleId.value, [...currentIds, tag.id]);
  } else {
    pendingTags.value.push(tag);
  }
  tagInput.value = '';
  showTagSuggestions.value = false;
}

function addExistingTag(tag: Tag) {
  addTag(tag);
}

async function onTagEnter() {
  const name = tagInput.value.trim();
  if (!name) return;

  // Check if an exact match exists in suggestions
  const existing = stylesStore.tags.find((t) => t.name.toLowerCase() === name.toLowerCase());
  if (existing && !assignedTagIds.value.has(existing.id)) {
    await addTag(existing);
    return;
  }

  // Create a new tag
  const tag = await stylesStore.createTag(name);
  if (tag) {
    await addTag(tag);
  }
}

async function onLoraStrengthChange(lora: { lora_id: string; strength: number; priority: number }, val: number) {
  if (!styleId.value) return;
  const all = stylesStore.editorLoras.map((l) => ({
    lora_id: l.lora_id,
    strength: l.lora_id === lora.lora_id ? val : l.strength,
    priority: l.priority,
  }));
  await stylesStore.saveStyleLoras(styleId.value, all);
}

function removePendingLora(loraId: string) {
  pendingLoras.value = pendingLoras.value.filter((l) => l.id !== loraId);
}

async function addLoras(loras: LoRA[]) {
  if (!loras.length) return;

  if (styleId.value) {
    const existing = stylesStore.editorLoras.map((l) => ({
      lora_id: l.lora_id,
      strength: l.strength,
      priority: l.priority,
    }));
    const additions = loras.map((l, i) => ({
      lora_id: l.id,
      strength: l.strength,
      priority: existing.length + i,
    }));
    await stylesStore.saveStyleLoras(styleId.value, [...existing, ...additions]);
  } else {
    pendingLoras.value.push(...loras);
  }
}

async function onBrowseLoras() {
  const newLoras = await stylesStore.browseLoras();
  await addLoras(newLoras);
}

// ── Drag and drop ──

function onDragEnter() {
  dragCounter++;
  isDragging.value = true;
}

function onDragLeave() {
  dragCounter--;
  if (dragCounter <= 0) {
    dragCounter = 0;
    isDragging.value = false;
  }
}

function extractPathsFromDrop(e: DragEvent, extensions: string[]): string[] {
  const uriList = e.dataTransfer?.getData('text/uri-list') ?? '';
  return uriList
    .split(/\r?\n/)
    .filter((line) => line && !line.startsWith('#'))
    .map((uri) => {
      try {
        return decodeURIComponent(new URL(uri).pathname);
      } catch {
        return '';
      }
    })
    .filter((p) => p && extensions.some((ext) => p.endsWith(ext)));
}

async function onDrop(e: DragEvent) {
  isDragging.value = false;
  dragCounter = 0;

  const paths = extractPathsFromDrop(e, LORA_EXTENSIONS);
  if (!paths.length) return;

  const newLoras = await stylesStore.registerLoraPaths(paths);
  await addLoras(newLoras);
}

// ── Thumbnail ──

async function loadThumbnail(path: string) {
  const res = await api.value.get_image_base64({ image_path: path });
  if (res.status === 'success' && res.data_url) {
    thumbnailDataUrl.value = res.data_url;
  }
}

async function setThumbnail(path: string) {
  thumbnailPath.value = path;
  await loadThumbnail(path);
}

async function onBrowseThumbnail() {
  const res = await api.value.browse_image_file({ style_id: styleId.value });
  if (res.status === 'success' && res.path) {
    await setThumbnail(res.thumbnail_path || res.path);
  }
}

function onThumbnailDragEnter() {
  thumbDragCounter++;
  isThumbnailDragging.value = true;
}

function onThumbnailDragLeave() {
  thumbDragCounter--;
  if (thumbDragCounter <= 0) {
    thumbDragCounter = 0;
    isThumbnailDragging.value = false;
  }
}

async function onThumbnailDrop(e: DragEvent) {
  isThumbnailDragging.value = false;
  thumbDragCounter = 0;

  const paths = extractPathsFromDrop(e, IMAGE_EXTENSIONS);
  if (paths.length) {
    const res = await api.value.import_style_image({ source_path: paths[0], style_id: styleId.value });
    if (res.status === 'success' && res.thumbnail_path) {
      await setThumbnail(res.thumbnail_path);
    }
  }
}

// ── Examples ──

interface ParsedExample {
  id: string;
  prompt: string;
  image_path?: string | null;
  thumbnail_path?: string | null;
  imageDataUrl?: string | null;
  seed?: number | null;
  width?: number | null;
  height?: number | null;
  steps?: number | null;
  cfg_scale?: number | null;
}

const showAddExample = ref(false);
const overlayRef = ref<HTMLElement>();
const overlayImageUrl = ref<string | null>(null);
const overlayLoading = ref(false);
const exampleForm = ref({
  prompt: '',
  imagePath: '',
  seed: undefined as number | undefined,
  width: undefined as number | undefined,
  height: undefined as number | undefined,
  steps: undefined as number | undefined,
  cfgScale: undefined as number | undefined,
});

const exampleImageCache = ref<Record<string, string>>({});

const parsedExamples = computed<ParsedExample[]>(() => {
  return stylesStore.editorExamples.map((ex) => {
    try {
      const content = JSON.parse(ex.content);
      const thumbKey = content.thumbnail_path || content.image_path;
      return {
        id: ex.id,
        prompt: content.prompt ?? '',
        image_path: content.image_path,
        thumbnail_path: content.thumbnail_path,
        imageDataUrl: thumbKey ? exampleImageCache.value[thumbKey] : null,
        seed: content.seed,
        width: content.width,
        height: content.height,
        steps: content.steps,
        cfg_scale: content.cfg_scale,
      };
    } catch {
      return { id: ex.id, prompt: '(invalid example)' };
    }
  });
});

// Load thumbnail images for example cards
watch(
  () => stylesStore.editorExamples,
  async (examples) => {
    for (const ex of examples) {
      try {
        const content = JSON.parse(ex.content);
        // Prefer thumbnail_path for cards; fall back to image_path
        const path = content.thumbnail_path || content.image_path;
        if (path && !exampleImageCache.value[path]) {
          if (path.startsWith('http://') || path.startsWith('https://')) {
            exampleImageCache.value[path] = path;
          } else {
            const res = await api.value.get_image_base64({ image_path: path });
            if (res.status === 'success' && res.data_url) {
              exampleImageCache.value[path] = res.data_url;
            }
          }
        }
      } catch {
        /* skip */
      }
    }
  },
  { immediate: true },
);

function useExample(example: ParsedExample) {
  // Apply this style to the Create page
  inferenceStore.applyStyle(styleId.value!, form.value.promptTemplate, form.value.negativePrompt || null, stylesStore.editorLoras, selectedBundleId.value);

  // Apply the example's generation parameters
  const params: Record<string, any> = { prompt: example.prompt };
  if (example.seed != null) params.seed = example.seed;
  if (example.width != null) params.width = example.width;
  if (example.height != null) params.height = example.height;
  if (example.steps != null) params.steps = example.steps;
  if (example.cfg_scale != null) params.cfg_scale = example.cfg_scale;
  inferenceStore.applyParams(params);

  router.push({ name: 'create' });
}

async function onOpenExampleImage(example: ParsedExample) {
  const fullPath = example.image_path;
  if (!fullPath) return;

  overlayLoading.value = true;
  overlayImageUrl.value = null;
  await nextTick();
  overlayRef.value?.focus();

  if (fullPath.startsWith('http://') || fullPath.startsWith('https://')) {
    overlayImageUrl.value = fullPath;
    overlayLoading.value = false;
    return;
  }

  const res = await api.value.get_image_base64({ image_path: fullPath });
  if (res.status === 'success' && res.data_url) {
    overlayImageUrl.value = res.data_url;
  }
  overlayLoading.value = false;
}

function closeOverlay() {
  overlayImageUrl.value = null;
  overlayLoading.value = false;
}

async function onBrowseExampleImage() {
  const res = await api.value.browse_image_file({ style_id: styleId.value });
  if (res.status === 'success' && res.path) {
    exampleForm.value.imagePath = res.path;
  }
}

async function onAddExample() {
  if (!styleId.value || !exampleForm.value.prompt.trim()) return;

  await stylesStore.addExample(styleId.value, {
    prompt: exampleForm.value.prompt.trim(),
    imagePath: exampleForm.value.imagePath || undefined,
    seed: exampleForm.value.seed,
    width: exampleForm.value.width,
    height: exampleForm.value.height,
    steps: exampleForm.value.steps,
    cfgScale: exampleForm.value.cfgScale,
  });

  // Reset form
  exampleForm.value = {
    prompt: '',
    imagePath: '',
    seed: undefined,
    width: undefined,
    height: undefined,
    steps: undefined,
    cfgScale: undefined,
  };
  showAddExample.value = false;
}
</script>

<style scoped>
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.2s ease;
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}
</style>

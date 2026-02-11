<template>
  <div class="h-full flex flex-col gap-2">
    <!-- Toolbar -->
    <div class="flex items-center gap-1.5 px-3 h-10 bg-white rounded-xl border border-slate-200 shrink-0">
      <button
        class="flex items-center gap-1.5 px-2.5 h-7 rounded-md bg-slate-50 text-xs font-medium text-slate-600 hover:bg-slate-100 transition-colors"
        @click="goBack">
        <ArrowLeft :size="14" />
        <span>Back to Styles</span>
      </button>

      <div class="flex-1" />

      <button
        class="flex items-center gap-1.5 px-2.5 h-7 rounded-md bg-blue-500 text-xs font-medium text-white hover:bg-blue-600 transition-colors disabled:opacity-50"
        :disabled="!canSave"
        @click="onSave">
        <Save :size="14" />
        <span>{{ isNew ? 'Create' : 'Save' }}</span>
      </button>
    </div>

    <!-- Editor content -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <div class="max-w-5xl mx-auto p-4 grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Left column: Core details -->
        <div class="flex flex-col gap-4">
          <!-- Thumbnail -->
          <div
            class="relative aspect-video w-full rounded-lg overflow-hidden border-2 transition-colors cursor-pointer"
            :class="isThumbnailDragging
              ? 'border-blue-400 bg-blue-50'
              : thumbnailDataUrl
                ? 'border-slate-200 hover:border-blue-300'
                : 'border-dashed border-slate-200 bg-slate-100 hover:border-blue-300 hover:bg-blue-50/30'"
            @click="onBrowseThumbnail"
            @dragenter.prevent="onThumbnailDragEnter"
            @dragleave.prevent="onThumbnailDragLeave"
            @dragover.prevent
            @drop.prevent="onThumbnailDrop">
            <img
              v-if="thumbnailDataUrl"
              :src="thumbnailDataUrl"
              class="absolute inset-0 w-full h-full object-cover" />
            <div
              v-if="!thumbnailDataUrl || isThumbnailDragging"
              class="absolute inset-0 flex items-center justify-center pointer-events-none"
              :class="thumbnailDataUrl ? 'bg-black/40' : ''">
              <div class="text-center" :class="thumbnailDataUrl || isThumbnailDragging ? 'text-white' : 'text-slate-300'">
                <ImagePlus :size="32" class="mx-auto mb-1" />
                <p class="text-xs">{{ isThumbnailDragging ? 'Drop image here' : 'Drop or click to set thumbnail' }}</p>
              </div>
            </div>
          </div>

          <!-- Name -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-slate-600">Name</label>
            <input
              v-model="form.name"
              type="text"
              placeholder="Style name"
              class="px-3 h-9 rounded-lg bg-slate-50 border border-slate-200 text-sm text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors" />
          </div>

          <!-- Collection (category) -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-slate-600">Collection</label>
            <input
              v-model="form.category"
              type="text"
              placeholder="e.g. Photography, Illustration, Painting"
              class="px-3 h-9 rounded-lg bg-slate-50 border border-slate-200 text-sm text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors" />
          </div>

          <!-- Description -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-slate-600">Description</label>
            <textarea
              v-model="form.description"
              rows="2"
              placeholder="Brief description of this style"
              class="px-3 py-2 rounded-lg bg-slate-50 border border-slate-200 text-sm text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors resize-none" />
          </div>

          <!-- Positive prompt template -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-slate-600">Prompt Template</label>
            <textarea
              v-model="form.promptTemplate"
              rows="4"
              placeholder="cinematic film still, {prompt}, 35mm photograph, film grain"
              class="px-3 py-2 rounded-lg bg-slate-50 border border-slate-200 text-sm text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors resize-none font-mono" />
            <p class="text-[10px] text-slate-400">Use {prompt} as a placeholder for the user's prompt</p>
          </div>

          <!-- Negative prompt -->
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-slate-600">Negative Prompt</label>
            <textarea
              v-model="form.negativePrompt"
              rows="3"
              placeholder="Things to avoid in generated images"
              class="px-3 py-2 rounded-lg bg-slate-50 border border-slate-200 text-sm text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors resize-none font-mono" />
          </div>
        </div>

        <!-- Right column: LoRAs and Tags -->
        <div class="flex flex-col gap-4">
          <!-- Tags -->
          <div class="flex flex-col gap-2 p-3 rounded-lg bg-white border border-slate-200">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-700">Tags</span>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="tag in allEditorTags"
                :key="tag.id"
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-slate-100 text-slate-600">
                {{ tag.name }}
                <button class="hover:text-red-500 transition-colors" @click="removeTag(tag.id)">
                  <X :size="10" />
                </button>
              </span>
            </div>

            <!-- Tag input with suggestions -->
            <div class="relative">
              <input
                v-model="tagInput"
                type="text"
                placeholder="Type to add a tag..."
                class="w-full px-3 h-8 rounded-lg bg-slate-50 border border-slate-200 text-xs text-slate-700 placeholder-slate-400 outline-none focus:border-blue-400 transition-colors"
                @focus="showTagSuggestions = true"
                @keydown.enter.prevent="onTagEnter"
                @keydown.escape="showTagSuggestions = false" />
              <ul
                v-if="showTagSuggestions && tagSuggestions.length"
                class="absolute z-10 w-full mt-1 bg-white border border-slate-200 rounded-lg shadow-lg max-h-32 overflow-y-auto">
                <li
                  v-for="tag in tagSuggestions"
                  :key="tag.id"
                  class="px-3 py-1.5 text-xs text-slate-700 hover:bg-blue-50 cursor-pointer transition-colors"
                  @mousedown.prevent="addExistingTag(tag)">
                  {{ tag.name }}
                </li>
              </ul>
              <p v-if="tagInput.trim() && !tagSuggestions.length && showTagSuggestions" class="mt-1 text-[10px] text-slate-400">
                Press Enter to create "{{ tagInput.trim() }}"
              </p>
            </div>
          </div>

          <!-- LoRAs -->
          <div class="flex flex-col gap-2 p-3 rounded-lg bg-white border border-slate-200">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-700">LoRA Models</span>
            </div>

            <!-- Associated LoRAs -->
            <div v-if="stylesStore.editorLoras.length" class="flex flex-col gap-2">
              <div
                v-for="lora in stylesStore.editorLoras"
                :key="lora.id"
                class="flex items-center gap-2 p-2 rounded-lg bg-slate-50 border border-slate-100">
                <Layers :size="14" class="text-slate-400 shrink-0" />
                <div class="flex-1 min-w-0">
                  <p class="text-xs font-medium text-slate-700 truncate">{{ lora.lora_name }}</p>
                  <p class="text-[10px] text-slate-400 truncate">{{ lora.lora_path }}</p>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input
                    type="range"
                    :value="lora.strength"
                    min="0"
                    max="2"
                    step="0.05"
                    class="w-20 h-1 accent-blue-500" />
                  <span class="text-[10px] text-slate-500 w-7 text-right">{{ lora.strength.toFixed(2) }}</span>
                </div>
              </div>
            </div>

            <!-- Pending LoRAs (not yet saved to style) -->
            <div v-if="pendingLoras.length" class="flex flex-col gap-2">
              <div
                v-for="lora in pendingLoras"
                :key="lora.id"
                class="flex items-center gap-2 p-2 rounded-lg bg-blue-50 border border-blue-100">
                <Layers :size="14" class="text-blue-400 shrink-0" />
                <div class="flex-1 min-w-0">
                  <p class="text-xs font-medium text-slate-700 truncate">{{ lora.name }}</p>
                  <p class="text-[10px] text-slate-400 truncate">{{ lora.path }}</p>
                </div>
                <button class="text-slate-400 hover:text-red-500 transition-colors shrink-0" @click="removePendingLora(lora.id)">
                  <X :size="14" />
                </button>
              </div>
            </div>

            <!-- Drop zone / browse button -->
            <div
              class="flex items-center justify-center w-full h-20 rounded-lg border-2 border-dashed transition-colors cursor-pointer"
              :class="isDragging
                ? 'border-blue-400 bg-blue-50 text-blue-500'
                : 'border-slate-200 text-slate-400 hover:border-blue-300 hover:text-blue-400 hover:bg-blue-50/50'"
              @click="onBrowseLoras"
              @dragenter.prevent="onDragEnter"
              @dragleave.prevent="onDragLeave"
              @dragover.prevent
              @drop.prevent="onDrop">
              <div class="text-center pointer-events-none">
                <Plus :size="20" class="mx-auto mb-1" />
                <p class="text-xs">{{ isDragging ? 'Drop LoRA files here' : 'Drop or click to add LoRA files' }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ArrowLeft, Save, X, Layers, Plus, ImagePlus } from 'lucide-vue-next';
import { useStylesStore } from '@/stores/styles';
import { usePywebview } from '@/composables/usePywebview';
import type { LoRA, Tag } from '@/types/inference';

const LORA_EXTENSIONS = ['.safetensors', '.ckpt', '.pt'];
const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.webp', '.bmp'];

const { api } = usePywebview();
const route = useRoute();
const router = useRouter();
const stylesStore = useStylesStore();

const isNew = computed(() => route.name === 'styles-new');
const styleId = computed(() => route.params.id as string | undefined);

const form = ref({
  name: '',
  description: '',
  promptTemplate: '',
  negativePrompt: '',
  category: '',
});

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

const assignedTagIds = computed(() => new Set(allEditorTags.value.map(t => t.id)));

const tagSuggestions = computed(() => {
  const q = tagInput.value.trim().toLowerCase();
  return stylesStore.tags
    .filter(t => !assignedTagIds.value.has(t.id))
    .filter(t => !q || t.name.toLowerCase().includes(q));
});

const canSave = computed(() => form.value.name.trim() && form.value.promptTemplate.trim());

// Load existing style when editing
onMounted(async () => {
  stylesStore.loadTags();
  if (!isNew.value && styleId.value) {
    await stylesStore.loadStyleForEditor(styleId.value);
  } else {
    stylesStore.clearEditor();
  }
});

// Populate form when editor style loads
watch(() => stylesStore.editorStyle, async (style) => {
  if (style) {
    form.value = {
      name: style.name,
      description: style.description ?? '',
      promptTemplate: style.prompt_template,
      negativePrompt: style.negative_prompt ?? '',
      category: style.category ?? '',
    };
    if (style.thumbnail_path) {
      thumbnailPath.value = style.thumbnail_path;
      await loadThumbnail(style.thumbnail_path);
    }
  }
}, { immediate: true });

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
        await stylesStore.saveStyleTags(id, pendingTags.value.map(t => t.id));
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
    });
    router.push({ name: 'styles' });
  }
}

function removeTag(tagId: number) {
  // Remove from pending if it's there
  const wasPending = pendingTags.value.some(t => t.id === tagId);
  if (wasPending) {
    pendingTags.value = pendingTags.value.filter(t => t.id !== tagId);
    return;
  }
  if (!styleId.value) return;
  const remaining = stylesStore.editorTags.filter(t => t.id !== tagId).map(t => t.id);
  stylesStore.saveStyleTags(styleId.value, remaining);
}

async function addTag(tag: Tag) {
  if (assignedTagIds.value.has(tag.id)) return;

  if (styleId.value) {
    const currentIds = stylesStore.editorTags.map(t => t.id);
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
  const existing = stylesStore.tags.find(t => t.name.toLowerCase() === name.toLowerCase());
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

function removePendingLora(loraId: string) {
  pendingLoras.value = pendingLoras.value.filter(l => l.id !== loraId);
}

async function addLoras(loras: LoRA[]) {
  if (!loras.length) return;

  if (styleId.value) {
    const existing = stylesStore.editorLoras.map(l => ({
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
    .filter(line => line && !line.startsWith('#'))
    .map(uri => {
      try { return decodeURIComponent(new URL(uri).pathname); }
      catch { return ''; }
    })
    .filter(p => p && extensions.some(ext => p.endsWith(ext)));
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
  const res = await api.value.browse_image_file();
  if (res.status === 'success' && res.path) {
    await setThumbnail(res.path);
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
    await setThumbnail(paths[0]);
  }
}
</script>

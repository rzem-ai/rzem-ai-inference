<template>
	<div class="flex h-full flex-col gap-2">
		<!-- Toolbar -->
		<Toolbar>
			<template #start>
				<Button severity="secondary" @click="router.push({ name: 'styles' })">
					<ArrowLeft :size="14" />
					<span>Back to Styles</span>
				</Button>
				<Divider layout="vertical" />
				<span class="text-lg font-semibold text-slate-800">Style Builder</span>
			</template>
			<template #center />
			<template #end>
				<Button severity="secondary" @click="showThumbnailDialog = true">
					<ImageIcon :size="14" />
					<span>Generate Thumbnails</span>
				</Button>
			</template>
		</Toolbar>

		<!-- Body -->
		<div class="flex min-h-0 flex-1 gap-0">
			<!-- Left: Filter Browser -->
			<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden p-3">
				<!-- Search -->
				<InputGroup>
					<InputGroupAddon>
						<Search :size="14" class="text-surface-400" />
					</InputGroupAddon>
					<InputText v-model="searchQuery" placeholder="Search filters..." fluid />
				</InputGroup>

				<!-- Category Tabs -->
				<div class="flex flex-wrap gap-1">
					<Button
						v-for="cat in categories"
						:key="cat"
						size="small"
						:severity="activeCategory === cat ? 'primary' : 'secondary'"
						class="font-medium transition-colors"
						@click="onCategoryClick(cat)">
						{{ cat }}
					</Button>
				</div>

				<!-- Subcategory Tabs -->
				<div class="flex flex-wrap gap-3">
					<Button
						v-for="sub in subcategories"
						:key="sub"
						size="small"
						variant="outlined"
						:severity="activeSubcategory === sub ? 'primary' : 'secondary'"
						class="border-t-0 border-r-0 border-l-0 font-medium transition-colors"
						:class="activeSubcategory === sub ? 'border-b-2 pb-1' : 'pb-1'"
						@click="activeSubcategory = activeSubcategory === sub ? '' : sub">
						{{ sub }}
					</Button>
				</div>

				<!-- Filter Grid -->
				<div class="min-h-0 flex-1 overflow-y-auto">
					<div class="grid grid-cols-5 gap-2">
						<div
							v-for="filter in visibleFilters"
							:key="filter.prompt"
							class="cursor-pointer overflow-hidden rounded-lg border-2 transition-all hover:shadow-sm"
							:class="isSelected(filter) ? 'border-blue-500 ring-1 ring-blue-200' : 'border-slate-200 hover:border-slate-300'"
							@click="stylesStore.toggleFilter(filter)">
							<!-- Thumbnail image or gradient fallback -->
							<div
								v-if="!thumbnailUrls[filter.image]"
								class="flex h-24 items-center justify-center text-xs font-medium text-white [text-shadow:0_1px_3px_rgba(0,0,0,0.6)]"
								:style="{ background: `linear-gradient(135deg, ${filter.colours[0]}, ${filter.colours[1]})` }">
								{{ filter.label }}
							</div>
							<div v-else class="relative h-24">
								<img :src="thumbnailUrls[filter.image]" class="h-full w-full object-cover" :alt="filter.label" />
								<span class="absolute bottom-0 left-0 w-full bg-black/40 px-2 py-1 text-xs font-medium text-white">
									{{ filter.label }}
								</span>
							</div>
							<!-- Label row -->
							<div class="flex items-center justify-between bg-white px-2 py-1">
								<span class="truncate text-base font-medium text-slate-700">{{ filter.label }}</span>
								<Check v-if="isSelected(filter)" :size="12" class="shrink-0 text-blue-500" />
							</div>
						</div>
					</div>

					<div v-if="!visibleFilters.length" class="flex h-40 items-center justify-center text-sm text-slate-400"> No filters match your search </div>
				</div>
			</div>

			<!-- Right: Selection Panel -->
			<div class="flex w-96 shrink-0 flex-col gap-4 border-l border-slate-200 bg-white p-4">
				<!-- Selected Filters -->
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<span class="text-lg font-semibold text-slate-800"> Selected Filters ({{ stylesStore.builderSelected.length }}) </span>
						<Button v-if="stylesStore.builderSelected.length" severity="secondary" size="small" class="hover:text-red-500" @click="stylesStore.clearBuilder()">
							Clear all
						</Button>
					</div>

					<div class="flex flex-wrap gap-1">
						<Chip
							v-for="filter in stylesStore.builderSelected"
							:key="filter.prompt"
							:label="filter.label"
							removable
							@remove="stylesStore.removeFilter(filter)"
							class="border border-blue-200 bg-blue-50 px-2 py-1 text-sm text-blue-700" />
					</div>

					<div v-if="!stylesStore.builderSelected.length" class="py-6 text-center text-xs text-slate-400"> Click filters on the left to select them </div>
				</div>

				<!-- Divider -->
				<div class="border-t border-slate-200" />

				<!-- Generate Button -->
				<Button
					:disabled="!stylesStore.builderSelected.length || stylesStore.builderGenerating"
					:loading="stylesStore.builderGenerating"
					@click="stylesStore.generateStyleFromFilters()">
					<Sparkles :size="14" />
					<span>{{ stylesStore.builderGenerating ? 'Generating...' : 'Generate Style with Claude' }}</span>
				</Button>

				<!-- Error -->
				<div v-if="stylesStore.builderError" class="rounded-lg border border-red-200 bg-red-50 p-3 text-xs text-red-600">
					{{ stylesStore.builderError }}
				</div>

				<!-- Preview -->
				<template v-if="stylesStore.generatedStyle">
					<div class="border-t border-slate-200" />

					<div class="flex flex-col gap-3">
						<span class="text-sm font-semibold text-slate-800">Preview</span>

						<div class="flex flex-col gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3">
							<div>
								<div class="text-xs font-medium text-slate-400">Name</div>
								<div class="text-sm font-semibold text-slate-800">{{ stylesStore.generatedStyle.name }}</div>
							</div>
							<div>
								<div class="text-xs font-medium text-slate-400">Description</div>
								<div class="text-xs leading-relaxed text-slate-600">{{ stylesStore.generatedStyle.description }}</div>
							</div>
							<div>
								<div class="text-xs font-medium text-slate-400">Prompt Template</div>
								<div class="text-xs leading-relaxed text-slate-600">{{ stylesStore.generatedStyle.prompt_template }}</div>
							</div>
							<div>
								<div class="text-xs font-medium text-slate-400">Negative Prompt</div>
								<div class="text-xs text-slate-400">{{ stylesStore.generatedStyle.negative_prompt }}</div>
							</div>
							<div>
								<div class="text-xs font-medium text-slate-400">Category</div>
								<div class="text-xs text-slate-600">{{ stylesStore.generatedStyle.category }}</div>
							</div>
						</div>
					</div>

					<!-- Spacer -->
					<div class="flex-1" />

					<!-- Create Style Button -->
					<Button severity="success" @click="onCreateStyle">
						<Plus :size="14" />
						<span>Create Style</span>
					</Button>
				</template>
			</div>
		</div>

		<!-- Generate Thumbnails Dialog -->
		<Dialog v-model:visible="showThumbnailDialog" header="Generate Thumbnails" modal :style="{ width: '32rem' }">
			<div class="flex flex-col gap-3 pt-2">
				<p class="text-sm text-slate-500">
					Select subcategory groups to generate thumbnail images for. Images will be created using the currently selected model.
				</p>

				<div class="flex items-center justify-between">
					<span class="text-xs font-medium text-slate-400">{{ thumbnailSelectedCount }} / {{ thumbnailGroups.length }} groups selected</span>
					<button class="text-xs text-blue-500 hover:text-blue-600" @click="toggleAllThumbnailGroups">
						{{ thumbnailSelectedCount === thumbnailGroups.length ? 'Deselect All' : 'Select All' }}
					</button>
				</div>

				<div class="max-h-80 overflow-y-auto rounded-lg border border-slate-200">
					<label
						v-for="group in thumbnailGroups"
						:key="group.key"
						class="flex cursor-pointer items-center gap-3 border-b border-slate-100 px-3 py-2 transition-colors last:border-b-0 hover:bg-slate-50">
						<Checkbox v-model="group.checked" :binary="true" />
						<div class="flex-1">
							<span class="text-sm font-medium text-slate-700">{{ group.key }}</span>
							<span class="ml-2 text-xs text-slate-400">({{ group.count }} filters)</span>
						</div>
					</label>
				</div>
			</div>
			<template #footer>
				<div class="flex justify-end gap-2">
					<Button severity="secondary" @click="showThumbnailDialog = false">Cancel</Button>
					<Button :disabled="thumbnailSelectedCount === 0" @click="onGenerateThumbnails">
						<ImageIcon :size="14" />
						<span>Create {{ thumbnailPromptCount }} Thumbnails</span>
					</Button>
				</div>
			</template>
		</Dialog>
	</div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useStylesStore } from '@/stores/styles';
import type { Filter } from '@/types/inference';
import thumbnailPrompts from '@/stores/thumbnail_prompts.json';

const router = useRouter();
const stylesStore = useStylesStore();

const searchQuery = ref('');
const activeCategory = ref('Basic Elements');
const activeSubcategory = ref('');

// ── Filter thumbnails ──
const thumbnailUrls = ref<Record<string, string>>({});
const loadingThumbnails = new Set<string>();

// ── Thumbnail generation ──
const showThumbnailDialog = ref(false);

interface ThumbnailGroup {
	key: string;
	count: number;
	checked: boolean;
}

const thumbnailGroups = reactive<ThumbnailGroup[]>(
	Object.keys(thumbnailPrompts).map((key) => ({
		key,
		count: 0,
		checked: false,
	})),
);

const thumbnailSelectedCount = computed(() => thumbnailGroups.filter((g) => g.checked).length);

const thumbnailPromptCount = computed(() => thumbnailGroups.filter((g) => g.checked).reduce((sum, g) => sum + g.count, 0));

function updateThumbnailCounts() {
	for (const group of thumbnailGroups) {
		group.count = stylesStore.builderFilters.filter((f) => `${f.category} / ${f.subcategory}` === group.key).length;
	}
}

function toggleAllThumbnailGroups() {
	const allSelected = thumbnailSelectedCount.value === thumbnailGroups.length;
	for (const group of thumbnailGroups) {
		group.checked = !allSelected;
	}
}

async function onGenerateThumbnails() {
	const selected = thumbnailGroups.filter((g) => g.checked).map((g) => g.key);
	showThumbnailDialog.value = false;
	await stylesStore.submitThumbnailBatch(selected);
}

const categories = computed(() => {
	const cats = new Set(stylesStore.builderFilters.map((f) => f.category));
	return Array.from(cats).sort();
});

const subcategories = computed(() => {
	const subs = new Set(stylesStore.builderFilters.filter((f) => f.category === activeCategory.value).map((f) => f.subcategory));
	return Array.from(subs).sort();
});

const visibleFilters = computed(() => {
	let filtered = stylesStore.builderFilters.filter((f) => f.category === activeCategory.value);

	if (activeSubcategory.value) {
		filtered = filtered.filter((f) => f.subcategory === activeSubcategory.value);
	}

	if (searchQuery.value.trim()) {
		const q = searchQuery.value.toLowerCase();
		filtered = filtered.filter((f) => f.label.toLowerCase().includes(q) || f.prompt.toLowerCase().includes(q));
	}

	return filtered;
});

const selectedPrompts = computed(() => new Set(stylesStore.builderSelected.map((f) => f.prompt)));

function isSelected(filter: Filter): boolean {
	return selectedPrompts.value.has(filter.prompt);
}

function onCategoryClick(cat: string) {
	activeCategory.value = cat;
	activeSubcategory.value = '';
}

watch(
	subcategories,
	(subs) => {
		activeSubcategory.value = subs[0] ?? '';
	},
	{ immediate: true },
);

watch(visibleFilters, async (filters) => {
	const toLoad = filters.filter(
		(f) => !(f.image in thumbnailUrls.value) && !loadingThumbnails.has(f.image) && stylesStore.availableThumbnails.includes(f.image),
	);
	if (!toLoad.length) return;

	for (const f of toLoad) loadingThumbnails.add(f.image);

	await Promise.all(
		toLoad.map(async (f) => {
			const url = await stylesStore.loadFilterThumbnail(f.image);
			if (url) thumbnailUrls.value[f.image] = url;
			loadingThumbnails.delete(f.image);
		}),
	);
});

function onCreateStyle() {
	router.push({ name: 'styles-new' });
}

onMounted(async () => {
	await stylesStore.loadFilters();
	updateThumbnailCounts();
	await stylesStore.loadFilterThumbnailList();
});
</script>

<style scoped>
:deep(.p-toolbar-center) {
	width: 50%;
}

:deep(.p-toolbar) {
	padding: 0.5rem 0.75rem;
}
</style>

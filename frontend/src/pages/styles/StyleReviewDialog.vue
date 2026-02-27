<template>
	<Dialog
		v-model:visible="visibleModel"
		modal
		:draggable="false"
		:closable="!stylesStore.reviewLoading"
		header="AI Style Review"
		class="w-175 max-w-[95vw]">
		<!-- Loading state -->
		<div v-if="stylesStore.reviewLoading" class="flex flex-col items-center gap-4 py-8">
			<ProgressBar mode="indeterminate" class="w-full" />
			<p class="text-sm text-surface-500">Analyzing {{ stylesStore.styles.length }} styles...</p>
		</div>

		<!-- Error state -->
		<div v-else-if="stylesStore.reviewError" class="flex flex-col gap-4 py-4">
			<div class="flex items-start gap-3 rounded-lg bg-red-50 p-4">
				<AlertCircle :size="20" class="mt-1 shrink-0 text-red-500" />
				<p class="text-sm text-red-700">{{ stylesStore.reviewError }}</p>
			</div>
			<div class="flex justify-end gap-2">
				<Button severity="secondary" @click="close">Cancel</Button>
				<Button severity="primary" @click="retry">Try Again</Button>
			</div>
		</div>

		<!-- Results state -->
		<div v-else-if="stylesStore.reviewSuggestions.length" class="flex flex-col gap-4">
			<!-- Select all toggle -->
			<div class="flex items-center justify-between border-b border-surface-200 pb-3">
				<div class="flex items-center gap-2">
					<Checkbox v-model="allSelected" :binary="true" @change="onToggleAll" />
					<span class="text-sm font-medium text-surface-600">
						{{ selectedCount }} of {{ stylesStore.reviewSuggestions.length }} selected
					</span>
				</div>
			</div>

			<!-- Scrollable suggestions list -->
			<div class="flex max-h-96 flex-col gap-3 overflow-y-auto pr-1">
				<div
					v-for="suggestion in stylesStore.reviewSuggestions"
					:key="suggestion.id"
					class="flex items-start gap-3 rounded-lg border border-surface-200 p-3 transition-colors"
					:class="selected.has(suggestion.id) ? 'border-blue-200 bg-blue-50/50' : ''">
					<Checkbox
						:modelValue="selected.has(suggestion.id)"
						:binary="true"
						@update:modelValue="onToggle(suggestion.id)" />
					<div class="flex min-w-0 flex-1 flex-col gap-2">
						<!-- Name row -->
						<div class="flex items-center gap-2">
							<span v-if="suggestion.name_changed" class="text-sm text-surface-400 line-through">
								{{ getOriginalName(suggestion.id) }}
							</span>
							<ArrowRight v-if="suggestion.name_changed" :size="14" class="shrink-0 text-surface-400" />
							<span class="text-sm font-medium" :class="suggestion.name_changed ? 'text-blue-600' : 'text-surface-700'">
								{{ suggestion.suggested_name }}
							</span>
						</div>

						<!-- Reason -->
						<p class="text-xs text-surface-400">{{ suggestion.reason }}</p>

						<!-- Tags -->
						<div class="flex flex-wrap gap-1">
							<span
								v-for="tag in suggestion.suggested_tags"
								:key="tag"
								class="rounded-full px-2 py-1 text-xs"
								:class="isExistingTag(tag) ? 'bg-surface-100 text-surface-600' : 'border border-dashed border-blue-300 text-blue-600'">
								{{ tag }}
							</span>
						</div>
					</div>
				</div>
			</div>

			<!-- Footer -->
			<div class="flex justify-end gap-2 border-t border-surface-200 pt-3">
				<Button severity="secondary" @click="close">Cancel</Button>
				<Button severity="primary" :disabled="!selectedCount" :loading="applying" @click="onApply">
					Apply {{ selectedCount }} Change{{ selectedCount !== 1 ? 's' : '' }}
				</Button>
			</div>
		</div>

		<!-- Empty state (no suggestions returned) -->
		<div v-else class="py-8 text-center">
			<p class="text-sm text-surface-500">No suggestions were returned. Your styles look great!</p>
			<Button severity="secondary" class="mt-4" @click="close">Close</Button>
		</div>
	</Dialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { useStylesStore } from '@/stores/styles';

const visibleModel = defineModel<boolean>('visible', { default: false });
const stylesStore = useStylesStore();

const selected = reactive(new Set<string>());
const applying = ref(false);

// Build a lookup of existing tag names for styling
const existingTagNames = computed(() => new Set(stylesStore.tags.map((t) => t.name.toLowerCase())));

const selectedCount = computed(() => selected.size);

const allSelected = computed({
	get: () => selected.size === stylesStore.reviewSuggestions.length && stylesStore.reviewSuggestions.length > 0,
	set: () => {},
});

// Auto-select all when suggestions load
watch(
	() => stylesStore.reviewSuggestions,
	(suggestions) => {
		selected.clear();
		for (const s of suggestions) {
			selected.add(s.id);
		}
	},
);

function getOriginalName(styleId: string): string {
	return stylesStore.styles.find((s) => s.id === styleId)?.name ?? '';
}

function isExistingTag(tagName: string): boolean {
	return existingTagNames.value.has(tagName.toLowerCase());
}

function onToggle(id: string) {
	if (selected.has(id)) {
		selected.delete(id);
	} else {
		selected.add(id);
	}
}

function onToggleAll() {
	if (selected.size === stylesStore.reviewSuggestions.length) {
		selected.clear();
	} else {
		for (const s of stylesStore.reviewSuggestions) {
			selected.add(s.id);
		}
	}
}

async function onApply() {
	const changes = stylesStore.reviewSuggestions
		.filter((s) => selected.has(s.id))
		.map((s) => ({
			style_id: s.id,
			name: s.name_changed ? s.suggested_name : undefined,
			tags: s.suggested_tags,
		}));

	applying.value = true;
	try {
		await stylesStore.applyReview(changes);
		close();
	} finally {
		applying.value = false;
	}
}

function retry() {
	stylesStore.fetchReview();
}

function close() {
	visibleModel.value = false;
	stylesStore.clearReview();
}
</script>

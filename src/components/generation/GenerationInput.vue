<template>
    <div class="pb-2 pl-2 pr-1 panel-scroll bg-surface-100 dark:bg-surface-100">
        <div
            class="card-wrapper"
            :class="{ 'is-dragging': isDragging }"
            @dragenter.prevent="handleDragEnter"
            @dragover.prevent="handleDragOver"
            @dragleave.prevent="handleDragLeave"
            @drop.prevent="handleDrop"
        >
            <Card :class="{ 'drag-highlight': isDragging }">
                <template #title>Generate</template>
                <template #content>
                    <PromptInput />
                    <div class="divider"></div>
                    <ModelSelector />
                    <PresetSelector />
                    <div class="divider"></div>
                    <ParameterControls />
                </template>
            </Card>

            <!-- Drag Overlay -->
            <div v-if="isDragging" class="drag-overlay">
                <div class="drag-content">
                    <i class="pi pi-image drag-icon"></i>
                    <div class="drag-text">
                        <div class="drag-title">Drop Image to Analyze</div>
                        <div class="drag-subtitle">Generate a prompt to recreate this image</div>
                    </div>
                </div>
            </div>

            <!-- Analysis Loading Overlay -->
            <div v-if="isAnalyzing" class="analysis-overlay">
                <div class="analysis-content">
                    <i class="pi pi-spin pi-spinner analysis-spinner"></i>
                    <div class="analysis-text">
                        <div class="analysis-title">Analyzing Image</div>
                        <div class="analysis-subtitle">Claude is generating a prompt...</div>
                    </div>
                </div>
            </div>
        </div>

        <div class="pt-2 shrink-0">
            <GenerateButton :queue-count="queueStore.queueLength" @generate="handleGenerate" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Card from 'primevue/card';
import PromptInput from '@/components/generation/PromptInput.vue';
import ModelSelector from '@/components/generation/ModelSelector.vue';
import PresetSelector from '@/components/generation/PresetSelector.vue';
import ParameterControls from '@/components/generation/ParameterControls.vue';
import GenerateButton from '@/components/generation/GenerateButton.vue';
import ImageCanvas from '@/components/generation/ImageCanvas.vue';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import type { GenerationParams } from '@/stores/queue';
import { analyzeImageForPrompt, fileToDataUrl, isValidImageFile } from '@/services/imageAnalysis';

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null);
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

// Drag and drop state
const isDragging = ref(false);
const isAnalyzing = ref(false);
const dragCounter = ref(0);

// Drag and drop handlers
const handleDragEnter = (e: DragEvent) => {
    dragCounter.value++;
    // Check if dragging files
    if (e.dataTransfer?.types.includes('Files')) {
        isDragging.value = true;
    }
};

const handleDragOver = (_e: DragEvent) => {
    // Required to allow drop - just prevent default
    // The visual state is handled by dragenter/dragleave
};

const handleDragLeave = () => {
    dragCounter.value--;
    if (dragCounter.value === 0) {
        isDragging.value = false;
    }
};

const handleDrop = async (e: DragEvent) => {
    dragCounter.value = 0;
    isDragging.value = false;

    const file = e.dataTransfer?.files?.[0];
    if (!file) return;

    if (!isValidImageFile(file)) {
        toast.add({
            severity: 'warn',
            summary: 'Invalid File',
            detail: 'Please drop an image file (PNG, JPEG, WebP, or GIF)',
            life: 3000,
        });
        return;
    }

    await analyzeDroppedImage(file);
};

const analyzeDroppedImage = async (file: File) => {
    isAnalyzing.value = true;

    try {
        // Convert file to data URL
        const dataUrl = await fileToDataUrl(file);

        // Call Claude API to analyze the image
        const prompt = await analyzeImageForPrompt(dataUrl);

        // Update the prompt in the store
        generationStore.currentParams.prompt = prompt;

        toast.add({
            severity: 'success',
            summary: 'Image Analyzed',
            detail: 'Prompt generated from image',
            life: 3000,
        });
    } catch (error) {
        console.error('Failed to analyze image:', error);
        toast.add({
            severity: 'error',
            summary: 'Analysis Failed',
            detail: error instanceof Error ? error.message : 'Failed to analyze image',
            life: 5000,
        });
    } finally {
        isAnalyzing.value = false;
    }
};

// Watch for model changes and update default parameters
watch(
  () => modelsStore.selectedModelId,
  (newModelId) => {
    const model = modelsStore.models.find((m) => m.id === newModelId);
    if (model) {
      // Update generation params with model defaults
      generationStore.currentParams.model = newModelId;
      if (model.defaultSteps) {
        generationStore.currentParams.steps = model.defaultSteps;
      }
      if (model.defaultGuidance !== undefined) {
        generationStore.currentParams.cfgScale = model.defaultGuidance;
      }
    }
  },
);

// Track displayed job IDs to avoid re-displaying
const displayedJobIds = ref<Set<string>>(new Set());

// Watch all jobs for NEW completions with result_path
watch(
    () => queueStore.jobs,
    (jobs) => {
        // Find completed jobs with result_path that we haven't displayed yet
        const newlyCompleted = jobs.filter((j) => j.status === 'completed' && j.result_path && !displayedJobIds.value.has(j.id));

        if (newlyCompleted.length > 0) {
            // Display the most recent newly completed one
            const latestJob = newlyCompleted[newlyCompleted.length - 1];
            if (canvasRef.value && latestJob.result_path) {
                canvasRef.value.setImage(latestJob.result_path);
                // Mark ALL newly completed jobs as displayed
                newlyCompleted.forEach((j) => displayedJobIds.value.add(j.id));
            }
        }
    },
    { deep: true },
);

const handleGenerate = async () => {
    const params = generationStore.currentParams;

    // Build queue params from current generation params
    const queueParams: GenerationParams = {
        prompt: params.prompt,
        negative_prompt: params.negativePrompt || undefined,
        steps: params.steps,
        cfg_scale: params.cfgScale,
        width: params.width,
        height: params.height,
        seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed,
        model: params.model,
    };

    try {
        // Add to queue - backend will handle processing
        await queueStore.addToQueue(queueParams);
    } catch (error) {
        console.error('Failed to add to queue:', error);
        toast.add({
            severity: 'error',
            summary: 'Generation Failed',
            detail: queueStore.error || 'Failed to add generation to queue',
            life: 5000,
        });
    }
};

defineExpose({
    canvasRef,
});
</script>

<style scoped>
@reference "tailwindcss";

.card-wrapper {
    @apply relative;
}

.card-wrapper.is-dragging :deep(.p-card) {
    @apply border-2 border-dashed;
    border-color: var(--p-primary-color);
}

/* Drag Overlay */
.drag-overlay {
    @apply absolute inset-0 z-50 flex items-center justify-center rounded-lg pointer-events-none;
    background: rgba(59, 130, 246, 0.15);
    backdrop-filter: blur(4px);
    border: 2px dashed var(--p-primary-color);
}

.drag-content {
    @apply flex flex-col items-center gap-3 p-6 rounded-xl;
    background: var(--p-surface-0);
    box-shadow: var(--p-card-shadow);
}

.drag-icon {
    @apply text-4xl;
    color: var(--p-primary-color);
}

.drag-text {
    @apply text-center;
}

.drag-title {
    @apply text-lg font-semibold;
    color: var(--p-text-color);
}

.drag-subtitle {
    @apply text-sm;
    color: var(--p-text-muted-color);
}

/* Analysis Loading Overlay */
.analysis-overlay {
    @apply absolute inset-0 z-50 flex items-center justify-center rounded-lg;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
}

.analysis-content {
    @apply flex flex-col items-center gap-4 p-8 rounded-xl;
    background: var(--p-surface-0);
    box-shadow: var(--p-card-shadow);
}

.analysis-spinner {
    @apply text-4xl;
    color: var(--p-primary-color);
}

.analysis-text {
    @apply text-center;
}

.analysis-title {
    @apply text-lg font-semibold;
    color: var(--p-text-color);
}

.analysis-subtitle {
    @apply text-sm;
    color: var(--p-text-muted-color);
}
</style>

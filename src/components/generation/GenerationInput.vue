<template>
    <div class="pb-2 pl-2 pr-1 panel-scroll bg-surface-100 dark:bg-surface-100">
        <Card>
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

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null);
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

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

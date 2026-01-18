<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Generate</h1>
      <div>Text-to-image, Image-to-image, and Inpainting</div>
    </div>
    <div class="workspace-content">
      <Splitter>
        <SplitterPanel :size="30" :minSize="20">
          <Splitter>
            <SplitterPanel :size="70" :minSize="25">
              <div class="p-2">
              <h2>Generate</h2>
              <PromptInput />
              <div class="divider"></div>
              <ModelSelector />
              <PresetSelector />
              <div class="divider"></div>
              <ParameterControls />
              <div class="divider"></div>
              <GenerateButton :queue-count="queueStore.queueLength" @generate="handleGenerate" />
              </div>
            </SplitterPanel>
            <SplitterPanel :size="30" :minSize="25">
              <QueuePanel />
            </SplitterPanel>
          </Splitter>
        </SplitterPanel>
        <SplitterPanel :size="70" :minSize="10">
          <h2>Canvas</h2>
          <ImageCanvas ref="canvasRef" />
        </SplitterPanel>
      </Splitter>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import PromptInput from '@/components/generation/PromptInput.vue';
import ModelSelector from '@/components/generation/ModelSelector.vue';
import PresetSelector from '@/components/generation/PresetSelector.vue';
import ParameterControls from '@/components/generation/ParameterControls.vue';
import GenerateButton from '@/components/generation/GenerateButton.vue';
import QueuePanel from '@/components/queue/QueuePanel.vue';
import ImageCanvas from '@/components/generation/ImageCanvas.vue';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import type { GenerationParams } from '@/stores/queue';

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null);
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const toast = useToast();

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

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
            <SplitterPanel :size="60" :minSize="25">
              <GenerationInput />
            </SplitterPanel>
            <SplitterPanel :size="40" :minSize="25">
              <QueuePanel />
            </SplitterPanel>
          </Splitter>
        </SplitterPanel>
        <SplitterPanel :size="70" :minSize="10">
          <ImageCanvas ref="canvasRef" />
        </SplitterPanel>
      </Splitter>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import QueuePanel from '@/components/queue/QueuePanel.vue';
import ImageCanvas from '@/components/generation/ImageCanvas.vue';
import GenerationInput from '@/components/generation/GenerationInput.vue';
import { useQueueStore } from '@/stores/queue';

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null);
const queueStore = useQueueStore();

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

defineExpose({
  canvasRef,
});
</script>

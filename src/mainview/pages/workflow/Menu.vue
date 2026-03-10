<template>
  <MenuPanel title="Workflow" :icon="Workflow">
    <template #content>
      <div class="flex flex-col gap-4">
        <!-- Workflow name -->
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-slate-500">Workflow Name</label>
          <InputText
            v-model="workflowStore.workflowName"
            class="w-full"
            size="small"
            placeholder="Untitled Workflow" />
        </div>

        <!-- Save / New / Delete -->
        <div class="flex gap-2">
          <Button class="flex-1" severity="primary" size="small" @click="handleSave">
            <Save :size="14" />
            <span class="ml-1">Save</span>
          </Button>
          <Button severity="secondary" variant="outlined" size="small" @click="handleNew">
            <FilePlus :size="14" />
          </Button>
          <Button
            v-if="workflowStore.workflowId"
            severity="danger"
            variant="outlined"
            size="small"
            @click="handleDelete">
            <Trash2 :size="14" />
          </Button>
        </div>

        <!-- Saved workflows list -->
        <div v-if="workflowStore.workflows.length > 0" class="flex flex-col gap-1">
          <label class="text-xs font-medium text-slate-500">Saved Workflows</label>
          <div class="flex flex-col gap-1 max-h-40 overflow-y-auto">
            <button
              v-for="wf in workflowStore.workflows"
              :key="wf.id"
              class="flex items-center gap-2 px-2 py-1 rounded text-sm text-left transition-colors"
              :class="wf.id === workflowStore.workflowId
                ? 'bg-blue-600/20 text-blue-400'
                : 'text-slate-600 hover:bg-surface-100'"
              @click="handleLoad(wf.id)">
              <Workflow :size="12" class="shrink-0" />
              <span class="truncate">{{ wf.name }}</span>
            </button>
          </div>
        </div>

        <!-- Divider -->
        <div class="border-t border-surface-200" />

        <!-- Add Node palette -->
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-slate-500">Add Node</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="(info, nodeType) in NODE_REGISTRY"
              :key="nodeType"
              class="flex items-center gap-2 px-3 py-2 rounded-lg border border-surface-200 text-sm text-slate-600 hover:bg-surface-50 hover:border-surface-300 transition-colors"
              @click="handleAddNode(nodeType as WorkflowNodeType)">
              <component :is="getIcon(info.icon)" :size="14" :class="getIconColor(info.color)" />
              <span class="truncate">{{ info.label }}</span>
            </button>
          </div>
        </div>
      </div>
    </template>
    <template #footer>
      <div v-if="workflowStore.isRunning" class="flex gap-2">
        <Button class="flex-1" severity="danger" raised @click="handleCancel">
          <Square :size="14" />
          <span class="ml-1">Cancel</span>
        </Button>
      </div>
      <div v-else class="flex gap-2">
        <Button
          class="flex-1"
          severity="primary"
          raised
          :disabled="workflowStore.nodes.length === 0"
          @click="handleRun">
          <Play :size="14" />
          <span class="ml-1">Run Workflow</span>
        </Button>
      </div>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import {
  Workflow,
  Save,
  FilePlus,
  Trash2,
  Play,
  Square,
  ImagePlus,
  Eye,
  Sparkles,
  Type,
  Monitor,
} from 'lucide-vue-next';
import MenuPanel from '@/components/MenuPanel.vue';
import { NODE_REGISTRY } from './nodeRegistry';
import { useWorkflowStore } from '@/stores/workflow';
import type { WorkflowNodeType } from '@/types/workflow';

const workflowStore = useWorkflowStore();

const iconMap: Record<string, typeof ImagePlus> = {
  ImagePlus,
  Eye,
  Sparkles,
  Type,
  Monitor,
};

function getIcon(iconName: string) {
  return iconMap[iconName] ?? ImagePlus;
}

function getIconColor(bgClass: string): string {
  // Map bg color classes to text color classes
  const colorMap: Record<string, string> = {
    'bg-blue-600': 'text-blue-500',
    'bg-purple-600': 'text-purple-500',
    'bg-green-600': 'text-green-500',
    'bg-amber-600': 'text-amber-500',
    'bg-red-600': 'text-red-500',
  };
  return colorMap[bgClass] ?? 'text-slate-500';
}

function handleSave() {
  workflowStore.saveWorkflow();
}

function handleNew() {
  workflowStore.newWorkflow();
}

function handleDelete() {
  if (!workflowStore.workflowId) return;
  workflowStore.deleteWorkflow(workflowStore.workflowId);
}

function handleLoad(id: string) {
  workflowStore.loadWorkflow(id);
}

function handleAddNode(nodeType: WorkflowNodeType) {
  workflowStore.addNode(nodeType);
}

function handleRun() {
  workflowStore.runWorkflow();
}

function handleCancel() {
  workflowStore.cancelWorkflow();
}

onMounted(async () => {
  await workflowStore.loadWorkflows();
  if (!workflowStore.activeWorkflow && workflowStore.workflows.length === 0) {
    await workflowStore.loadDefaultWorkflow();
  }
});
</script>

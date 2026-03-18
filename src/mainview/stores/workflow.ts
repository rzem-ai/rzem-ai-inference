import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { Node, Edge, NodeChange, EdgeChange, Connection } from '@vue-flow/core';
import { applyNodeChanges, applyEdgeChanges } from '@vue-flow/core';
import type {
  WorkflowNodeType,
  WorkflowNodeData,
  WorkflowListItem,
  WorkflowEvent,
  NodeExecutionState,
  SerializedWorkflow,
} from '@/types/workflow';

// Default data factories for each node type
const NODE_DEFAULTS: Record<WorkflowNodeType, () => WorkflowNodeData> = {
  image_input: () => ({
    type: 'image_input',
    label: 'Image Input',
    imagePath: '',
  }),
  vision_qa: () => ({
    type: 'vision_qa',
    label: 'Vision Q&A',
    provider: 'chat_service',
    question: '',
    endpointUrl: '',
    modelName: '',
    apiKey: '',
    maxTokens: 1024,
  }),
  image_gen: () => ({
    type: 'image_gen',
    label: 'Image Generation',
    bundleId: '',
    steps: 28,
    cfg: 3.5,
    width: 1024,
    height: 1024,
    seed: -1,
    sampler: 'euler',
    scheduler: 'simple',
  }),
  text: () => ({
    type: 'text',
    label: 'Text',
    content: '',
    isTemplate: false,
  }),
  output: () => ({
    type: 'output',
    label: 'Output',
  }),
};

// Non-reactive module-level state
let _pollTimer: ReturnType<typeof setInterval> | null = null;

// Explicit state interface prevents TS7056 serialization overflow from Vue Flow's deep generics
interface WorkflowState {
  workflows: WorkflowListItem[];
  activeWorkflow: SerializedWorkflow | null;
  workflowName: string;
  nodes: any[];
  edges: any[];
  viewport: { x: number; y: number; zoom: number };
  selectedNodeId: string | null;
  isRunning: boolean;
  runId: string | null;
  nodeStates: Record<string, NodeExecutionState>;
  finalOutputs: Record<string, unknown> | null;
  error: string | null;
}

export const useWorkflowStore = defineStore('workflow', {
  state: (): WorkflowState => ({
    workflows: [],
    activeWorkflow: null,
    workflowName: '',
    nodes: [],
    edges: [],
    viewport: { x: 0, y: 0, zoom: 1 },
    selectedNodeId: null,
    isRunning: false,
    runId: null,
    nodeStates: {},
    finalOutputs: null,
    error: null,
  }),

  getters: {
    workflowId(state): string | null {
      return state.activeWorkflow?.id ?? null;
    },

    selectedNode(state): Node | undefined {
      return state.nodes.find((n) => n.id === state.selectedNodeId);
    },

    selectedNodeData(): WorkflowNodeData | undefined {
      return this.selectedNode?.data as WorkflowNodeData | undefined;
    },

    selectedNodeState(state): NodeExecutionState {
      if (!state.selectedNodeId) return { status: 'idle', error: null, outputs: null, progress: 0 };
      return state.nodeStates[state.selectedNodeId] ?? { status: 'idle', error: null, outputs: null, progress: 0 };
    },
  },

  actions: {
    // ── Workflow CRUD ──

    async loadWorkflows() {
      const api = await getApiAsync();
      const res = await api.get_workflows();
      if (res.status === 'success') {
        this.workflows = res.workflows ?? [];
      }
    },

    async loadWorkflow(id: string) {
      const api = await getApiAsync();
      const res = await api.get_workflow({ workflow_id: id });
      if (res.status === 'success' && res.workflow) {
        const parsed: SerializedWorkflow = JSON.parse(res.workflow.graph_json);
        this.activeWorkflow = parsed;
        this.workflowName = parsed.name;
        this.nodes = parsed.nodes.map((n) => ({
          id: n.id,
          type: n.type,
          position: n.position,
          data: n.data,
        }));
        this.edges = parsed.edges.map((e) => ({
          id: e.id,
          source: e.source,
          sourceHandle: e.sourceHandle,
          target: e.target,
          targetHandle: e.targetHandle,
        }));
        if (parsed.viewport) {
          this.viewport = parsed.viewport;
        }
      }
    },

    async saveWorkflow() {
      if (!this.activeWorkflow) return;

      // Sync workflowName back to activeWorkflow before saving
      this.activeWorkflow.name = this.workflowName;

      const serialized: SerializedWorkflow = {
        id: this.activeWorkflow.id,
        name: this.workflowName,
        description: this.activeWorkflow.description,
        nodes: this.nodes.map((n) => ({
          id: n.id,
          type: n.type as WorkflowNodeType,
          position: n.position,
          data: n.data as WorkflowNodeData,
        })),
        edges: this.edges.map((e) => ({
          id: e.id,
          source: e.source,
          sourceHandle: e.sourceHandle ?? '',
          target: e.target,
          targetHandle: e.targetHandle ?? '',
        })),
        viewport: this.viewport,
      };

      const api = await getApiAsync();
      const res = await api.save_workflow({
        workflow_id: serialized.id,
        name: serialized.name,
        description: serialized.description,
        graph_json: JSON.stringify(serialized),
      });

      if (res.status === 'success') {
        this.activeWorkflow = serialized;
        await this.loadWorkflows();
      }
    },

    async deleteWorkflow(id: string) {
      const api = await getApiAsync();
      const res = await api.delete_workflow({ workflow_id: id });
      if (res.status === 'success') {
        this.workflows = this.workflows.filter((w) => w.id !== id);
        if (this.activeWorkflow?.id === id) {
          this.activeWorkflow = null;
          this.nodes = [];
          this.edges = [];
        }
      }
    },

    // ── Node operations ──

    nodeState(nodeId: string): NodeExecutionState {
      return this.nodeStates[nodeId] ?? { status: 'idle', error: null, outputs: null, progress: 0 };
    },

    getNodeState(nodeId: string): NodeExecutionState {
      return this.nodeState(nodeId);
    },

    addNode(type: WorkflowNodeType, position?: { x: number; y: number }) {
      if (!position) {
        // Place nodes in a staggered grid when added from palette
        const count = this.nodes.length;
        position = { x: 100 + (count % 3) * 250, y: 100 + Math.floor(count / 3) * 150 };
      }
      const id = crypto.randomUUID();
      const data = NODE_DEFAULTS[type]();
      const node: Node = {
        id,
        type,
        position,
        data,
      };
      this.nodes.push(node);
      return id;
    },

    removeNode(id: string) {
      this.nodes = this.nodes.filter((n) => n.id !== id);
      this.edges = this.edges.filter((e) => e.source !== id && e.target !== id);
      if (this.selectedNodeId === id) {
        this.selectedNodeId = null;
      }
    },

    updateNodeData(nodeId: string, data: Partial<WorkflowNodeData>) {
      const node = this.nodes.find((n) => n.id === nodeId);
      if (node) {
        node.data = { ...node.data, ...data };
      }
    },

    selectNode(id: string | null) {
      this.selectedNodeId = id;
    },

    // ── Execution ──

    async runWorkflow() {
      if (this.isRunning || !this.activeWorkflow) return;

      this.resetExecution();
      this.isRunning = true;
      this.error = null;

      const serialized: SerializedWorkflow = {
        id: this.activeWorkflow.id,
        name: this.activeWorkflow.name,
        description: this.activeWorkflow.description,
        nodes: this.nodes.map((n) => ({
          id: n.id,
          type: n.type as WorkflowNodeType,
          position: n.position,
          data: n.data as WorkflowNodeData,
        })),
        edges: this.edges.map((e) => ({
          id: e.id,
          source: e.source,
          sourceHandle: e.sourceHandle ?? '',
          target: e.target,
          targetHandle: e.targetHandle ?? '',
        })),
        viewport: this.viewport,
      };

      const api = await getApiAsync();
      const res = await api.run_workflow({ graph_json: JSON.stringify(serialized) });
      if (res.status === 'success' && res.run_id) {
        this.runId = res.run_id;
        this.startWorkflowPolling();
      } else {
        this.isRunning = false;
        this.error = 'Failed to start workflow';
      }
    },

    async cancelWorkflow() {
      if (!this.runId) return;
      const api = await getApiAsync();
      await api.cancel_workflow({ run_id: this.runId });
      this.stopWorkflowPolling();
      this.isRunning = false;
      this.runId = null;
    },

    startWorkflowPolling() {
      if (_pollTimer) return;
      _pollTimer = setInterval(() => this.pollWorkflowEvents(), 200);
    },

    stopWorkflowPolling() {
      if (_pollTimer) {
        clearInterval(_pollTimer);
        _pollTimer = null;
      }
    },

    async pollWorkflowEvents() {
      const api = await getApiAsync();
      try {
        const res = await api.poll_workflow_events();
        if (res.status === 'success' && res.events?.length) {
          this.processWorkflowEvents(res.events as WorkflowEvent[]);
        }
      } catch {
        // Silently ignore poll errors
      }
    },

    processWorkflowEvents(events: WorkflowEvent[]) {
      for (const event of events) {
        switch (event.type) {
          case 'workflow_started':
            this.isRunning = true;
            break;

          case 'node_started':
            if (event.node_id) {
              this.nodeStates[event.node_id] = {
                status: 'running',
                error: null,
                outputs: null,
                progress: 0,
              };
            }
            break;

          case 'node_progress':
            if (event.node_id && this.nodeStates[event.node_id]) {
              this.nodeStates[event.node_id].progress = event.progress ?? 0;
            }
            break;

          case 'node_completed':
            if (event.node_id) {
              this.nodeStates[event.node_id] = {
                status: 'completed',
                error: null,
                outputs: event.outputs ?? null,
                progress: 1,
              };
            }
            break;

          case 'node_failed':
            if (event.node_id) {
              this.nodeStates[event.node_id] = {
                status: 'failed',
                error: event.error ?? 'Unknown error',
                outputs: null,
                progress: 0,
              };
            }
            break;

          case 'workflow_completed':
            this.isRunning = false;
            this.finalOutputs = event.outputs ?? event.final_outputs ?? null;
            this.stopWorkflowPolling();
            break;

          case 'workflow_failed':
            this.isRunning = false;
            this.error = event.error ?? 'Workflow failed';
            this.stopWorkflowPolling();
            break;
        }
      }
    },

    resetExecution() {
      this.nodeStates = {};
      this.finalOutputs = null;
      this.error = null;
      this.runId = null;
    },

    // ── Workflow management ──

    newWorkflow() {
      const id = crypto.randomUUID();
      this.activeWorkflow = {
        id,
        name: 'Untitled Workflow',
        description: '',
        nodes: [],
        edges: [],
        viewport: { x: 0, y: 0, zoom: 1 },
      };
      this.workflowName = 'Untitled Workflow';
      this.nodes = [];
      this.edges = [];
      this.viewport = { x: 0, y: 0, zoom: 1 };
      this.selectedNodeId = null;
      this.resetExecution();
    },

    async loadDefaultWorkflow() {
      // Fetch bundles to pre-select the first available one
      let firstBundle: Record<string, any> | null = null;
      try {
        const api = await getApiAsync();
        const bundleRes = await api.get_bundles();
        firstBundle = bundleRes.status === 'success' && bundleRes.bundles?.length
          ? bundleRes.bundles[0] as Record<string, any>
          : null;
      } catch (err) {
        console.warn('[workflow] Failed to fetch bundles for default workflow:', err);
      }

      const textId = crypto.randomUUID();
      const genId = crypto.randomUUID();
      const visionId = crypto.randomUUID();
      const outputId = crypto.randomUUID();

      this.activeWorkflow = {
        id: crypto.randomUUID(),
        name: 'Badger in Space',
        description: 'Generate an image from a prompt, then describe it with Vision QA.',
        nodes: [],
        edges: [],
        viewport: { x: 0, y: 0, zoom: 1 },
      };
      this.workflowName = 'Badger in Space';

      const genData = NODE_DEFAULTS.image_gen();
      if (firstBundle && 'bundleId' in genData) {
        genData.bundleId = firstBundle.id;
        if (firstBundle.steps) genData.steps = firstBundle.steps;
        if (firstBundle.cfg_scale) genData.cfg = firstBundle.cfg_scale;
        if (firstBundle.sampler) genData.sampler = firstBundle.sampler;
        if (firstBundle.scheduler) genData.scheduler = firstBundle.scheduler;
      }

      this.nodes = [
        {
          id: textId,
          type: 'text',
          position: { x: 50, y: 150 },
          data: { type: 'text', label: 'Prompt', content: 'A Badger in Space', isTemplate: false },
        },
        {
          id: genId,
          type: 'image_gen',
          position: { x: 350, y: 100 },
          data: genData,
        },
        {
          id: visionId,
          type: 'vision_qa',
          position: { x: 650, y: 50 },
          data: {
            ...NODE_DEFAULTS.vision_qa(),
            label: 'Describe Image',
            question: 'Write a detailed report describing this image, including the subject, style, composition, colors, and mood.',
          },
        },
        {
          id: outputId,
          type: 'output',
          position: { x: 950, y: 100 },
          data: { type: 'output', label: 'Results' },
        },
      ];

      this.edges = [
        {
          id: crypto.randomUUID(),
          source: textId,
          sourceHandle: 'text',
          target: genId,
          targetHandle: 'text',
        },
        {
          id: crypto.randomUUID(),
          source: genId,
          sourceHandle: 'image',
          target: visionId,
          targetHandle: 'image',
        },
        {
          id: crypto.randomUUID(),
          source: visionId,
          sourceHandle: 'text',
          target: outputId,
          targetHandle: 'text',
        },
        {
          id: crypto.randomUUID(),
          source: genId,
          sourceHandle: 'image',
          target: outputId,
          targetHandle: 'image',
        },
      ];

      this.viewport = { x: 0, y: 0, zoom: 1 };
      this.selectedNodeId = null;
      this.resetExecution();
    },

    // ── Vue Flow change handlers ──

    onNodesChange(changes: NodeChange[]) {
      this.nodes = applyNodeChanges(changes, this.nodes as any) as Node[];
    },

    onEdgesChange(changes: EdgeChange[]) {
      this.edges = applyEdgeChanges(changes, this.edges as any) as Edge[];
    },

    onConnect(connection: Connection) {
      const id = crypto.randomUUID();
      const newEdge: Edge = {
        id,
        source: connection.source,
        sourceHandle: connection.sourceHandle ?? undefined,
        target: connection.target,
        targetHandle: connection.targetHandle ?? undefined,
      };
      this.edges.push(newEdge);
    },
  },
});

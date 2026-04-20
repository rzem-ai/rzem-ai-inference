// Port data types
export type PortDataType = 'image' | 'text';

// Node types
export type WorkflowNodeType = 'image_input' | 'vision_qa' | 'image_gen' | 'text' | 'output';

// Node data interfaces
export interface ImageInputNodeData {
  type: 'image_input';
  label: string;
  imagePath: string;
}

export interface VisionQANodeData {
  type: 'vision_qa';
  label: string;
  provider: 'chat_service' | 'openai_compatible';
  question: string;
  // OpenAI-compatible settings
  endpointUrl: string;
  modelName: string;
  apiKey: string;
  maxTokens: number;
}

export interface ImageGenNodeData {
  type: 'image_gen';
  label: string;
  bundleId: string;
  steps: number;
  cfg: number;
  width: number;
  height: number;
  seed: number;
  sampler: string;
  scheduler: string;
}

export interface TextNodeData {
  type: 'text';
  label: string;
  content: string;
  isTemplate: boolean;
}

export interface OutputNodeData {
  type: 'output';
  label: string;
}

export type WorkflowNodeData = ImageInputNodeData | VisionQANodeData | ImageGenNodeData | TextNodeData | OutputNodeData;

// Node execution state
export type NodeStatus = 'idle' | 'pending' | 'running' | 'completed' | 'failed';

export interface NodeExecutionState {
  status: NodeStatus;
  error: string | null;
  outputs: Record<string, unknown> | null;
  progress: number;
}

// Serialization types (what gets sent to/from backend)
export interface SerializedNode {
  id: string;
  type: WorkflowNodeType;
  position: { x: number; y: number };
  data: WorkflowNodeData;
}

export interface SerializedEdge {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
}

export interface SerializedWorkflow {
  id: string;
  name: string;
  description: string;
  nodes: SerializedNode[];
  edges: SerializedEdge[];
  viewport?: { x: number; y: number; zoom: number };
}

// Workflow list item (from get_workflows)
export interface WorkflowListItem {
  id: string;
  name: string;
  description: string;
  created_at: number;
  updated_at: number;
}

// Workflow events from polling
export interface WorkflowEvent {
  type: 'workflow_started' | 'node_started' | 'node_progress' | 'node_completed' | 'node_failed' | 'workflow_completed' | 'workflow_failed';
  run_id: string;
  node_id?: string;
  progress?: number;
  outputs?: Record<string, unknown>;
  error?: string;
  final_outputs?: Record<string, unknown>;
}

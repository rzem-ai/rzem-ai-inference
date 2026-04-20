import type { PortDataType, WorkflowNodeType, WorkflowNodeData } from '@/types/workflow';

export interface PortDefinition {
  id: string;
  label: string;
  dataType: PortDataType;
}

export interface NodeTypeInfo {
  label: string;
  icon: string;
  color: string;
  inputs: PortDefinition[];
  outputs: PortDefinition[];
  defaultData: WorkflowNodeData;
}

export const NODE_REGISTRY: Record<WorkflowNodeType, NodeTypeInfo> = {
  image_input: {
    label: 'Image Input',
    icon: 'ImagePlus',
    color: 'bg-blue-600',
    inputs: [],
    outputs: [{ id: 'image', label: 'Image', dataType: 'image' }],
    defaultData: { type: 'image_input', label: 'Image Input', imagePath: '' },
  },
  vision_qa: {
    label: 'Vision QA',
    icon: 'Eye',
    color: 'bg-purple-600',
    inputs: [
      { id: 'image', label: 'Image', dataType: 'image' },
      { id: 'text', label: 'Question', dataType: 'text' },
    ],
    outputs: [{ id: 'text', label: 'Answer', dataType: 'text' }],
    defaultData: {
      type: 'vision_qa',
      label: 'Vision QA',
      provider: 'chat_service',
      question: '',
      endpointUrl: '',
      modelName: '',
      apiKey: '',
      maxTokens: 1024,
    },
  },
  image_gen: {
    label: 'Image Gen',
    icon: 'Sparkles',
    color: 'bg-green-600',
    inputs: [{ id: 'text', label: 'Prompt', dataType: 'text' }],
    outputs: [{ id: 'image', label: 'Image', dataType: 'image' }],
    defaultData: {
      type: 'image_gen',
      label: 'Image Gen',
      bundleId: '',
      steps: 20,
      cfg: 7,
      width: 512,
      height: 512,
      seed: -1,
      sampler: 'euler',
      scheduler: 'normal',
    },
  },
  text: {
    label: 'Text',
    icon: 'Type',
    color: 'bg-amber-600',
    inputs: [{ id: 'text', label: 'Input', dataType: 'text' }],
    outputs: [{ id: 'text', label: 'Text', dataType: 'text' }],
    defaultData: { type: 'text', label: 'Text', content: '', isTemplate: false },
  },
  output: {
    label: 'Output',
    icon: 'Monitor',
    color: 'bg-red-600',
    inputs: [
      { id: 'image', label: 'Image', dataType: 'image' },
      { id: 'text', label: 'Text', dataType: 'text' },
    ],
    outputs: [],
    defaultData: { type: 'output', label: 'Output' },
  },
};

export function isValidConnection(
  sourceHandleType: PortDataType,
  targetHandleType: PortDataType,
): boolean {
  return sourceHandleType === targetHandleType;
}

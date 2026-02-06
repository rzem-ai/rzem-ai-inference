import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useGenerationStore } from './generation';

/**
 * Chat message role
 */
export type MessageRole = 'user' | 'assistant';

/**
 * A single chat message
 */
export interface ChatMessage {
  role: MessageRole;
  content: string;
  timestamp: number;
  suggestedPrompt?: string;
}

/**
 * Context about current generation settings for the chatbot
 */
export interface GenerationContext {
  current_prompt: string;
  width: number;
  height: number;
  model: string;
  style_loras: string[];
  steps: number;
  cfg_scale: number;
}

/**
 * Request to refine a prompt
 */
interface RefineRequest {
  user_message: string;
  history: Array<{ role: string; content: string; timestamp: number }>;
  context: GenerationContext;
}

/**
 * Response from the chatbot
 */
interface ChatResponse {
  message: string;
  suggested_prompt?: string;
}

/**
 * Quick suggestion prompts for users
 */
export const QUICK_SUGGESTIONS = [
  'Make my prompt more detailed',
  'Suggest a cinematic style',
  'Help me describe lighting',
  'Add artistic modifiers',
  'Improve composition details',
  'Make it more photorealistic',
] as const;

export const useChatbotStore = defineStore('chatbot', () => {
  // State
  const messages = ref<ChatMessage[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isPanelOpen = ref(false);
  const isInitialized = ref(false);

  // Access generation store for context
  const generationStore = useGenerationStore();

  // Computed
  const hasMessages = computed(() => messages.value.length > 0);

  const lastAssistantMessage = computed(() => {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      if (messages.value[i].role === 'assistant') {
        return messages.value[i];
      }
    }
    return null;
  });

  const hasSuggestedPrompt = computed(() => {
    return lastAssistantMessage.value?.suggestedPrompt !== undefined;
  });

  // Build the generation context from current store state
  function buildContext(): GenerationContext {
    const params = generationStore.currentParams;
    return {
      current_prompt: params.prompt,
      width: params.width,
      height: params.height,
      model: params.bundle_id || params.model_component_id || 'flux',
      style_loras: params.loras?.map((l) => l.id) ?? [],
      steps: params.steps,
      cfg_scale: params.cfg_scale,
    };
  }

  // Actions

  /**
   * Minimal initialization (no backend calls needed)
   */
  async function initialize(): Promise<void> {
    if (isInitialized.value) return
    // No-op - chatbot state is local UI state
    isInitialized.value = true
  }

  /**
   * Send a message to the chatbot
   */
  async function sendMessage(userMessage: string): Promise<void> {
    if (!userMessage.trim() || isLoading.value) return;

    error.value = null;

    // Add user message
    const userMsg: ChatMessage = {
      role: 'user',
      content: userMessage.trim(),
      timestamp: Date.now(),
    };
    messages.value.push(userMsg);

    isLoading.value = true;

    try {
      // Build request with history and context
      const request: RefineRequest = {
        user_message: userMessage,
        history: messages.value.slice(0, -1).map((msg) => ({
          role: msg.role,
          content: msg.content,
          timestamp: msg.timestamp,
        })),
        context: buildContext(),
      };

      console.log('request:', request);

      const response = await invoke<ChatResponse>('chat_refine_prompt', {
        request,
      });

      console.log('response:', response);
      console.log('response:message:', response.message);
      console.log('response:prompt:', response.suggested_prompt);

      // Add assistant response
      const assistantMsg: ChatMessage = {
        role: 'assistant',
        content: response.message,
        timestamp: Date.now(),
        suggestedPrompt: response.suggested_prompt,
      };
      messages.value.push(assistantMsg);
    } catch (e) {
      const errorMessage = String(e);
      error.value = errorMessage;

      // If it's an API key error, provide helpful message
      if (errorMessage.includes('API key not configured')) {
        error.value = 'Claude API key required. Configure it in Settings.';
      }

      // Remove user message on error
      messages.value.pop();
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Apply a suggested prompt to the generation params
   */
  function applyPrompt(prompt: string): void {
    generationStore.currentParams.prompt = prompt;
  }

  /**
   * Apply the last suggested prompt
   */
  function applyLastSuggestion(): void {
    const suggestion = lastAssistantMessage.value?.suggestedPrompt;
    if (suggestion) {
      applyPrompt(suggestion);
    }
  }

  /**
   * Clear all chat messages
   */
  function clearChat(): void {
    messages.value = [];
    error.value = null;
  }

  /**
   * Toggle the chat panel visibility
   */
  function togglePanel(): void {
    isPanelOpen.value = !isPanelOpen.value;
  }

  /**
   * Open the chat panel
   */
  function openPanel(): void {
    isPanelOpen.value = true;
  }

  /**
   * Close the chat panel
   */
  function closePanel(): void {
    isPanelOpen.value = false;
  }

  return {
    // State
    messages,
    isLoading,
    error,
    isPanelOpen,
    isInitialized,
    // Computed
    hasMessages,
    lastAssistantMessage,
    hasSuggestedPrompt,
    // Actions
    initialize,
    sendMessage,
    applyPrompt,
    applyLastSuggestion,
    clearChat,
    togglePanel,
    openPanel,
    closePanel,
  };
});

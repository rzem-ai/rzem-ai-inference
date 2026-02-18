import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';
import type { Conversation, ConversationMessage, ChatEvent } from '@/types/inference';
import { useInferenceStore } from '@/stores/inference';

let _api: PywebviewAPI | null = null;
let _pollTimer: ReturnType<typeof setInterval> | null = null;

export const useChatStore = defineStore('chat', {
  state: () => ({
    conversations: [] as Conversation[],
    activeConversationId: null as string | null,
    messages: [] as ConversationMessage[],
    isStreaming: false,
    streamingText: '',
    isConfigured: false,
    pendingImagePaths: [] as string[],
  }),

  getters: {
    activeConversation(state): Conversation | null {
      return state.conversations.find(c => c.id === state.activeConversationId) ?? null;
    },
  },

  actions: {
    setApi(apiRef: PywebviewAPI) {
      _api = apiRef;
    },

    async checkConfigured() {
      if (!_api) return;
      const res = await _api.chat_is_configured();
      if (res.status === 'success') {
        this.isConfigured = res.configured ?? false;
      }
    },

    async setApiKey(apiKey: string) {
      if (!_api) return;
      const res = await _api.chat_set_api_key({ api_key: apiKey });
      if (res.status === 'success') {
        this.isConfigured = true;
      }
    },

    async loadConversations() {
      if (!_api) return;
      const res = await _api.chat_get_conversations();
      if (res.status === 'success' && res.conversations) {
        this.conversations = res.conversations;
      }
    },

    async createConversation(title?: string) {
      if (!_api) return;
      const res = await _api.chat_create_conversation({ title: title ?? 'New Chat' });
      if (res.status === 'success' && res.conversation) {
        this.conversations.unshift(res.conversation);
        this.activeConversationId = res.conversation.id;
        this.messages = [];
        this.streamingText = '';
      }
    },

    async switchConversation(id: string) {
      if (!_api) return;
      this.activeConversationId = id;
      this.streamingText = '';
      const res = await _api.chat_get_messages({ conversation_id: id });
      if (res.status === 'success' && res.messages) {
        this.messages = res.messages;
      }
    },

    async deleteConversation(id: string) {
      if (!_api) return;
      await _api.chat_delete_conversation({ conversation_id: id });
      this.conversations = this.conversations.filter(c => c.id !== id);
      if (this.activeConversationId === id) {
        this.activeConversationId = this.conversations[0]?.id ?? null;
        if (this.activeConversationId) {
          await this.switchConversation(this.activeConversationId);
        } else {
          this.messages = [];
        }
      }
    },

    async sendMessage(content: string, displayText?: string) {
      if (!_api || !this.activeConversationId || !content.trim()) return;

      const inferenceStore = useInferenceStore();
      const generationContext: Record<string, any> = {
        prompt: inferenceStore.params.prompt,
        width: inferenceStore.params.width,
        height: inferenceStore.params.height,
        steps: inferenceStore.params.steps,
        cfg_scale: inferenceStore.params.cfg_scale,
        seed: inferenceStore.params.seed,
        sampler: inferenceStore.params.sampler,
        scheduler: inferenceStore.params.scheduler,
      };
      if (inferenceStore.selectedBundle) {
        generationContext.model = inferenceStore.selectedBundle.label;
      }

      // Optimistically add user message (show displayText in the bubble if provided)
      const userMsg: ConversationMessage = {
        id: crypto.randomUUID(),
        conversation_id: this.activeConversationId,
        role: 'user',
        content,
        display_text: displayText ?? null,
        image_paths: this.pendingImagePaths.length ? JSON.stringify(this.pendingImagePaths) : null,
        tool_calls: null,
        created_at: Math.floor(Date.now() / 1000),
      };
      this.messages.push(userMsg);

      const imagePaths = this.pendingImagePaths.length ? [...this.pendingImagePaths] : undefined;
      this.pendingImagePaths = [];

      this.isStreaming = true;
      this.streamingText = '';

      await _api.chat_send_message({
        conversation_id: this.activeConversationId,
        content,
        image_paths: imagePaths,
        generation_context: generationContext,
        display_text: displayText,
      });

      this.startChatPolling();
    },

    startChatPolling() {
      if (_pollTimer) return;
      _pollTimer = setInterval(() => this.pollChatEvents(), 200);
    },

    stopChatPolling() {
      if (_pollTimer) {
        clearInterval(_pollTimer);
        _pollTimer = null;
      }
    },

    async pollChatEvents() {
      if (!_api) return;
      try {
        const res = await _api.poll_chat_events();
        if (res.status === 'success' && res.events?.length) {
          this.processChatEvents(res.events);
        }
      } catch {
        // Silently ignore poll errors
      }
    },

    processChatEvents(events: ChatEvent[]) {
      const inferenceStore = useInferenceStore();

      for (const event of events) {
        switch (event.type) {
          case 'chat_chunk':
            this.streamingText += event.data.text ?? '';
            break;

          case 'chat_tool_use': {
            const toolName = event.data.tool_name;
            const toolInput = event.data.tool_input ?? {};

            if (toolName === 'update_prompt' && toolInput.prompt) {
              inferenceStore.params.prompt = toolInput.prompt;
            } else if (toolName === 'update_generation_settings') {
              if (toolInput.width !== undefined) inferenceStore.params.width = toolInput.width;
              if (toolInput.height !== undefined) inferenceStore.params.height = toolInput.height;
              if (toolInput.steps !== undefined) inferenceStore.params.steps = toolInput.steps;
              if (toolInput.cfg_scale !== undefined) inferenceStore.params.cfg_scale = toolInput.cfg_scale;
              if (toolInput.seed !== undefined) inferenceStore.params.seed = toolInput.seed;
            }
            break;
          }

          case 'chat_complete':
            this.isStreaming = false;
            this.streamingText = '';
            this.stopChatPolling();
            this.reloadMessages();
            break;

          case 'chat_error': {
            this.isStreaming = false;
            this.streamingText = '';
            this.stopChatPolling();
            const raw = event.data.error ?? 'An unknown error occurred.';
            const msgMatch = raw.match(/'message':\s*'([^']+)'/);
            this.messages.push({
              id: crypto.randomUUID(),
              conversation_id: this.activeConversationId ?? '',
              role: 'error',
              content: msgMatch ? msgMatch[1] : raw,
              display_text: null,
              image_paths: null,
              tool_calls: null,
              created_at: Math.floor(Date.now() / 1000),
            });
            break;
          }
        }
      }
    },

    async reloadMessages() {
      if (!_api || !this.activeConversationId) return;
      const res = await _api.chat_get_messages({ conversation_id: this.activeConversationId });
      if (res.status === 'success' && res.messages) {
        this.messages = res.messages;
      }
    },

    addPendingImage(path: string) {
      this.pendingImagePaths.push(path);
    },

    removePendingImage(index: number) {
      this.pendingImagePaths.splice(index, 1);
    },
  },
});

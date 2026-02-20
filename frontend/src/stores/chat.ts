import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { Conversation, ConversationMessage, ChatEvent } from '@/types/inference';
import { useInferenceStore } from '@/stores/inference';

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
    async checkConfigured() {
      const api = await getApiAsync();
      const res = await api.chat_is_configured();
      if (res.status === 'success') {
        this.isConfigured = res.configured ?? false;
      }
    },

    async setApiKey(apiKey: string) {
      const api = await getApiAsync();
      const res = await api.chat_set_api_key({ api_key: apiKey });
      if (res.status === 'success') {
        this.isConfigured = true;
      }
    },

    async loadConversations() {
      const api = await getApiAsync();
      const res = await api.chat_get_conversations();
      if (res.status === 'success' && res.conversations) {
        this.conversations = res.conversations;
      }
    },

    async createConversation(title?: string) {
      const api = await getApiAsync();
      const res = await api.chat_create_conversation({ title: title ?? 'New Chat' });
      if (res.status === 'success' && res.conversation) {
        this.conversations.unshift(res.conversation);
        this.activeConversationId = res.conversation.id;
        this.messages = [];
        this.streamingText = '';
      }
    },

    async switchConversation(id: string) {
      const api = await getApiAsync();
      this.activeConversationId = id;
      this.streamingText = '';
      const res = await api.chat_get_messages({ conversation_id: id });
      if (res.status === 'success' && res.messages) {
        this.messages = res.messages;
      }
    },

    async deleteConversation(id: string) {
      const api = await getApiAsync();
      await api.chat_delete_conversation({ conversation_id: id });
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
      if (!this.activeConversationId || !content.trim()) return;
      const api = await getApiAsync();

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

      try {
        await api.chat_send_message({
          conversation_id: this.activeConversationId,
          content,
          image_paths: imagePaths,
          generation_context: generationContext,
          display_text: displayText,
        });

        this.startChatPolling();
      } catch (e: any) {
        this.isStreaming = false;
        this.messages.push({
          id: crypto.randomUUID(),
          conversation_id: this.activeConversationId ?? '',
          role: 'error',
          content: e.message ?? 'Failed to send message',
          display_text: null,
          image_paths: null,
          tool_calls: null,
          created_at: Math.floor(Date.now() / 1000),
        });
      }
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
      const api = await getApiAsync();
      try {
        const res = await api.poll_chat_events();
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
              inferenceStore.applyParams({ prompt: toolInput.prompt });
            } else if (toolName === 'update_generation_settings') {
              const updates: Record<string, any> = {};
              if (toolInput.width !== undefined) updates.width = toolInput.width;
              if (toolInput.height !== undefined) updates.height = toolInput.height;
              if (toolInput.steps !== undefined) updates.steps = toolInput.steps;
              if (toolInput.cfg_scale !== undefined) updates.cfg_scale = toolInput.cfg_scale;
              if (toolInput.seed !== undefined) updates.seed = toolInput.seed;
              inferenceStore.applyParams(updates);
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
      if (!this.activeConversationId) return;
      const api = await getApiAsync();
      const res = await api.chat_get_messages({ conversation_id: this.activeConversationId });
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

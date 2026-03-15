import { describe, it, expect, beforeEach } from 'vitest';
import { makeDb } from './helpers/db';
import { createChatService } from '../src/main/services/chat';
import type Database from 'better-sqlite3';

let db: Database.Database;
let chat: ReturnType<typeof createChatService>;

beforeEach(() => {
  db = makeDb();
  chat = createChatService(db);
});

describe('isConfigured', () => {
  it('returns false when no API key is stored', () => {
    const res = chat.isConfigured();
    expect(res.status).toBe('success');
    expect(res.configured).toBe(false);
  });

  it('returns true after a key is stored', () => {
    db.prepare("INSERT INTO settings (key, value) VALUES ('CLAUDE_API_KEY', 'sk-ant-test')").run();
    const res = chat.isConfigured();
    expect(res.configured).toBe(true);
  });
});

describe('setApiKey', () => {
  it('persists CLAUDE_API_KEY for claude provider', () => {
    const res = chat.setApiKey({ apiKey: 'sk-ant-test', provider: 'claude' });
    expect(res.status).toBe('success');
    const row = db.prepare("SELECT value FROM settings WHERE key = 'CLAUDE_API_KEY'").get() as { value: string } | null;
    expect(row?.value).toBe('sk-ant-test');
  });

  it('defaults to CLAUDE_API_KEY when provider is omitted', () => {
    chat.setApiKey({ apiKey: 'sk-ant-test' });
    const row = db.prepare("SELECT value FROM settings WHERE key = 'CLAUDE_API_KEY'").get() as { value: string } | null;
    expect(row?.value).toBe('sk-ant-test');
  });

  it('persists PERPLEXITY_API_KEY for perplexity provider', () => {
    chat.setApiKey({ apiKey: 'pplx-test', provider: 'perplexity' });
    const row = db.prepare("SELECT value FROM settings WHERE key = 'PERPLEXITY_API_KEY'").get() as { value: string } | null;
    expect(row?.value).toBe('pplx-test');
  });

  it('updates an existing key (upsert)', () => {
    chat.setApiKey({ apiKey: 'sk-ant-first' });
    chat.setApiKey({ apiKey: 'sk-ant-second' });
    const row = db.prepare("SELECT value FROM settings WHERE key = 'CLAUDE_API_KEY'").get() as { value: string } | null;
    expect(row?.value).toBe('sk-ant-second');
  });

  it('makes isConfigured return true afterwards', () => {
    chat.setApiKey({ apiKey: 'sk-ant-test' });
    expect(chat.isConfigured().configured).toBe(true);
  });
});

describe('createConversation', () => {
  it('returns the created conversation', () => {
    const res = chat.createConversation({ title: 'My Chat' });
    expect(res.status).toBe('success');
    expect(res.conversation).toBeTruthy();
    const conv = res.conversation as Record<string, unknown>;
    expect(conv.title).toBe('My Chat');
    expect(typeof conv.id).toBe('string');
  });

  it('uses "New Chat" as the default title', () => {
    const res = chat.createConversation({});
    const conv = res.conversation as Record<string, unknown>;
    expect(conv.title).toBe('New Chat');
  });

  it('persists the conversation to the database', () => {
    chat.createConversation({ title: 'Persisted' });
    const row = db.prepare('SELECT * FROM conversations WHERE title = ?').get('Persisted');
    expect(row).toBeTruthy();
  });
});

describe('getConversations', () => {
  it('returns empty array when no conversations exist', () => {
    const res = chat.getConversations();
    expect(res.status).toBe('success');
    expect(res.conversations).toEqual([]);
  });

  it('returns all conversations', () => {
    chat.createConversation({ title: 'A' });
    chat.createConversation({ title: 'B' });
    const res = chat.getConversations();
    expect((res.conversations as unknown[]).length).toBe(2);
  });
});

describe('getMessages', () => {
  it('returns empty array for a conversation with no messages', () => {
    const { conversation } = chat.createConversation({ title: 'Empty' });
    const conv = conversation as Record<string, unknown>;
    const res = chat.getMessages({ conversationId: conv.id as string });
    expect(res.status).toBe('success');
    expect(res.messages).toEqual([]);
  });
});

describe('deleteConversation', () => {
  it('removes the conversation from the database', () => {
    const { conversation } = chat.createConversation({ title: 'To Delete' });
    const conv = conversation as Record<string, unknown>;
    chat.deleteConversation({ conversationId: conv.id as string });
    const row = db.prepare('SELECT * FROM conversations WHERE id = ?').get(conv.id);
    expect(row).toBeUndefined();
  });

  it('returns success even for a non-existent id', () => {
    const res = chat.deleteConversation({ conversationId: 'does-not-exist' });
    expect(res.status).toBe('success');
  });
});

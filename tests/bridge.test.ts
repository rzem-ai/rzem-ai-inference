import { describe, it, expect } from 'vitest';
import { snakeToCamel, camelToSnake, keysToCamel, keysToSnake } from '../src/mainview/bridge';

describe('snakeToCamel', () => {
  it('converts simple snake_case', () => {
    expect(snakeToCamel('hello_world')).toBe('helloWorld');
  });

  it('converts multi-segment snake_case', () => {
    expect(snakeToCamel('fal_aspect_ratio')).toBe('falAspectRatio');
  });

  it('leaves already-camel untouched', () => {
    expect(snakeToCamel('helloWorld')).toBe('helloWorld');
  });

  it('leaves plain words untouched', () => {
    expect(snakeToCamel('status')).toBe('status');
  });

  // Covers the exact conversions the bridge must get right
  it('converts api_key → apiKey', () => {
    expect(snakeToCamel('api_key')).toBe('apiKey');
  });

  it('converts filter_names → filterNames', () => {
    expect(snakeToCamel('filter_names')).toBe('filterNames');
  });

  it('converts style_id → styleId', () => {
    expect(snakeToCamel('style_id')).toBe('styleId');
  });

  it('converts conversation_id → conversationId', () => {
    expect(snakeToCamel('conversation_id')).toBe('conversationId');
  });
});

describe('camelToSnake', () => {
  it('converts simple camelCase', () => {
    expect(camelToSnake('helloWorld')).toBe('hello_world');
  });

  it('converts multi-segment camelCase', () => {
    expect(camelToSnake('falAspectRatio')).toBe('fal_aspect_ratio');
  });

  it('leaves plain words untouched', () => {
    expect(camelToSnake('status')).toBe('status');
  });

  it('is the inverse of snakeToCamel for simple cases', () => {
    const snake = 'style_data';
    expect(camelToSnake(snakeToCamel(snake))).toBe(snake);
  });

  // IPC response keys the bridge must convert back for the frontend
  it('converts styleData → style_data', () => {
    expect(camelToSnake('styleData')).toBe('style_data');
  });

  it('converts bundleId → bundle_id', () => {
    expect(camelToSnake('bundleId')).toBe('bundle_id');
  });
});

describe('keysToCamel', () => {
  it('converts top-level keys', () => {
    expect(keysToCamel({ api_key: 'sk-test', provider: 'claude' })).toEqual({
      apiKey: 'sk-test',
      provider: 'claude',
    });
  });

  it('converts nested object keys deeply', () => {
    expect(keysToCamel({ outer_key: { inner_key: 'value' } })).toEqual({
      outerKey: { innerKey: 'value' },
    });
  });

  it('converts keys inside arrays', () => {
    expect(keysToCamel({ items: [{ style_id: '1' }, { style_id: '2' }] })).toEqual({
      items: [{ styleId: '1' }, { styleId: '2' }],
    });
  });

  it('does not mutate primitive values', () => {
    expect(keysToCamel({ count: 42, active: true, label: 'test' })).toEqual({
      count: 42,
      active: true,
      label: 'test',
    });
  });

  it('passes through null/undefined values', () => {
    expect(keysToCamel({ some_key: null })).toEqual({ someKey: null });
  });

  it('returns non-objects as-is', () => {
    expect(keysToCamel('string')).toBe('string');
    expect(keysToCamel(42)).toBe(42);
    expect(keysToCamel(null)).toBe(null);
  });

  // Simulates the exact param conversion for chat_set_api_key
  it('converts { api_key, provider } for chatSetApiKey call', () => {
    const params = { api_key: 'sk-ant-test', provider: 'claude' };
    expect(keysToCamel(params)).toEqual({ apiKey: 'sk-ant-test', provider: 'claude' });
  });

  // Simulates applyStyleReview changes array
  it('converts style_id inside nested changes array', () => {
    const params = { changes: [{ style_id: 'abc', name: 'New Name' }] };
    expect(keysToCamel(params)).toEqual({ changes: [{ styleId: 'abc', name: 'New Name' }] });
  });
});

describe('keysToSnake', () => {
  it('converts top-level camelCase keys', () => {
    expect(keysToSnake({ styleData: { name: 'test' } })).toEqual({
      style_data: { name: 'test' },
    });
  });

  it('converts nested keys deeply', () => {
    expect(keysToSnake({ outerKey: { innerKey: 'value' } })).toEqual({
      outer_key: { inner_key: 'value' },
    });
  });

  it('converts keys inside arrays', () => {
    expect(keysToSnake({ bundleTypes: [{ transformerType: 'flux1_dev' }] })).toEqual({
      bundle_types: [{ transformer_type: 'flux1_dev' }],
    });
  });

  it('is the inverse of keysToCamel for simple cases', () => {
    const original = { some_key: { nested_val: 42 } };
    expect(keysToSnake(keysToCamel(original))).toEqual(original);
  });
});

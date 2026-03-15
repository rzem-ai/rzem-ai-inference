import { describe, it, expect } from 'vitest';
import { batchParseData, batchRenderTemplate } from '../src/main/services/batch';

describe('batchParseData', () => {
  it('returns error for empty input', () => {
    expect(batchParseData('').status).toBe('error');
    expect(batchParseData('   ').status).toBe('error');
  });

  it('returns error for header-only (no data rows)', () => {
    const res = batchParseData('prompt,style');
    expect(res.status).toBe('error');
  });

  it('parses a simple CSV', () => {
    const csv = 'prompt,style\na sunset,cinematic\na forest,painterly';
    const res = batchParseData(csv);
    expect(res.status).toBe('success');
    expect(res.columns).toEqual(['prompt', 'style']);
    expect(res.rows).toHaveLength(2);
    expect(res.rows![0]).toEqual({ prompt: 'a sunset', style: 'cinematic' });
    expect(res.rows![1]).toEqual({ prompt: 'a forest', style: 'painterly' });
  });

  it('handles quoted values with commas inside', () => {
    const csv = 'prompt\n"a cat, a dog"';
    const res = batchParseData(csv);
    expect(res.status).toBe('success');
    expect(res.rows![0]!['prompt']).toBe('a cat, a dog');
  });

  it('handles escaped double-quotes inside quoted fields', () => {
    const csv = 'prompt\n"she said ""hello"""';
    const res = batchParseData(csv);
    expect(res.status).toBe('success');
    expect(res.rows![0]!['prompt']).toBe('she said "hello"');
  });

  it('skips blank lines in data section', () => {
    const csv = 'prompt\nfirst\n\nthird';
    const res = batchParseData(csv);
    expect(res.status).toBe('success');
    expect(res.rows).toHaveLength(2);
  });

  it('fills missing columns with empty string', () => {
    const csv = 'a,b,c\n1,2';
    const res = batchParseData(csv);
    expect(res.status).toBe('success');
    expect(res.rows![0]!['c']).toBe('');
  });
});

describe('batchRenderTemplate', () => {
  it('substitutes a single variable', () => {
    const rows = [{ subject: 'a cat' }];
    const res = batchRenderTemplate('photo of {{ subject }}', rows);
    expect(res.status).toBe('success');
    expect(res.rendered![0]).toBe('photo of a cat');
  });

  it('substitutes multiple variables', () => {
    const rows = [{ subject: 'a dog', style: 'oil painting' }];
    const res = batchRenderTemplate('{{subject}} in {{style}} style', rows);
    expect(res.rendered![0]).toBe('a dog in oil painting style');
  });

  it('leaves unknown variables as-is', () => {
    const rows = [{ prompt: 'test' }];
    const res = batchRenderTemplate('{{prompt}} with {{unknown}}', rows);
    expect(res.rendered![0]).toBe('test with {{unknown}}');
  });

  it('renders multiple rows independently', () => {
    const rows = [{ p: 'cats' }, { p: 'dogs' }];
    const res = batchRenderTemplate('{{p}} photo', rows);
    expect(res.rendered).toEqual(['cats photo', 'dogs photo']);
  });

  it('handles whitespace inside template braces', () => {
    const rows = [{ item: 'sunset' }];
    const res = batchRenderTemplate('a {{ item }} scene', rows);
    expect(res.rendered![0]).toBe('a sunset scene');
  });

  it('returns empty rendered array for empty rows', () => {
    const res = batchRenderTemplate('{{p}}', []);
    expect(res.status).toBe('success');
    expect(res.rendered).toHaveLength(0);
    expect(res.errors).toHaveLength(0);
  });
});

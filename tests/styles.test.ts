import { describe, it, expect, beforeEach } from 'vitest';
import { makeDb } from './helpers/db';
import { createStylesHandlers } from '../src/main/services/styles';
import type Database from 'better-sqlite3';

let db: Database.Database;
let styles: ReturnType<typeof createStylesHandlers>;

const STYLES_DIR = '/tmp/test-styles';

function makeStyle(overrides: Record<string, unknown> = {}) {
  return styles.createStyle({
    id: crypto.randomUUID(),
    name: 'Test Style',
    promptTemplate: 'a {prompt} in test style',
    description: 'A test style',
    category: 'test',
    ...overrides,
  });
}

beforeEach(() => {
  db = makeDb();
  styles = createStylesHandlers(db, STYLES_DIR);
});

describe('createStyle', () => {
  it('returns the created style', () => {
    const res = makeStyle({ name: 'Cinematic' });
    expect(res.status).toBe('success');
    const style = res.style as Record<string, unknown>;
    expect(style.name).toBe('Cinematic');
    expect(style.prompt_template).toBe('a {prompt} in test style');
  });

  it('persists to the database', () => {
    const res = makeStyle({ name: 'Stored' });
    const style = res.style as Record<string, unknown>;
    const row = db.prepare('SELECT * FROM styles WHERE id = ?').get(style.id);
    expect(row).toBeTruthy();
  });

  it('stores optional fields as null when omitted', () => {
    const res = styles.createStyle({
      id: 'minimal-id',
      name: 'Minimal',
      promptTemplate: '{prompt}',
    });
    const style = res.style as Record<string, unknown>;
    expect(style.negative_prompt).toBeNull();
    expect(style.category).toBeNull();
  });
});

describe('getStyle', () => {
  it('returns error for unknown id', () => {
    const res = styles.getStyle({ styleId: 'does-not-exist' });
    expect(res.status).toBe('error');
  });

  it('returns the style with empty loras, tags, and examples', () => {
    const created = makeStyle();
    const id = (created.style as Record<string, unknown>).id as string;
    const res = styles.getStyle({ styleId: id });
    expect(res.status).toBe('success');
    expect(res.loras).toEqual([]);
    expect(res.tags).toEqual([]);
    expect(res.examples).toEqual([]);
  });
});

describe('getStyles', () => {
  it('returns empty array when no styles exist', () => {
    const res = styles.getStyles({});
    expect(res.status).toBe('success');
    expect(res.styles).toEqual([]);
  });

  it('returns all styles when no filters applied', () => {
    makeStyle({ name: 'A' });
    makeStyle({ name: 'B' });
    const res = styles.getStyles({});
    expect((res.styles as unknown[]).length).toBe(2);
  });

  it('filters by category', () => {
    makeStyle({ name: 'Portrait', category: 'portrait' });
    makeStyle({ name: 'Landscape', category: 'landscape' });
    const res = styles.getStyles({ category: 'portrait' });
    expect((res.styles as unknown[]).length).toBe(1);
    expect((res.styles as Record<string, unknown>[])[0]!.name).toBe('Portrait');
  });

  it('filters by search term (name match)', () => {
    makeStyle({ name: 'Cinematic Dark' });
    makeStyle({ name: 'Bright Pastel' });
    const res = styles.getStyles({ search: 'dark' });
    expect((res.styles as unknown[]).length).toBe(1);
  });

  it('filters favorites only', () => {
    const created = makeStyle({ name: 'Fav' });
    const id = (created.style as Record<string, unknown>).id as string;
    makeStyle({ name: 'Not Fav' });
    db.prepare('UPDATE styles SET is_favorite = 1 WHERE id = ?').run(id);

    const res = styles.getStyles({ favoritesOnly: true });
    expect((res.styles as unknown[]).length).toBe(1);
    expect((res.styles as Record<string, unknown>[])[0]!.name).toBe('Fav');
  });
});

describe('updateStyle', () => {
  it('updates the name', () => {
    const created = makeStyle({ name: 'Old Name' });
    const id = (created.style as Record<string, unknown>).id as string;
    const res = styles.updateStyle({ styleId: id, name: 'New Name' });
    expect(res.status).toBe('success');
    expect((res.style as Record<string, unknown>).name).toBe('New Name');
  });

  it('updates only the provided fields', () => {
    const created = makeStyle({ name: 'Keep Name', category: 'keep-cat' });
    const id = (created.style as Record<string, unknown>).id as string;
    styles.updateStyle({ styleId: id, category: 'updated-cat' });
    const row = db.prepare('SELECT * FROM styles WHERE id = ?').get(id) as Record<string, unknown>;
    expect(row.name).toBe('Keep Name');
    expect(row.category).toBe('updated-cat');
  });

  it('returns error when no fields are provided', () => {
    const created = makeStyle();
    const id = (created.style as Record<string, unknown>).id as string;
    const res = styles.updateStyle({ styleId: id });
    expect(res.status).toBe('error');
  });
});

describe('deleteStyle', () => {
  it('removes the style from the database', () => {
    const created = makeStyle();
    const id = (created.style as Record<string, unknown>).id as string;
    styles.deleteStyle({ styleId: id });
    const row = db.prepare('SELECT * FROM styles WHERE id = ?').get(id);
    expect(row).toBeUndefined();
  });

  it('returns success even for a non-existent id', () => {
    const res = styles.deleteStyle({ styleId: 'ghost' });
    expect(res.status).toBe('success');
  });
});

describe('toggleStyleFavorite', () => {
  it('sets is_favorite to 1 on first toggle', () => {
    const created = makeStyle();
    const id = (created.style as Record<string, unknown>).id as string;
    const res = styles.toggleStyleFavorite({ styleId: id });
    expect((res.style as Record<string, unknown>).is_favorite).toBe(1);
  });

  it('toggles back to 0 on second call', () => {
    const created = makeStyle();
    const id = (created.style as Record<string, unknown>).id as string;
    styles.toggleStyleFavorite({ styleId: id });
    const res = styles.toggleStyleFavorite({ styleId: id });
    expect((res.style as Record<string, unknown>).is_favorite).toBe(0);
  });
});

describe('getStyleCategories', () => {
  it('returns distinct categories', () => {
    makeStyle({ category: 'portrait' });
    makeStyle({ category: 'landscape' });
    makeStyle({ category: 'portrait' }); // duplicate
    const res = styles.getStyleCategories();
    expect(res.status).toBe('success');
    expect(res.categories).toEqual(['landscape', 'portrait']);
  });

  it('excludes null categories', () => {
    styles.createStyle({ id: 'no-cat', name: 'No Cat', promptTemplate: '{prompt}' });
    const res = styles.getStyleCategories();
    expect((res.categories as string[]).includes('')).toBe(false);
  });
});

describe('createStyleExample / deleteStyleExample', () => {
  it('creates and retrieves an example', () => {
    const created = makeStyle();
    const styleId = (created.style as Record<string, unknown>).id as string;
    const res = styles.createStyleExample({ styleId, prompt: 'a test prompt' });
    expect(res.status).toBe('success');
    const example = res.example as Record<string, unknown>;
    expect(example.entity_id).toBe(styleId);
    const content = JSON.parse(example.content as string);
    expect(content.prompt).toBe('a test prompt');
  });

  it('deletes an example by id', () => {
    const created = makeStyle();
    const styleId = (created.style as Record<string, unknown>).id as string;
    const { example } = styles.createStyleExample({ styleId, prompt: 'delete me' });
    const exampleId = (example as Record<string, unknown>).id as string;
    styles.deleteStyleExample({ exampleId });
    const row = db.prepare('SELECT * FROM examples WHERE id = ?').get(exampleId);
    expect(row).toBeUndefined();
  });
});

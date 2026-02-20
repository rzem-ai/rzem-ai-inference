import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { Style, StyleLoRA, StyleExample, LoRA, Tag } from '@/types/inference';

export const useStylesStore = defineStore('styles', {
  state: () => ({
    styles: [] as Style[],
    categories: [] as string[],
    tags: [] as Tag[],
    loras: [] as LoRA[],
    loading: false,

    // Filters
    currentCategory: null as string | null,
    currentTagId: null as number | null,
    searchQuery: '',
    favoritesOnly: false,

    // Sort
    sortBy: 'updated_at' as string,
    sortOrder: 'asc' as 'asc' | 'desc',

    // Editor state
    editorStyle: null as Style | null,
    editorLoras: [] as StyleLoRA[],
    editorTags: [] as Tag[],
    editorExamples: [] as StyleExample[],
  }),

  getters: {},

  actions: {
    async loadStyles(reset = true) {
      const api = await getApiAsync();
      this.loading = true;
      try {
        const res = await api.get_styles({
          category: this.currentCategory ?? undefined,
          tag_id: this.currentTagId ?? undefined,
          search: this.searchQuery || undefined,
          favorites_only: this.favoritesOnly,
          sort_by: this.sortBy,
          sort_order: this.sortOrder,
        });

        if (res.status === 'success') {
          this.styles = res.styles ?? [];
        }
      } finally {
        this.loading = false;
      }
    },

    async loadCategories() {
      const api = await getApiAsync();
      const res = await api.get_style_categories();
      if (res.status === 'success') {
        this.categories = res.categories ?? [];
      }
    },

    async loadTags() {
      const api = await getApiAsync();
      const res = await api.get_tags();
      if (res.status === 'success') {
        this.tags = (res.tags ?? []).filter(t => t.category === 'style');
      }
    },

    async loadLoras() {
      const api = await getApiAsync();
      const res = await api.get_loras();
      if (res.status === 'success') {
        this.loras = res.loras ?? [];
      }
    },

    async browseLoras(): Promise<LoRA[]> {
      const api = await getApiAsync();
      const res = await api.browse_lora_files();
      if (res.status === 'success' && res.loras?.length) {
        this.loras = [...this.loras, ...res.loras];
        return res.loras;
      }
      return [];
    },

    async registerLoraPaths(paths: string[]): Promise<LoRA[]> {
      if (!paths.length) return [];
      const api = await getApiAsync();
      const created: LoRA[] = [];
      for (const path of paths) {
        const name = path.split('/').pop()?.replace(/\.[^.]+$/, '') || 'Unknown';
        const res = await api.create_lora({ id: crypto.randomUUID(), name, path });
        if (res.status === 'success' && res.lora) {
          created.push(res.lora);
        }
      }
      if (created.length) {
        this.loras = [...this.loras, ...created];
      }
      return created;
    },

    // ── CRUD ──

    async createStyle(data: {
      id: string;
      name: string;
      promptTemplate: string;
      description?: string;
      negativePrompt?: string;
      category?: string;
      thumbnailPath?: string;
    }) {
      const api = await getApiAsync();
      const res = await api.create_style({
        id: data.id,
        name: data.name,
        prompt_template: data.promptTemplate,
        description: data.description,
        negative_prompt: data.negativePrompt,
        category: data.category,
        thumbnail_path: data.thumbnailPath,
      });
      if (res.status === 'success') {
        await this.loadStyles();
        await this.loadCategories();
      }
      return res;
    },

    async updateStyle(styleId: string, data: Record<string, any>) {
      const api = await getApiAsync();
      const res = await api.update_style({ style_id: styleId, ...data });
      if (res.status === 'success') {
        await this.loadStyles();
        await this.loadCategories();
      }
      return res;
    },

    async deleteStyle(styleId: string) {
      const api = await getApiAsync();
      const res = await api.delete_style({ style_id: styleId });
      if (res.status === 'success') {
        this.styles = this.styles.filter(s => s.id !== styleId);
      }
      return res;
    },

    async toggleFavorite(styleId: string) {
      const api = await getApiAsync();
      const res = await api.toggle_style_favorite({ style_id: styleId });
      if (res.status === 'success' && res.style) {
        const idx = this.styles.findIndex(s => s.id === styleId);
        if (idx !== -1) {
          this.styles[idx] = res.style;
        }
      }
    },

    // ── Editor ──

    async loadStyleForEditor(styleId: string) {
      const api = await getApiAsync();
      const res = await api.get_style({ style_id: styleId });
      if (res.status === 'success') {
        this.editorStyle = res.style ?? null;
        this.editorLoras = res.loras ?? [];
        this.editorTags = res.tags ?? [];
        this.editorExamples = res.examples ?? [];
      }
      return res;
    },

    clearEditor() {
      this.editorStyle = null;
      this.editorLoras = [];
      this.editorTags = [];
      this.editorExamples = [];
    },

    async saveStyleLoras(styleId: string, loras: Array<{ lora_id: string; strength: number; priority?: number }>) {
      const api = await getApiAsync();
      const res = await api.set_style_loras({ style_id: styleId, loras });
      if (res.status === 'success') {
        this.editorLoras = res.loras ?? [];
      }
      return res;
    },

    async createTag(name: string): Promise<Tag | null> {
      const api = await getApiAsync();
      const res = await api.create_tag({ name, category: 'style' });
      if (res.status === 'success' && res.tag) {
        this.tags.push(res.tag);
        return res.tag;
      }
      return null;
    },

    async saveStyleTags(styleId: string, tagIds: number[]) {
      const api = await getApiAsync();
      const res = await api.set_style_tags({ style_id: styleId, tag_ids: tagIds });
      if (res.status === 'success') {
        this.editorTags = res.tags ?? [];
      }
      return res;
    },

    // ── Examples ──

    async addExample(styleId: string, data: {
      prompt: string;
      imagePath?: string;
      seed?: number;
      width?: number;
      height?: number;
      steps?: number;
      cfgScale?: number;
    }) {
      const api = await getApiAsync();
      const res = await api.create_style_example({
        style_id: styleId,
        prompt: data.prompt,
        image_path: data.imagePath,
        seed: data.seed,
        width: data.width,
        height: data.height,
        steps: data.steps,
        cfg_scale: data.cfgScale,
      });
      if (res.status === 'success' && res.example) {
        this.editorExamples.push(res.example);
      }
      return res;
    },

    async removeExample(exampleId: string) {
      const api = await getApiAsync();
      const res = await api.delete_style_example({ example_id: exampleId });
      if (res.status === 'success') {
        this.editorExamples = this.editorExamples.filter(e => e.id !== exampleId);
      }
      return res;
    },

    // ── Import ──

    async importCivitaiMetadata() {
      const api = await getApiAsync();
      const res = await api.browse_and_import_metadata();
      if (res.status === 'success' && res.styles?.length) {
        await this.loadStyles();
        await this.loadCategories();
        await this.loadTags();
      }
      return res;
    },

    // ── State setters ──

    setFavoritesOnly(val: boolean) {
      this.favoritesOnly = val;
    },

    setCurrentTagId(id: number | null) {
      this.currentTagId = id;
    },

    setCurrentCategory(category: string | null) {
      this.currentCategory = category;
    },

    // ── Filters ──

    async filterByCategory(category: string | null) {
      this.currentCategory = category;
      this.favoritesOnly = false;
      this.currentTagId = null;
      await this.loadStyles();
    },

    async filterByTag(tagId: number | null) {
      this.currentTagId = tagId;
      this.favoritesOnly = false;
      this.currentCategory = null;
      await this.loadStyles();
    },

    async searchStyles(query: string) {
      this.searchQuery = query;
      await this.loadStyles();
    },

    async toggleFavoritesFilter() {
      this.favoritesOnly = !this.favoritesOnly;
      this.currentCategory = null;
      this.currentTagId = null;
      await this.loadStyles();
    },

    async setSort(sortBy: string, sortOrder?: 'asc' | 'desc') {
      this.sortBy = sortBy;
      if (sortOrder) {
        this.sortOrder = sortOrder;
      }
      await this.loadStyles();
    },

    async toggleSortOrder() {
      this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
      await this.loadStyles();
    },
  },
});

import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { Style, StyleLoRA, StyleExample, LoRA, Tag, Filter, GeneratedStyleData } from '@/types/inference';

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
    sortOrder: 'desc' as 'asc' | 'desc',

    // Import progress
    importing: false,
    importCurrent: 0,
    importTotal: 0,
    importMessage: '',

    // Editor state
    editorStyle: null as Style | null,
    editorLoras: [] as StyleLoRA[],
    editorTags: [] as Tag[],
    editorExamples: [] as StyleExample[],

    // Builder state
    builderFilters: [] as Filter[],
    builderSelected: [] as Filter[],
    generatedStyle: null as GeneratedStyleData | null,
    builderGenerating: false,
    builderError: '',

    // Filter thumbnails
    availableThumbnails: [] as string[],
    thumbnailCache: {} as Record<string, string>,
  }),

  getters: {},

  actions: {
    /**
     * Initialize the store: wait for the pywebview bridge, then load all data.
     * Safe to call from multiple components -- only runs once.
     */
    async init() {
      Promise.all([this.loadStyles(), this.loadCategories(), this.loadLoras(), this.loadTags()]).then(() => {});
    },

    async loadStyles(reset = true) {
      const api = await getApiAsync();
      this.loading = true;
      console.log('loadStyles: ', reset);
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
      } catch (e: any) {
        console.error('[gallery] Failed to load images:', e);
      } finally {
        console.log('loadImages: ', reset);
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
        this.tags = (res.tags ?? []).filter((t) => t.category === 'style');
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
        const name =
          path
            .split('/')
            .pop()
            ?.replace(/\.[^.]+$/, '') || 'Unknown';
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
      bundleId?: string;
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
        bundle_id: data.bundleId,
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
        this.styles = this.styles.filter((s) => s.id !== styleId);
      }
      return res;
    },

    async toggleFavorite(styleId: string) {
      const api = await getApiAsync();
      const res = await api.toggle_style_favorite({ style_id: styleId });
      if (res.status === 'success' && res.style) {
        const idx = this.styles.findIndex((s) => s.id === styleId);
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

    async addExample(
      styleId: string,
      data: {
        prompt: string;
        imagePath?: string;
        seed?: number;
        width?: number;
        height?: number;
        steps?: number;
        cfgScale?: number;
      },
    ) {
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
        this.editorExamples = this.editorExamples.filter((e) => e.id !== exampleId);
      }
      return res;
    },

    // ── Import ──

    async importCivitaiMetadata() {
      const api = await getApiAsync();

      // Step 1: Browse for files
      const browse = await api.browse_metadata_files();
      const paths = browse.paths ?? [];
      if (!paths.length) return browse;

      // Step 2: Import each file with progress tracking
      this.importing = true;
      this.importTotal = paths.length;
      this.importCurrent = 0;
      this.importMessage = 'Starting import...';

      let imported = 0;
      for (const filePath of paths) {
        const name = filePath.split('/').pop()?.replace('.metadata.json', '') ?? 'style';
        this.importCurrent++;
        this.importMessage = `Importing "${name}" (${this.importCurrent}/${this.importTotal})...`;

        const res = await api.import_civitai_metadata({ file_path: filePath });
        if (res.status === 'success') imported++;
      }

      this.importMessage = 'Refreshing...';
      if (imported > 0) {
        await this.loadStyles();
        await this.loadCategories();
        await this.loadTags();
      }

      this.importing = false;
      this.importMessage = '';
      return { status: 'success' as const };
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
      this.sortOrder = sortOrder ?? (sortBy.includes('_at') ? 'desc' : 'asc');
      await this.loadStyles();
    },

    async toggleSortOrder() {
      this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
      await this.loadStyles();
    },

    // ── Builder ──

    async loadFilters() {
      if (this.builderFilters.length) return;
      const mod = await import('@/stores/filters.json');
      this.builderFilters = mod.default?.filters ?? mod.filters ?? [];
    },

    async loadFilterThumbnailList() {
      const api = await getApiAsync();
      const res = await api.list_filter_thumbnails();
      if (res.status === 'success') {
        this.availableThumbnails = res.thumbnails ?? [];
      }
    },

    async loadFilterThumbnail(imageName: string): Promise<string | null> {
      if (this.thumbnailCache[imageName]) return this.thumbnailCache[imageName];
      if (!this.availableThumbnails.includes(imageName)) return null;

      const api = await getApiAsync();
      const res = await api.get_filter_thumbnail({ image_name: imageName });
      if (res.status === 'success' && res.data_url) {
        this.thumbnailCache[imageName] = res.data_url;
        return res.data_url;
      }
      return null;
    },

    toggleFilter(filter: Filter) {
      const idx = this.builderSelected.findIndex((f) => f.prompt === filter.prompt);
      if (idx === -1) {
        this.builderSelected.push(filter);
      } else {
        this.builderSelected.splice(idx, 1);
      }
    },

    removeFilter(filter: Filter) {
      this.builderSelected = this.builderSelected.filter((f) => f.prompt !== filter.prompt);
    },

    clearGeneratedStyle() {
      this.generatedStyle = null;
    },

    clearBuilder() {
      this.builderSelected = [];
      this.generatedStyle = null;
      this.builderError = '';
    },

    async submitThumbnailBatch(selectedGroups: string[]) {
      if (!selectedGroups.length) return;
      const api = await getApiAsync();

      // Get styles dir from backend
      const dirRes = await api.get_styles_dir();
      const stylesPath = dirRes.status === 'success' && dirRes.path ? dirRes.path : '~/.rzem-ai/styles';
      const outputDir = `${stylesPath}/thumbnails`;

      // Load thumbnail prompts
      const promptsMod = await import('@/stores/thumbnail_prompts.json');
      const prompts: Record<string, string> = promptsMod.default ?? promptsMod;

      // Get current inference params for model config
      const { useInferenceStore } = await import('@/stores/inference');
      const inferenceStore = useInferenceStore();

      // Build all prompts from selected groups
      for (const groupKey of selectedGroups) {
        const template = prompts[groupKey];
        if (!template) continue;

        const filters = this.builderFilters.filter((f) => `${f.category} / ${f.subcategory}` === groupKey);

        for (const filter of filters) {
          const prompt = template.replace('{label}', filter.label);
          const jobParams: Record<string, any> = {
            ...inferenceStore.params,
            prompt,
            width: 1184,
            height: 896,
            seed: -1,
            output_dir: outputDir,
            output_filename: filter.image,
          };
          if (inferenceStore.selectedBundleId) {
            jobParams.bundle_id = inferenceStore.selectedBundleId;
          }
          // Clean undefined keys
          for (const key of Object.keys(jobParams)) {
            if (jobParams[key] === undefined || jobParams[key] === '') {
              delete jobParams[key];
            }
          }
          await api.submit_job(jobParams);
        }
      }
    },

    async generateStyleFromFilters() {
      if (!this.builderSelected.length) return;
      this.builderGenerating = true;
      this.builderError = '';
      this.generatedStyle = null;

      try {
        const api = await getApiAsync();
        const res = await api.generate_style_from_filters({
          filters: this.builderSelected.map((f) => f.prompt),
        });

        if (res.status === 'success' && res.style_data) {
          this.generatedStyle = res.style_data;
        } else {
          this.builderError = res.message ?? 'Generation failed';
        }
      } catch (e: any) {
        this.builderError = e.message ?? 'Generation failed';
      } finally {
        this.builderGenerating = false;
      }
    },
  },
});

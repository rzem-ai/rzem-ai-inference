import { defineStore } from 'pinia';
import { invoke } from '@/utils/backend-bridge';
import type { StyleInfo, StyleDetail, StyleRequest } from '@/types';

export const useStylesStore = defineStore('styles', {
  state: () => ({
    styles: [] as StyleInfo[],
    selectedStyle: null as StyleDetail | null,
    loading: false,
    error: null as string | null,
    isInitialized: false,
  }),

  getters: {
    stylesByCategory(state): Record<string, StyleInfo[]> {
      const grouped: Record<string, StyleInfo[]> = {};
      for (const style of state.styles) {
        const category = style.category || 'Uncategorized';
        if (!grouped[category]) grouped[category] = [];
        grouped[category].push(style);
      }
      // Sort each group by name
      for (const key of Object.keys(grouped)) {
        grouped[key].sort((a, b) => a.name.localeCompare(b.name));
      }
      return grouped;
    },

    categoriesWithCounts(state): Map<string, number> {
      const counts = new Map<string, number>();
      for (const style of state.styles) {
        const category = style.category || 'Uncategorized';
        counts.set(category, (counts.get(category) || 0) + 1);
      }
      return counts;
    },

    favoriteStyles(state): StyleInfo[] {
      return state.styles.filter((s) => s.isFavorite);
    },

    sortedByUsage(state): StyleInfo[] {
      return [...state.styles].sort((a, b) => b.usageCount - a.usageCount);
    },
  },

  actions: {
    // Standard initialization method
    async initialize(): Promise<void> {
      // Guard: only initialize once
      if (this.isInitialized) {
        return
      }

      try {
        await this.loadStyles()
        this.isInitialized = true
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('[StylesStore] Initialization failed:', err)
        throw err
      }
    },

    // Force reload (for manual refresh)
    async reload(): Promise<void> {
      this.isInitialized = false
      await this.initialize()
    },

    async loadStyles() {
      this.loading = true;
      this.error = null;
      try {
        this.styles = await invoke<StyleInfo[]>('get_all_styles');
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async loadStyleDetail(styleId: string) {
      this.loading = true;
      this.error = null;
      try {
        const detail = await invoke<StyleDetail | null>('get_style_detail', { styleId });
        this.selectedStyle = detail ?? null;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async createStyle(request: StyleRequest): Promise<StyleInfo> {
      this.loading = true;
      this.error = null;
      try {
        const newStyle = await invoke<StyleInfo>('create_style', request);
        this.styles.push(newStyle);
        return newStyle;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async updateStyle(styleId: string, request: StyleRequest): Promise<StyleInfo> {
      this.loading = true;
      this.error = null;
      try {
        const updatedStyle = await invoke<StyleInfo>('update_style', {
          styleId,
          ...request,
        });
        const index = this.styles.findIndex((s) => s.id === styleId);
        if (index !== -1) {
          this.styles[index] = updatedStyle;
        }
        return updatedStyle;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async deleteStyle(styleId: string) {
      this.loading = true;
      this.error = null;
      try {
        await invoke('delete_style', { styleId });
        this.styles = this.styles.filter((s) => s.id !== styleId);
        if (this.selectedStyle?.id === styleId) {
          this.selectedStyle = null;
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async addLoraToStyle(styleId: string, loraId: string, strength: number, priority: number) {
      try {
        await invoke('add_lora_to_style', { styleId, loraId, strength, priority });
        // Reload style detail to reflect changes
        if (this.selectedStyle?.id === styleId) {
          await this.loadStyleDetail(styleId);
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      }
    },

    async removeLoraFromStyle(styleId: string, loraId: string) {
      try {
        await invoke('remove_lora_from_style', { styleId, loraId });
        // Reload style detail to reflect changes
        if (this.selectedStyle?.id === styleId) {
          await this.loadStyleDetail(styleId);
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      }
    },

    async addExample(
      styleId: string,
      exampleType: 'prompt' | 'image',
      content: string,
      generationParams?: string
    ): Promise<string> {
      try {
        const exampleId = await invoke<string>('add_style_example', {
          styleId,
          exampleType,
          content,
          generationParams,
        });
        // Reload style detail to reflect changes
        if (this.selectedStyle?.id === styleId) {
          await this.loadStyleDetail(styleId);
        }
        return exampleId;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      }
    },

    async removeExample(exampleId: string) {
      try {
        await invoke('remove_style_example', { exampleId });
        // Reload current style detail if it has this example
        if (this.selectedStyle) {
          await this.loadStyleDetail(this.selectedStyle.id);
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      }
    },

    async previewTemplate(template: string, prompt: string): Promise<string> {
      try {
        return await invoke<string>('render_style_template', {
          template,
          userPrompt: prompt,
        });
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      }
    },

    async bulkDeleteStyles(styleIds: string[]) {
      this.loading = true;
      this.error = null;
      try {
        // Delete styles one by one (backend doesn't have bulk delete yet)
        await Promise.all(styleIds.map(id => invoke('delete_style', { styleId: id })));
        // Remove from local state
        this.styles = this.styles.filter(s => !styleIds.includes(s.id));
        // Clear selection if selected style was deleted
        if (this.selectedStyle && styleIds.includes(this.selectedStyle.id)) {
          this.selectedStyle = null;
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async bulkUpdateStyles(styleIds: string[], updates: Partial<StyleRequest>) {
      this.loading = true;
      this.error = null;
      try {
        // Update each style
        for (const styleId of styleIds) {
          const style = this.styles.find(s => s.id === styleId);
          if (style) {
            const request: StyleRequest = {
              name: updates.name ?? style.name,
              description: updates.description ?? style.description,
              promptTemplate: updates.promptTemplate ?? style.promptTemplate,
              defaultStrength: updates.defaultStrength ?? style.defaultStrength,
              strengthMin: updates.strengthMin ?? style.strengthMin,
              strengthMax: updates.strengthMax ?? style.strengthMax,
              category: updates.category !== undefined ? updates.category : style.category,
              isFavorite: updates.isFavorite !== undefined ? updates.isFavorite : style.isFavorite,
            };
            await this.updateStyle(styleId, request);
          }
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async uploadThumbnail(styleId: string, imageFile: File): Promise<string> {
      this.loading = true;
      this.error = null;
      try {
        // Convert file to base64
        const base64 = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => {
            const result = reader.result as string;
            // Remove data URL prefix (e.g., "data:image/png;base64,")
            const base64Data = result.split(',')[1];
            resolve(base64Data);
          };
          reader.onerror = reject;
          reader.readAsDataURL(imageFile);
        });

        const thumbnailPath = await invoke<string>('upload_style_thumbnail', {
          styleId,
          imageData: base64,
        });

        // Update local state
        const style = this.styles.find(s => s.id === styleId);
        if (style) {
          style.thumbnailPath = thumbnailPath;
        }
        if (this.selectedStyle?.id === styleId) {
          this.selectedStyle.thumbnailPath = thumbnailPath;
        }

        return thumbnailPath;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async deleteThumbnail(styleId: string) {
      this.loading = true;
      this.error = null;
      try {
        await invoke('delete_style_thumbnail', { styleId });

        // Update local state
        const style = this.styles.find(s => s.id === styleId);
        if (style) {
          style.thumbnailPath = undefined;
        }
        if (this.selectedStyle?.id === styleId) {
          this.selectedStyle.thumbnailPath = undefined;
        }
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      } finally {
        this.loading = false;
      }
    },

    clearSelection() {
      this.selectedStyle = null;
    },
  },
});

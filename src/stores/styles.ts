import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { StyleInfo, StyleDetail, StyleLoraWithInfo, StyleExample, StyleRequest } from '@/types';

// Backend uses snake_case, frontend uses camelCase
interface BackendStyleInfo {
  id: string;
  name: string;
  description?: string;
  prompt_template: string;
  default_strength: number;
  strength_min: number;
  strength_max: number;
  category?: string;
  thumbnail_path?: string;
  is_favorite: boolean;
  usage_count: number;
  created_at: number;
  updated_at: number;
}

interface BackendStyleLoraWithInfo {
  lora_id: string;
  lora_name: string;
  lora_trigger_words?: string;
  strength: number;
  priority: number;
}

interface BackendStyleExample {
  id: string;
  style_id: string;
  example_type: string;
  content: string;
  generation_params?: string;
  created_at: number;
}

interface BackendStyleDetail extends BackendStyleInfo {
  loras: BackendStyleLoraWithInfo[];
  examples: BackendStyleExample[];
}

function mapStyleInfo(backend: BackendStyleInfo): StyleInfo {
  return {
    id: backend.id,
    name: backend.name,
    description: backend.description,
    promptTemplate: backend.prompt_template,
    defaultStrength: backend.default_strength,
    strengthMin: backend.strength_min,
    strengthMax: backend.strength_max,
    category: backend.category,
    thumbnailPath: backend.thumbnail_path,
    isFavorite: backend.is_favorite,
    usageCount: backend.usage_count,
    createdAt: backend.created_at,
    updatedAt: backend.updated_at,
  };
}

function mapStyleLoraWithInfo(backend: BackendStyleLoraWithInfo): StyleLoraWithInfo {
  return {
    loraId: backend.lora_id,
    loraName: backend.lora_name,
    loraTriggerWords: backend.lora_trigger_words,
    strength: backend.strength,
    priority: backend.priority,
  };
}

function mapStyleExample(backend: BackendStyleExample): StyleExample {
  return {
    id: backend.id,
    styleId: backend.style_id,
    exampleType: backend.example_type as 'prompt' | 'image',
    content: backend.content,
    generationParams: backend.generation_params,
    createdAt: backend.created_at,
  };
}

function mapStyleDetail(backend: BackendStyleDetail): StyleDetail {
  return {
    ...mapStyleInfo(backend),
    loras: backend.loras.map(mapStyleLoraWithInfo),
    examples: backend.examples.map(mapStyleExample),
  };
}

function mapStyleRequest(request: StyleRequest): Record<string, unknown> {
  return {
    name: request.name,
    description: request.description,
    prompt_template: request.promptTemplate,
    default_strength: request.defaultStrength,
    strength_min: request.strengthMin,
    strength_max: request.strengthMax,
    category: request.category,
    is_favorite: request.isFavorite,
  };
}

export const useStylesStore = defineStore('styles', {
  state: () => ({
    styles: [] as StyleInfo[],
    selectedStyle: null as StyleDetail | null,
    loading: false,
    error: null as string | null,
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
    async loadStyles() {
      this.loading = true;
      this.error = null;
      try {
        const backendStyles = await invoke<BackendStyleInfo[]>('get_all_styles');
        this.styles = backendStyles.map(mapStyleInfo);
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
        const backendDetail = await invoke<BackendStyleDetail | null>('get_style_detail', { styleId });
        if (backendDetail) {
          this.selectedStyle = mapStyleDetail(backendDetail);
        } else {
          this.selectedStyle = null;
        }
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
        const backendStyle = await invoke<BackendStyleInfo>('create_style', {
          request: mapStyleRequest(request),
        });
        const newStyle = mapStyleInfo(backendStyle);
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
        const backendStyle = await invoke<BackendStyleInfo>('update_style', {
          styleId,
          request: mapStyleRequest(request),
        });
        const updatedStyle = mapStyleInfo(backendStyle);
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

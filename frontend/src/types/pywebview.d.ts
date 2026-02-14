import type { ApiResponse, InferenceEvent, ModelBundle, GalleryImage, Folder, Tag, Style, StyleLoRA, LoRA, VramUsage, EngineStatus, CachedModel, DataPaths, DiskUsage, Conversation, ConversationMessage, ChatEvent } from "./inference";

export interface PywebviewAPI {
  // ── System ──
  get_system_info(): Promise<{
    platform: string;
    platform_version: string;
    python_version: string;
    machine: string;
    processor: string;
  }>;
  greet(name: string): Promise<string>;
  increment_counter(): Promise<number>;
  get_counter(): Promise<number>;
  health_check(): Promise<{ status: string }>;

  // ── Inference engine ──
  get_gpu_info(): Promise<ApiResponse<{
    device_type?: string;
    device_name?: string | null;
    total_vram_gb?: number;
  }>>;
  start_engine(args?: {
    device?: string;
    vram_limit_gb?: number;
  }): Promise<ApiResponse>;
  stop_engine(): Promise<ApiResponse>;
  engine_ready(): Promise<ApiResponse<{ ready: boolean }>>;

  // ── Jobs ──
  submit_job(args: Record<string, any>): Promise<ApiResponse<{ job_id?: string }>>;
  cancel_job(args: { job_id: string }): Promise<ApiResponse>;
  poll_events(): Promise<ApiResponse<{ events?: InferenceEvent[] }>>;

  // ── Images ──
  get_image_base64(args: {
    image_path: string;
  }): Promise<ApiResponse<{ data_url?: string }>>;
  get_debug_images(): Promise<ApiResponse<{
    output?: string | null;
    previews?: Record<string, string>;
  }>>;

  // ── Bundles ──
  get_bundles(): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  get_bundle(args: { bundle_id: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  get_bundles_for_type(args: { transformer_type: string }): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  create_bundle(args: Record<string, any>): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  update_bundle(args: { bundle_id: string } & Partial<ModelBundle>): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  delete_bundle(args: { bundle_id: string }): Promise<ApiResponse>;
  reset_default_bundles(): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;

  // ── Gallery images ──
  get_gallery_images(args?: {
    limit?: number;
    offset?: number;
    folder_id?: string;
    tag_id?: number;
    search?: string;
    favorites_only?: boolean;
  }): Promise<ApiResponse<{ images?: GalleryImage[]; total?: number }>>;
  get_image(args: { image_id: string }): Promise<ApiResponse<{ image?: GalleryImage }>>;
  toggle_favorite(args: { image_id: string }): Promise<ApiResponse<{ image?: GalleryImage }>>;
  delete_image(args: { image_id: string }): Promise<ApiResponse>;

  // ── Folders ──
  get_folders(): Promise<ApiResponse<{ folders?: Folder[] }>>;
  create_folder(args: {
    id: string;
    name: string;
    parent_id?: string;
    color?: string;
    icon?: string;
    sort_order?: number;
  }): Promise<ApiResponse<{ folder?: Folder }>>;
  update_folder(args: { folder_id: string; name?: string; parent_id?: string; color?: string; icon?: string; sort_order?: number }): Promise<ApiResponse<{ folder?: Folder }>>;
  delete_folder(args: { folder_id: string }): Promise<ApiResponse>;

  // ── Folder ↔ Image ──
  add_image_to_folder(args: { image_id: string; folder_id: string }): Promise<ApiResponse>;
  remove_image_from_folder(args: { image_id: string; folder_id: string }): Promise<ApiResponse>;

  // ── Tags ──
  get_tags(): Promise<ApiResponse<{ tags?: Tag[] }>>;
  create_tag(args: { name: string; color?: string; category?: string }): Promise<ApiResponse<{ tag?: Tag }>>;
  update_tag(args: { tag_id: number; name?: string; color?: string; category?: string }): Promise<ApiResponse<{ tag?: Tag }>>;
  delete_tag(args: { tag_id: number }): Promise<ApiResponse>;

  // ── Tag ↔ Image ──
  add_tag_to_image(args: { image_id: string; tag_id: number }): Promise<ApiResponse>;
  remove_tag_from_image(args: { image_id: string; tag_id: number }): Promise<ApiResponse>;
  get_image_tags(args: { image_id: string }): Promise<ApiResponse<{ tags?: Tag[] }>>;

  // ── Settings (key-value) ──
  get_setting(args: { key: string }): Promise<ApiResponse<{ value?: string | null }>>;
  set_setting(args: { key: string; value: string }): Promise<ApiResponse>;

  // ── Settings pages ──
  get_vram_usage(): Promise<ApiResponse<Partial<VramUsage>>>;
  clear_vram_cache(): Promise<ApiResponse>;
  get_engine_status(): Promise<ApiResponse<Partial<EngineStatus>>>;
  get_cuda_version(): Promise<ApiResponse<{ cuda_version?: string | null }>>;
  reset_engine(): Promise<ApiResponse>;
  get_cache_info(): Promise<ApiResponse<{ models?: CachedModel[]; total_size?: number }>>;
  delete_cached_model(args: { repo_id: string }): Promise<ApiResponse>;
  get_data_paths(): Promise<ApiResponse<Partial<DataPaths>>>;
  get_disk_usage(): Promise<ApiResponse<Partial<DiskUsage>>>;

  // ── Styles ──
  get_styles(args?: {
    category?: string;
    tag_id?: number;
    search?: string;
    favorites_only?: boolean;
  }): Promise<ApiResponse<{ styles?: Style[] }>>;
  get_style(args: { style_id: string }): Promise<ApiResponse<{ style?: Style; loras?: StyleLoRA[]; tags?: Tag[] }>>;
  create_style(args: {
    id: string;
    name: string;
    prompt_template: string;
    description?: string;
    negative_prompt?: string;
    category?: string;
    thumbnail_path?: string;
  }): Promise<ApiResponse<{ style?: Style }>>;
  update_style(args: { style_id: string; name?: string; description?: string; prompt_template?: string; negative_prompt?: string; category?: string; thumbnail_path?: string }): Promise<ApiResponse<{ style?: Style }>>;
  delete_style(args: { style_id: string }): Promise<ApiResponse>;
  toggle_style_favorite(args: { style_id: string }): Promise<ApiResponse<{ style?: Style }>>;
  get_style_categories(): Promise<ApiResponse<{ categories?: string[] }>>;

  // ── Style ↔ LoRA ──
  get_style_loras(args: { style_id: string }): Promise<ApiResponse<{ loras?: StyleLoRA[] }>>;
  set_style_loras(args: { style_id: string; loras: Array<{ lora_id: string; strength: number; priority?: number }> }): Promise<ApiResponse<{ loras?: StyleLoRA[] }>>;

  // ── Style ↔ Tag ──
  get_style_tags(args: { style_id: string }): Promise<ApiResponse<{ tags?: Tag[] }>>;
  set_style_tags(args: { style_id: string; tag_ids: number[] }): Promise<ApiResponse<{ tags?: Tag[] }>>;

  // ── LoRAs ──
  get_loras(): Promise<ApiResponse<{ loras?: LoRA[] }>>;
  create_lora(args: {
    id: string;
    name: string;
    path: string;
    trigger_words?: string;
    base_model?: string;
    size_bytes?: number;
    strength?: number;
  }): Promise<ApiResponse<{ lora?: LoRA }>>;
  browse_lora_files(): Promise<ApiResponse<{ loras?: LoRA[] }>>;
  browse_image_file(): Promise<ApiResponse<{ path?: string | null }>>;

  // ── Batch ──
  batch_parse_data(args: { content: string }): Promise<ApiResponse<{
    columns?: string[];
    rows?: Record<string, string>[];
  }>>;
  batch_render_template(args: {
    template: string;
    rows: Record<string, string>[];
  }): Promise<ApiResponse<{
    rendered?: string[];
    errors?: Array<{ row: number; error: string }>;
  }>>;

  // ── Chat ──
  chat_is_configured(): Promise<ApiResponse<{ configured?: boolean }>>;
  chat_set_api_key(args: { api_key: string }): Promise<ApiResponse>;
  chat_create_conversation(args?: { title?: string }): Promise<ApiResponse<{ conversation?: Conversation }>>;
  chat_get_conversations(): Promise<ApiResponse<{ conversations?: Conversation[] }>>;
  chat_get_messages(args: { conversation_id: string }): Promise<ApiResponse<{ messages?: ConversationMessage[] }>>;
  chat_delete_conversation(args: { conversation_id: string }): Promise<ApiResponse>;
  chat_send_message(args: {
    conversation_id: string;
    content: string;
    image_paths?: string[];
    generation_context?: Record<string, any>;
  }): Promise<ApiResponse>;
  poll_chat_events(): Promise<ApiResponse<{ events?: ChatEvent[] }>>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}

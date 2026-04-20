import type { ApiResponse, InferenceEvent, BundleType, ModelBundle, SubmitJobParams, GalleryImage, Folder, Tag, Style, StyleLoRA, StyleExample, StyleReviewSuggestion, LoRA, VramUsage, EngineStatus, CachedModel, DataPaths, DiskUsage, Conversation, ConversationMessage, ChatEvent, DiscoveredServer } from "./inference";
import type { WorkflowListItem, WorkflowEvent } from "./workflow";

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
  start_engine(): Promise<ApiResponse>;
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

  // ── Bundle Types ──
  get_bundle_types(): Promise<ApiResponse<{ bundle_types?: BundleType[] }>>;

  // ── Bundles ──
  get_bundles(args?: { include_hidden?: boolean }): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  get_bundle(args: { bundle_id: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  get_bundles_for_type(args: { transformer_type: string }): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  create_bundle(args: Partial<ModelBundle> & { id: string; label: string; transformer_type: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  update_bundle(args: { bundle_id: string } & Partial<ModelBundle>): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  delete_bundle(args: { bundle_id: string }): Promise<ApiResponse>;
  toggle_bundle_favorite(args: { bundle_id: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  toggle_bundle_hidden(args: { bundle_id: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  reset_default_bundles(): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;

  // ── Gallery images ──
  get_gallery_images(args?: {
    limit?: number;
    offset?: number;
    folder_id?: string;
    tag_id?: number;
    search?: string;
    favorites_only?: boolean;
    sort_by?: string;
    sort_order?: string;
  }): Promise<ApiResponse<{ images?: GalleryImage[]; total?: number }>>;
  get_image(args: { image_id: string }): Promise<ApiResponse<{ image?: GalleryImage }>>;
  toggle_favorite(args: { image_id: string }): Promise<ApiResponse<{ image?: GalleryImage }>>;
  save_image_as(args: { file_path: string }): Promise<ApiResponse<{ saved?: boolean; path?: string }>>;
  batch_save_images(args: { image_ids: string[] }): Promise<ApiResponse<{ saved_count?: number; folder?: string }>>;
  delete_image(args: { image_id: string }): Promise<ApiResponse>;
  upscale_image(args: { image_id: string; scale_factor: number }): Promise<ApiResponse<{ image?: GalleryImage }>>;

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

  // ── Setup ──
  complete_setup(args: { generation_mode: string; api_keys?: Record<string, string> }): Promise<ApiResponse>;

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
  browse_output_directory(): Promise<ApiResponse<{ changed?: boolean; output_dir?: string }>>;

  // ── SSL Verification ──
  get_ssl_verification_disabled(): Promise<ApiResponse<{ disabled?: boolean }>>;
  set_ssl_verification_disabled(args: { disabled: boolean }): Promise<ApiResponse<{ disabled?: boolean }>>;

  // ── Styles ──
  get_styles(args?: {
    category?: string;
    tag_id?: number;
    search?: string;
    favorites_only?: boolean;
    bundle_type?: string;
    sort_by?: string;
    sort_order?: string;
  }): Promise<ApiResponse<{ styles?: Style[] }>>;
  get_style(args: { style_id: string }): Promise<ApiResponse<{ style?: Style; loras?: StyleLoRA[]; tags?: Tag[]; examples?: StyleExample[] }>>;
  create_style(args: {
    id: string;
    name: string;
    prompt_template: string;
    description?: string;
    negative_prompt?: string;
    category?: string;
    thumbnail_path?: string;
    bundle_id?: string;
  }): Promise<ApiResponse<{ style?: Style }>>;
  update_style(args: { style_id: string; name?: string; description?: string; prompt_template?: string; negative_prompt?: string; category?: string; thumbnail_path?: string; bundle_id?: string | null }): Promise<ApiResponse<{ style?: Style }>>;
  delete_style(args: { style_id: string }): Promise<ApiResponse>;
  toggle_style_favorite(args: { style_id: string }): Promise<ApiResponse<{ style?: Style }>>;
  get_style_categories(): Promise<ApiResponse<{ categories?: string[] }>>;
  get_style_bundle_type_counts(): Promise<ApiResponse<{ counts?: Array<{ type_id: string; count: number }> }>>;
  get_styles_dir(): Promise<ApiResponse<{ path?: string }>>;
  list_filter_thumbnails(): Promise<ApiResponse<{ thumbnails?: string[] }>>;
  get_filter_thumbnail(args: { image_name: string }): Promise<ApiResponse<{ data_url?: string }>>;

  // ── Style examples ──
  get_style_examples(args: { style_id: string }): Promise<ApiResponse<{ examples?: StyleExample[] }>>;
  create_style_example(args: {
    style_id: string;
    prompt: string;
    image_path?: string;
    seed?: number;
    width?: number;
    height?: number;
    steps?: number;
    cfg_scale?: number;
  }): Promise<ApiResponse<{ example?: StyleExample }>>;
  delete_style_example(args: { example_id: string }): Promise<ApiResponse>;

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
  browse_image_file(args?: { style_id?: string }): Promise<ApiResponse<{ path?: string | null; thumbnail_path?: string | null }>>;
  browse_input_image(): Promise<ApiResponse<{ path?: string | null }>>;
  save_clipboard_image(args: { data_url: string }): Promise<ApiResponse<{ path?: string }>>;
  import_style_image(args: { source_path: string; style_id?: string }): Promise<ApiResponse<{ path?: string; thumbnail_path?: string }>>;
  browse_metadata_files(): Promise<ApiResponse<{ paths?: string[] }>>;
  browse_and_import_metadata(): Promise<ApiResponse<{ styles?: Style[]; errors?: string[] }>>;
  import_civitai_metadata(args: { file_path?: string; json_content?: string }): Promise<ApiResponse<{ style?: Style }>>;
  browse_invokeai_db(): Promise<ApiResponse<{ path?: string | null }>>;
  import_invokeai_loras(args: { db_path: string }): Promise<ApiResponse<{ imported?: number; total?: number; errors?: string[] }>>;

  // ── Style from Image ──
  create_style_from_image(args: { image_path: string }): Promise<ApiResponse<{ style?: Style }>>;

  // ── Style AI Review ──
  review_styles(args: { style_ids: string[] }): Promise<ApiResponse<{ suggestions?: StyleReviewSuggestion[] }>>;
  apply_style_review(args: { changes: Array<{ style_id: string; name?: string; tags?: string[] }> }): Promise<ApiResponse<{ applied?: number }>>;

  // ── Style Builder ──
  generate_style_from_filters(args: { filters: string[] }): Promise<ApiResponse<{
    style_data?: {
      name: string;
      description: string;
      prompt_template: string;
      negative_prompt: string;
      category: string;
    };
  }>>;

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

  // ── Discovery ──
  get_discovered_servers(): Promise<ApiResponse<{ servers?: DiscoveredServer[] }>>;
  connect_to_server(args: { host: string; port: number }): Promise<ApiResponse<{ mode?: string }>>;
  disconnect_from_server(): Promise<ApiResponse<{ mode?: string }>>;
  get_connection_mode(): Promise<ApiResponse<{
    mode?: string;
    connected_server?: { host: string; port: number } | null;
  }>>;

  // ── Chat ──
  chat_is_configured(): Promise<ApiResponse<{ configured?: boolean }>>;
  chat_set_api_key(args: { api_key: string; provider?: string }): Promise<ApiResponse>;
  chat_create_conversation(args?: { title?: string }): Promise<ApiResponse<{ conversation?: Conversation }>>;
  chat_get_conversations(): Promise<ApiResponse<{ conversations?: Conversation[] }>>;
  chat_get_messages(args: { conversation_id: string }): Promise<ApiResponse<{ messages?: ConversationMessage[] }>>;
  chat_delete_conversation(args: { conversation_id: string }): Promise<ApiResponse>;
  chat_get_provider_info(): Promise<ApiResponse<{
    provider?: string;
    models?: Array<{ id: string; label: string }>;
  }>>;
  chat_send_message(args: {
    conversation_id: string;
    content: string;
    image_paths?: string[];
    generation_context?: Record<string, any>;
    display_text?: string;
  }): Promise<ApiResponse>;
  poll_chat_events(): Promise<ApiResponse<{ events?: ChatEvent[] }>>;

  // ── Workflows ──
  get_workflows(args?: Record<string, unknown>): Promise<ApiResponse<{ workflows?: WorkflowListItem[] }>>;
  get_workflow(args: { workflow_id: string }): Promise<ApiResponse<{ workflow?: { id: string; name: string; description: string; graph_json: string; created_at: number; updated_at: number } }>>;
  save_workflow(args: { workflow_id: string; name: string; description?: string; graph_json?: string }): Promise<ApiResponse>;
  delete_workflow(args: { workflow_id: string }): Promise<ApiResponse>;
  run_workflow(args: { graph_json: string }): Promise<ApiResponse<{ run_id?: string }>>;
  cancel_workflow(args: { run_id: string }): Promise<ApiResponse>;
  poll_workflow_events(args?: Record<string, unknown>): Promise<ApiResponse<{ events?: WorkflowEvent[] }>>;
  browse_workflow_image(args?: Record<string, unknown>): Promise<ApiResponse<{ path?: string | null }>>;

  // ── Auto-update ──
  install_update(): Promise<ApiResponse>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}

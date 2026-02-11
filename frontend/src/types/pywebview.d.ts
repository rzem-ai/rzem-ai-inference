import type { ApiResponse, InferenceEvent, ModelBundle, GalleryImage, Folder, Tag } from "./inference";

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

  // ── Settings ──
  get_setting(args: { key: string }): Promise<ApiResponse<{ value?: string | null }>>;
  set_setting(args: { key: string; value: string }): Promise<ApiResponse>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}

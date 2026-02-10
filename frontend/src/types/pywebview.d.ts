import type { ApiResponse, InferenceEvent, ModelBundle } from "./inference";

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

  // ── Bundles ──
  get_bundles(): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  get_bundle(args: { bundle_id: string }): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  get_bundles_for_type(args: { transformer_type: string }): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
  create_bundle(args: Record<string, any>): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  update_bundle(args: { bundle_id: string } & Partial<ModelBundle>): Promise<ApiResponse<{ bundle?: ModelBundle }>>;
  delete_bundle(args: { bundle_id: string }): Promise<ApiResponse>;
  reset_default_bundles(): Promise<ApiResponse<{ bundles?: ModelBundle[] }>>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}

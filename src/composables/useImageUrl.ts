/**
 * Image URL helper for handling paths in both local and client modes
 *
 * In local/server mode: Uses Tauri's convertFileSrc to access local files
 * In client mode: Converts file paths to server HTTP URLs
 */

import { ref } from 'vue'
import { convertFileSrc } from '@/utils/backend-bridge'
import { isClientMode, useRuntimeConfig } from './useRuntimeConfig'

/**
 * Convert a file path to a usable URL based on the current runtime mode
 *
 * @param filePath - The file path from the backend (can be absolute or just filename)
 * @param isImage - Whether this is an image file (for client mode URL construction)
 * @returns URL that can be used in <img> src or background-image
 */
export async function getImageUrl(filePath: string | undefined): Promise<string | undefined> {
  if (!filePath) {
    return undefined
  }

  if (isClientMode()) {
    // Client mode: Convert to server URL
    const config = await useRuntimeConfig()
    const serverUrl = config.config.value?.server_url

    if (!serverUrl) {
      console.error('Client mode but no server URL configured')
      return undefined
    }

    // Extract filename from path (handle both Unix and Windows paths)
    const filename = filePath.split(/[/\\]/).pop()
    if (!filename) {
      console.error('Invalid file path:', filePath)
      return undefined
    }

    // Construct server URL
    return `${serverUrl}/api/v1/files/${encodeURIComponent(filename)}`
  } else {
    // Local/Server mode: Use Tauri's file serving
    return convertFileSrc(filePath)
  }
}

/**
 * Convert multiple file paths to URLs
 */
export async function getImageUrls(
  filePaths: (string | undefined)[]
): Promise<(string | undefined)[]> {
  return Promise.all(filePaths.map(getImageUrl))
}

/**
 * Get URL for an image synchronously (for use in templates)
 *
 * Note: This assumes runtime config is already loaded. Use after app initialization.
 */
export function getImageUrlSync(filePath: string | undefined): string | undefined {
  if (!filePath) {
    return undefined
  }

  // Check mode synchronously (assumes config is loaded)
  if (isClientMode()) {
    // We need to get the server URL from the cached config
    // This is a bit hacky but works for templates
    const mode = isClientMode()
    if (mode) {
      // For now, construct URL from window.location if available
      // In production, you'd want to pass server_url more cleanly
      // This is a limitation of synchronous access
      console.warn('getImageUrlSync called in client mode - use getImageUrl instead')
      return undefined
    }
  }

  // Local/Server mode: Use Tauri's file serving
  return convertFileSrc(filePath)
}

/**
 * Composable for reactive image URL handling
 */
export function useImageUrl(filePath: string | undefined) {
  const url = ref<string | undefined>(undefined)
  const loading = ref(true)
  const error = ref<Error | null>(null)

  const load = async () => {
    loading.value = true
    error.value = null

    try {
      url.value = await getImageUrl(filePath)
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      console.error('Failed to get image URL:', e)
    } finally {
      loading.value = false
    }
  }

  // Load immediately
  load()

  return {
    url,
    loading,
    error,
    reload: load,
  }
}

/**
 * Batch composable for loading multiple image URLs
 */
export function useImageUrls(filePaths: (string | undefined)[]) {
  const urls = ref<(string | undefined)[]>([])
  const loading = ref(true)
  const error = ref<Error | null>(null)

  const load = async () => {
    loading.value = true
    error.value = null

    try {
      urls.value = await getImageUrls(filePaths)
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      console.error('Failed to get image URLs:', e)
    } finally {
      loading.value = false
    }
  }

  // Load immediately
  load()

  return {
    urls,
    loading,
    error,
    reload: load,
  }
}

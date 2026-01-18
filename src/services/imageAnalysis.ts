/**
 * Image Analysis Service
 *
 * Uses Claude API (via Tauri backend) to analyze images and generate prompts.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Analyzes an image and generates a prompt to recreate it.
 *
 * @param imageData - Base64 image data (can be data URL or raw base64)
 * @param mediaType - Optional MIME type (inferred from data URL if not provided)
 * @returns A detailed prompt string
 */
export async function analyzeImageForPrompt(
  imageData: string,
  mediaType?: string
): Promise<string> {
  return invoke<string>('analyze_image_for_prompt', {
    imageData,
    mediaType,
  });
}

/**
 * Reads a File object and returns its base64 data URL.
 */
export function fileToDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const result = e.target?.result;
      if (typeof result === 'string') {
        resolve(result);
      } else {
        reject(new Error('Failed to read file as data URL'));
      }
    };
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsDataURL(file);
  });
}

/**
 * Validates that a file is an acceptable image type.
 */
export function isValidImageFile(file: File): boolean {
  const validTypes = ['image/png', 'image/jpeg', 'image/webp', 'image/gif'];
  return validTypes.includes(file.type);
}

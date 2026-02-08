/**
 * File dialog stubs for pywebview
 *
 * These are placeholder implementations until proper file dialogs are implemented
 * via HTML5 file inputs or native dialogs through pywebview.
 */

export async function open(_options?: any): Promise<string | string[] | null> {
  console.warn('File dialog not yet implemented in pywebview mode');
  alert('File selection is not yet available in pywebview mode. This feature is coming soon!');
  return null;
}

export async function save(_options?: any): Promise<string | null> {
  console.warn('Save dialog not yet implemented in pywebview mode');
  alert('File save dialog is not yet available in pywebview mode. This feature is coming soon!');
  return null;
}

export async function readFile(_path: string): Promise<Uint8Array> {
  console.warn('readFile not yet implemented in pywebview mode');
  throw new Error('File reading not yet implemented in pywebview mode');
}

export async function readTextFile(_path: string): Promise<string> {
  console.warn('readTextFile not yet implemented in pywebview mode');
  throw new Error('Text file reading not yet implemented in pywebview mode');
}

export async function writeFile(_path: string, _contents: Uint8Array | string): Promise<void> {
  console.warn('writeFile not yet implemented in pywebview mode');
  throw new Error('File writing not yet implemented in pywebview mode');
}

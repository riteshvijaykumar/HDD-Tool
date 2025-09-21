import { invoke } from '@tauri-apps/api/core';

export interface WipeMethodData {
  value: string;
  label: string;
  tooltip: string;
  default_pattern?: string;
  patterns?: { value: string; label: string }[];
}

export async function fetchWipeMethods(): Promise<WipeMethodData[] | null> {
  try {
    const result = await invoke<any>('get_wipe_methods');
    if (result && result.data) {
      return result.data as WipeMethodData[];
    }
    return result as WipeMethodData[];
  } catch (e) {
    console.error('[fetchWipeMethods] Failed to fetch wipe methods:', e);
    return null;
  }
}


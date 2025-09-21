import { invoke } from '@tauri-apps/api/core';

export interface SettingsData {
  darkMode: boolean;
  defaultMethod: string;
  defaultPasses: number;
  language: string;
  logLevel: string;
  reportLocation: string;
}

export async function fetchSettings(): Promise<SettingsData | null> {
  try {
    const result = await invoke<any>('get_settings');
    if (result && result.data) {
      return result.data as SettingsData;
    }
    return result as SettingsData;
  } catch (e) {
    console.error('[fetchSettings] Failed to fetch settings:', e);
    return null;
  }
}

export async function updateSettings(settings: SettingsData): Promise<boolean> {
  try {
    await invoke('set_settings', { settings });
    return true;
  } catch (e) {
    console.error('[updateSettings] Failed to update settings:', e);
    return false;
  }
}


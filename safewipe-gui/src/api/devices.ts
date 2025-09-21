import {invoke} from "@tauri-apps/api/core";

export interface DeviceInfo {
  name: string;
  path: string;
  size: string;
  type: string;
  removable: boolean;
}

export async function fetchDevices(): Promise<DeviceInfo[]> {
  console.log('[fetchDevices] Invoking list_devices...');
  try {
    const result = await invoke<any>('list_devices');
    console.log('[fetchDevices] Raw result from invoke:', result);
    if (result && Array.isArray(result.data)) {
      console.log('[fetchDevices] result.data is an array:', result.data);
      return result.data as DeviceInfo[];
    } else if (Array.isArray(result)) {
      console.log('[fetchDevices] Result is an array:', result);
      return result as DeviceInfo[];
    } else if (result && Array.isArray(result.devices)) {
      console.log('[fetchDevices] Result.devices is an array:', result.devices);
      return result.devices as DeviceInfo[];
    }
    console.warn('[fetchDevices] Result is not in expected format:', result);
    return [];
  } catch (e) {
    console.error('[fetchDevices] Failed to fetch devices:', e);
    throw e;
  }
}

import {invoke} from "@tauri-apps/api/core";

export interface DeviceInfo {
  name: string;
  path: string;
  size: string;
  type: string;
  removable: boolean;
}

export async function fetchDevices(): Promise<DeviceInfo[]> {
  try {
    const result = await invoke<any>('list_devices');
    // Adjust parsing as needed based on backend response structure
    if (Array.isArray(result)) {
      return result as DeviceInfo[];
    } else if (result && Array.isArray(result.devices)) {
      return result.devices as DeviceInfo[];
    }
    return [];
  } catch (e) {
    console.error('Failed to fetch devices:', e);
    throw e;
  }
}

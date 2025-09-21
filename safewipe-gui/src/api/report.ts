import { invoke } from '@tauri-apps/api/core';

export interface ReportData {
  device_name: string;
  device_path: string;
  device_size: string;
  method: string;
  status: string;
  date: string;
  certificate_id: string;
}

export async function fetchReport(operationId?: string): Promise<ReportData | null> {
  try {
    // You may need to adjust the command name and parameters to match your backend
    const result = await invoke<any>('generate_operation_report', operationId ? { operationId } : {});
    if (result && result.data) {
      return result.data as ReportData;
    }
    return result as ReportData;
  } catch (e) {
    console.error('[fetchReport] Failed to fetch report:', e);
    return null;
  }
}


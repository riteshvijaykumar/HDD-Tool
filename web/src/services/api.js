import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3001/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// API service for SafeWipe backend communication
export const safewipeAPI = {
  // Device management
  async getDevices() {
    const response = await api.get('/devices');
    return response.data;
  },

  async getDeviceRecommendation(deviceName) {
    const response = await api.get(`/devices/${deviceName}/recommendations`);
    return response.data;
  },

  async getAllRecommendations() {
    const response = await api.get('/recommendations');
    return response.data;
  },

  // Sanitization operations
  async createSanitizationPlan(method, deviceNames) {
    const response = await api.post('/sanitize/plan', {
      method,
      device_names: deviceNames,
    });
    return response.data;
  },

  async executeSanitization(method, deviceNames) {
    const response = await api.post('/sanitize/execute', {
      method,
      device_names: deviceNames,
    });
    return response.data;
  },

  // Operation monitoring
  async getOperationStatus(operationId) {
    const response = await api.get(`/operations/${operationId}`);
    return response.data;
  },

  async getAllOperations() {
    const response = await api.get('/operations');
    return response.data;
  },

  // Progress monitoring via EventSource (Server-Sent Events)
  createProgressStream() {
    return new EventSource(`${API_BASE_URL}/progress`);
  },
};

export default safewipeAPI;

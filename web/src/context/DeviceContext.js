import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { safewipeAPI } from '../services/api';
import toast from 'react-hot-toast';

const DeviceContext = createContext();

const initialState = {
  devices: [],
  recommendations: {},
  selectedDevices: [],
  operations: {},
  activeOperation: null,
  loading: false,
  error: null,
  progressStream: null,
};

function deviceReducer(state, action) {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };

    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };

    case 'SET_DEVICES':
      return { ...state, devices: action.payload, loading: false };

    case 'SET_RECOMMENDATIONS':
      return { ...state, recommendations: action.payload };

    case 'SET_SELECTED_DEVICES':
      return { ...state, selectedDevices: action.payload };

    case 'ADD_SELECTED_DEVICE':
      return {
        ...state,
        selectedDevices: [...state.selectedDevices, action.payload],
      };

    case 'REMOVE_SELECTED_DEVICE':
      return {
        ...state,
        selectedDevices: state.selectedDevices.filter(
          device => device.name !== action.payload
        ),
      };

    case 'SET_OPERATIONS':
      return { ...state, operations: action.payload };

    case 'SET_ACTIVE_OPERATION':
      return { ...state, activeOperation: action.payload };

    case 'UPDATE_PROGRESS':
      return {
        ...state,
        operations: {
          ...state.operations,
          [action.payload.operation_id]: {
            ...state.operations[action.payload.operation_id],
            progress: action.payload.progress,
          },
        },
      };

    default:
      return state;
  }
}

export function DeviceProvider({ children }) {
  const [state, dispatch] = useReducer(deviceReducer, initialState);

  // Load devices on mount
  useEffect(() => {
    loadDevices();
    loadRecommendations();
    setupProgressStream();

    return () => {
      if (state.progressStream) {
        state.progressStream.close();
      }
    };
  }, []);

  const loadDevices = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      const response = await safewipeAPI.getDevices();

      if (response.success) {
        dispatch({ type: 'SET_DEVICES', payload: response.data });
        toast.success(`Found ${response.data.length} storage devices`);
      } else {
        throw new Error(response.error || 'Failed to load devices');
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error.message });
      toast.error(`Failed to load devices: ${error.message}`);
    }
  };

  const loadRecommendations = async () => {
    try {
      const response = await safewipeAPI.getAllRecommendations();
      if (response.success) {
        dispatch({ type: 'SET_RECOMMENDATIONS', payload: response.data });
      }
    } catch (error) {
      console.error('Failed to load recommendations:', error);
    }
  };

  const setupProgressStream = () => {
    try {
      const stream = safewipeAPI.createProgressStream();

      stream.onmessage = (event) => {
        try {
          const update = JSON.parse(event.data);
          dispatch({ type: 'UPDATE_PROGRESS', payload: update });
        } catch (error) {
          console.error('Failed to parse progress update:', error);
        }
      };

      stream.onerror = (error) => {
        console.error('Progress stream error:', error);
      };

      return stream;
    } catch (error) {
      console.error('Failed to setup progress stream:', error);
    }
  };

  const selectDevice = (device) => {
    if (device.is_system_drive) {
      toast.error('Cannot select system drive for sanitization');
      return;
    }
    dispatch({ type: 'ADD_SELECTED_DEVICE', payload: device });
  };

  const deselectDevice = (deviceName) => {
    dispatch({ type: 'REMOVE_SELECTED_DEVICE', payload: deviceName });
  };

  const clearSelection = () => {
    dispatch({ type: 'SET_SELECTED_DEVICES', payload: [] });
  };

  const createSanitizationPlan = async (method) => {
    try {
      const deviceNames = state.selectedDevices.map(d => d.name);
      const response = await safewipeAPI.createSanitizationPlan(method, deviceNames);

      if (response.success) {
        return response.data;
      } else {
        throw new Error(response.error || 'Failed to create sanitization plan');
      }
    } catch (error) {
      toast.error(`Failed to create plan: ${error.message}`);
      throw error;
    }
  };

  const executeSanitization = async (method) => {
    try {
      const deviceNames = state.selectedDevices.map(d => d.name);
      const response = await safewipeAPI.executeSanitization(method, deviceNames);

      if (response.success) {
        dispatch({ type: 'SET_ACTIVE_OPERATION', payload: response.data });
        toast.success('Sanitization started successfully');
        return response.data;
      } else {
        throw new Error(response.error || 'Failed to start sanitization');
      }
    } catch (error) {
      toast.error(`Failed to start sanitization: ${error.message}`);
      throw error;
    }
  };

  const loadOperations = async () => {
    try {
      const response = await safewipeAPI.getAllOperations();
      if (response.success) {
        dispatch({ type: 'SET_OPERATIONS', payload: response.data });
      }
    } catch (error) {
      console.error('Failed to load operations:', error);
    }
  };

  const value = {
    ...state,
    loadDevices,
    selectDevice,
    deselectDevice,
    clearSelection,
    createSanitizationPlan,
    executeSanitization,
    loadOperations,
  };

  return (
    <DeviceContext.Provider value={value}>
      {children}
    </DeviceContext.Provider>
  );
}

export function useDevices() {
  const context = useContext(DeviceContext);
  if (!context) {
    throw new Error('useDevices must be used within a DeviceProvider');
  }
  return context;
}

export default DeviceContext;

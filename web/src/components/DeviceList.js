import React, { useEffect, useState } from 'react';
import { useDevices } from '../context/DeviceContext';
import {
  ComputerDesktopIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  InformationCircleIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';
import toast from 'react-hot-toast';

const driveTypeIcons = {
  HDD: '💾',
  SSD: '💿',
  Removable: '🔌',
  Unknown: '❓',
};

const driveTypeColors = {
  HDD: 'bg-blue-100 text-blue-800',
  SSD: 'bg-green-100 text-green-800',
  Removable: 'bg-yellow-100 text-yellow-800',
  Unknown: 'bg-gray-100 text-gray-800',
};

function DeviceList() {
  const {
    devices,
    recommendations,
    selectedDevices,
    loading,
    error,
    loadDevices,
    selectDevice,
    deselectDevice,
  } = useDevices();

  const [showSystemDrives, setShowSystemDrives] = useState(true);
  const [filterType, setFilterType] = useState('all');

  useEffect(() => {
    if (devices.length === 0) {
      loadDevices();
    }
  }, []);

  const handleDeviceSelect = (device) => {
    if (selectedDevices.find(d => d.name === device.name)) {
      deselectDevice(device.name);
    } else {
      selectDevice(device);
    }
  };

  const formatSize = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const filteredDevices = devices.filter(device => {
    if (!showSystemDrives && device.is_system_drive) return false;
    if (filterType !== 'all' && device.device_type !== filterType) return false;
    return true;
  });

  const deviceCounts = {
    total: devices.length,
    system: devices.filter(d => d.is_system_drive).length,
    sanitizable: devices.filter(d => !d.is_system_drive).length,
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600 mx-auto mb-4" />
          <p className="text-gray-600">Scanning storage devices...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-6">
        <div className="flex items-center">
          <ExclamationTriangleIcon className="h-6 w-6 text-red-600 mr-3" />
          <div>
            <h3 className="text-lg font-medium text-red-800">Error Loading Devices</h3>
            <p className="text-red-700 mt-1">{error}</p>
            <button
              onClick={loadDevices}
              className="mt-3 bg-red-600 text-white px-4 py-2 rounded-md hover:bg-red-700 transition-colors"
            >
              Retry Scan
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Storage Devices</h1>
          <p className="text-gray-600 mt-1">
            Detected {deviceCounts.total} devices ({deviceCounts.sanitizable} sanitizable)
          </p>
        </div>
        <button
          onClick={loadDevices}
          className="flex items-center space-x-2 bg-indigo-600 text-white px-4 py-2 rounded-lg hover:bg-indigo-700 transition-colors"
        >
          <ArrowPathIcon className="h-4 w-4" />
          <span>Refresh</span>
        </button>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <div className="flex items-center">
            <ComputerDesktopIcon className="h-8 w-8 text-blue-600" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Total Devices</p>
              <p className="text-2xl font-bold text-gray-900">{deviceCounts.total}</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <div className="flex items-center">
            <CheckCircleIcon className="h-8 w-8 text-green-600" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Sanitizable</p>
              <p className="text-2xl font-bold text-gray-900">{deviceCounts.sanitizable}</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <div className="flex items-center">
            <ExclamationTriangleIcon className="h-8 w-8 text-yellow-600" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">System Drives</p>
              <p className="text-2xl font-bold text-gray-900">{deviceCounts.system}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex items-center space-x-4 bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">Show:</label>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="border border-gray-300 rounded-md px-3 py-1 text-sm"
          >
            <option value="all">All Types</option>
            <option value="HDD">HDD Only</option>
            <option value="SSD">SSD Only</option>
            <option value="Removable">Removable Only</option>
          </select>
        </div>

        <label className="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={showSystemDrives}
            onChange={(e) => setShowSystemDrives(e.target.checked)}
            className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span className="text-sm text-gray-700">Show system drives</span>
        </label>
      </div>

      {/* Selected Devices Summary */}
      {selectedDevices.length > 0 && (
        <div className="bg-indigo-50 border border-indigo-200 rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <CheckCircleIcon className="h-5 w-5 text-indigo-600" />
              <span className="font-medium text-indigo-900">
                {selectedDevices.length} device(s) selected for sanitization
              </span>
            </div>
            <button
              onClick={() => selectedDevices.forEach(d => deselectDevice(d.name))}
              className="text-indigo-600 hover:text-indigo-800 text-sm font-medium"
            >
              Clear Selection
            </button>
          </div>
        </div>
      )}

      {/* Device Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {filteredDevices.map((device) => {
          const isSelected = selectedDevices.find(d => d.name === device.name);
          const recommendation = recommendations[device.name] || 'No recommendation available';

          return (
            <div
              key={device.name}
              className={`bg-white rounded-lg border-2 transition-all duration-200 card-hover ${
                isSelected
                  ? 'border-indigo-300 bg-indigo-50'
                  : device.is_system_drive
                  ? 'border-red-200 bg-red-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <div className="p-6">
                {/* Device Header */}
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    <div className="text-2xl">{driveTypeIcons[device.device_type]}</div>
                    <div>
                      <h3 className="font-semibold text-gray-900">{device.name}</h3>
                      <p className="text-sm text-gray-600">{device.path}</p>
                    </div>
                  </div>

                  <div className="flex flex-col items-end space-y-2">
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${driveTypeColors[device.device_type]}`}>
                      {device.device_type}
                    </span>
                    {device.is_system_drive && (
                      <span className="px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800">
                        SYSTEM
                      </span>
                    )}
                  </div>
                </div>

                {/* Device Info */}
                <div className="grid grid-cols-2 gap-4 mb-4">
                  <div>
                    <p className="text-sm text-gray-600">Size</p>
                    <p className="font-medium">{formatSize(device.size)}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-600">Interface</p>
                    <p className="font-medium">{device.interface}</p>
                  </div>
                </div>

                {/* Capabilities */}
                <div className="mb-4">
                  <p className="text-sm text-gray-600 mb-2">Capabilities</p>
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div className={`flex items-center space-x-1 ${device.capabilities.supports_ata_secure_erase ? 'text-green-600' : 'text-gray-400'}`}>
                      <span>{device.capabilities.supports_ata_secure_erase ? '✓' : '✗'}</span>
                      <span>ATA Secure Erase</span>
                    </div>
                    <div className={`flex items-center space-x-1 ${device.capabilities.supports_nvme_sanitize ? 'text-green-600' : 'text-gray-400'}`}>
                      <span>{device.capabilities.supports_nvme_sanitize ? '✓' : '✗'}</span>
                      <span>NVMe Sanitize</span>
                    </div>
                    <div className={`flex items-center space-x-1 ${device.capabilities.supports_crypto_erase ? 'text-green-600' : 'text-gray-400'}`}>
                      <span>{device.capabilities.supports_crypto_erase ? '✓' : '✗'}</span>
                      <span>Crypto Erase</span>
                    </div>
                  </div>
                </div>

                {/* Recommendation */}
                <div className="mb-4 p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-start space-x-2">
                    <InformationCircleIcon className="h-4 w-4 text-blue-600 mt-0.5" />
                    <div>
                      <p className="text-xs text-gray-600 mb-1">Recommendation</p>
                      <p className="text-sm text-gray-800">{recommendation}</p>
                    </div>
                  </div>
                </div>

                {/* Action Button */}
                <button
                  onClick={() => handleDeviceSelect(device)}
                  disabled={device.is_system_drive}
                  className={`w-full py-2 px-4 rounded-lg font-medium transition-colors ${
                    device.is_system_drive
                      ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                      : isSelected
                      ? 'bg-indigo-600 text-white hover:bg-indigo-700'
                      : 'bg-gray-900 text-white hover:bg-gray-800'
                  }`}
                >
                  {device.is_system_drive
                    ? 'System Drive - Cannot Sanitize'
                    : isSelected
                    ? 'Selected for Sanitization'
                    : 'Select for Sanitization'
                  }
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {filteredDevices.length === 0 && (
        <div className="text-center py-12">
          <ComputerDesktopIcon className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No devices found</h3>
          <p className="text-gray-600">
            {devices.length === 0
              ? 'No storage devices detected. Try refreshing the scan.'
              : 'No devices match the current filters.'}
          </p>
        </div>
      )}
    </div>
  );
}

export default DeviceList;

import React, { useState, useEffect } from 'react';
import { useDevices } from '../context/DeviceContext';
import {
  ChartBarIcon,
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';

const statusColors = {
  Starting: 'bg-blue-100 text-blue-800',
  InProgress: 'bg-yellow-100 text-yellow-800',
  Verifying: 'bg-purple-100 text-purple-800',
  Completed: 'bg-green-100 text-green-800',
  Failed: 'bg-red-100 text-red-800',
  Aborted: 'bg-gray-100 text-gray-800',
};

const statusIcons = {
  Starting: ClockIcon,
  InProgress: ArrowPathIcon,
  Verifying: ChartBarIcon,
  Completed: CheckCircleIcon,
  Failed: XCircleIcon,
  Aborted: ExclamationTriangleIcon,
};

function OperationMonitor() {
  const { operations, activeOperation, loadOperations } = useDevices();
  const [selectedOperation, setSelectedOperation] = useState(null);

  useEffect(() => {
    loadOperations();
    const interval = setInterval(loadOperations, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const operationsList = Object.entries(operations);

  const formatDuration = (seconds) => {
    if (!seconds) return 'N/A';
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp).toLocaleString();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Operation Monitor</h1>
          <p className="text-gray-600 mt-1">
            Track sanitization progress and view operation history
          </p>
        </div>
        <button
          onClick={loadOperations}
          className="flex items-center space-x-2 bg-indigo-600 text-white px-4 py-2 rounded-lg hover:bg-indigo-700 transition-colors"
        >
          <ArrowPathIcon className="h-4 w-4" />
          <span>Refresh</span>
        </button>
      </div>

      {/* Active Operation Alert */}
      {activeOperation && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <ArrowPathIcon className="h-5 w-5 text-blue-600 animate-spin" />
            <span className="font-medium text-blue-900">
              Active sanitization operation in progress
            </span>
            <span className="text-blue-700">ID: {activeOperation}</span>
          </div>
        </div>
      )}

      {/* Operations List */}
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Recent Operations</h2>
        </div>

        {operationsList.length === 0 ? (
          <div className="p-12 text-center">
            <ChartBarIcon className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">No operations found</h3>
            <p className="text-gray-600">
              Sanitization operations will appear here once started.
            </p>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {operationsList.map(([operationId, operation]) => {
              const StatusIcon = statusIcons[operation.status] || ChartBarIcon;
              const isSelected = selectedOperation === operationId;

              return (
                <div
                  key={operationId}
                  className={`p-6 cursor-pointer hover:bg-gray-50 transition-colors ${
                    isSelected ? 'bg-indigo-50' : ''
                  }`}
                  onClick={() => setSelectedOperation(isSelected ? null : operationId)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <StatusIcon className="h-6 w-6 text-gray-500" />
                      <div>
                        <h3 className="font-medium text-gray-900">
                          {operation.plan?.method || 'Unknown Method'} Sanitization
                        </h3>
                        <p className="text-sm text-gray-600">
                          {operation.plan?.devices?.length || 0} device(s) •
                          Started {formatTimestamp(operation.started_at)}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center space-x-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusColors[operation.status]}`}>
                        {operation.status}
                      </span>
                      <span className="text-sm text-gray-500">
                        {operation.duration ? formatDuration(operation.duration.secs) : 'In progress...'}
                      </span>
                    </div>
                  </div>

                  {/* Expanded Details */}
                  {isSelected && (
                    <div className="mt-6 grid grid-cols-1 lg:grid-cols-2 gap-6">
                      {/* Operation Details */}
                      <div>
                        <h4 className="font-medium text-gray-900 mb-3">Operation Details</h4>
                        <div className="space-y-2 text-sm">
                          <div className="flex justify-between">
                            <span className="text-gray-600">ID:</span>
                            <span className="font-mono text-xs">{operationId}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-gray-600">Method:</span>
                            <span className="font-medium">{operation.plan?.method || 'N/A'}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-gray-600">Started:</span>
                            <span>{formatTimestamp(operation.started_at)}</span>
                          </div>
                          {operation.completed_at && (
                            <div className="flex justify-between">
                              <span className="text-gray-600">Completed:</span>
                              <span>{formatTimestamp(operation.completed_at)}</span>
                            </div>
                          )}
                          <div className="flex justify-between">
                            <span className="text-gray-600">Success:</span>
                            <span className={operation.overall_success ? 'text-green-600' : 'text-red-600'}>
                              {operation.overall_success ? 'Yes' : 'No'}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Device Results */}
                      <div>
                        <h4 className="font-medium text-gray-900 mb-3">Device Results</h4>
                        <div className="space-y-2">
                          {operation.results?.map((result, index) => (
                            <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded">
                              <span className="text-sm font-medium">{result.device.name}</span>
                              <span className={`text-xs px-2 py-1 rounded ${statusColors[result.status]}`}>
                                {result.status}
                              </span>
                            </div>
                          )) || (
                            <p className="text-sm text-gray-500 italic">No results available</p>
                          )}
                        </div>
                      </div>

                      {/* Progress Information */}
                      {operation.progress && (
                        <div className="lg:col-span-2">
                          <h4 className="font-medium text-gray-900 mb-3">Progress</h4>
                          <div className="bg-gray-100 rounded-lg p-4">
                            <div className="flex justify-between text-sm mb-2">
                              <span>Pass {operation.progress.current_pass} of {operation.progress.total_passes}</span>
                              <span>
                                {((operation.progress.bytes_processed / operation.progress.total_bytes) * 100).toFixed(1)}%
                              </span>
                            </div>
                            <div className="w-full bg-gray-200 rounded-full h-2">
                              <div
                                className="progress-bar h-2 rounded-full"
                                style={{
                                  width: `${(operation.progress.bytes_processed / operation.progress.total_bytes) * 100}%`
                                }}
                              />
                            </div>
                            <div className="flex justify-between text-xs text-gray-600 mt-1">
                              <span>
                                {(operation.progress.bytes_processed / 1e9).toFixed(2)} GB processed
                              </span>
                              <span>
                                {(operation.progress.total_bytes / 1e9).toFixed(2)} GB total
                              </span>
                            </div>
                          </div>
                        </div>
                      )}

                      {/* Summary */}
                      {operation.summary && (
                        <div className="lg:col-span-2">
                          <h4 className="font-medium text-gray-900 mb-3">Summary</h4>
                          <p className="text-sm text-gray-700 bg-gray-50 p-3 rounded">
                            {operation.summary}
                          </p>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

export default OperationMonitor;

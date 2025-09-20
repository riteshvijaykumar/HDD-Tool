import React, { useState, useEffect } from 'react';
import { useDevices } from '../context/DeviceContext';
import {
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  ClockIcon,
  PlayIcon,
  XCircleIcon,
} from '@heroicons/react/24/outline';
import toast from 'react-hot-toast';

const sanitizationMethods = {
  clear: {
    name: 'Clear',
    description: 'Logical overwriting with data patterns',
    security: 'Medium',
    time: 'Long',
    color: 'blue',
    icon: '🔄',
    details: 'Overwrites data with zeros, ones, or random patterns. Suitable for most data protection needs.',
  },
  purge: {
    name: 'Purge',
    description: 'Hardware-based secure erase',
    security: 'High',
    time: 'Fast',
    color: 'green',
    icon: '⚡',
    details: 'Uses device capabilities like ATA Secure Erase or NVMe Sanitize for maximum security.',
  },
  destroy: {
    name: 'Destroy',
    description: 'Physical destruction guidelines',
    security: 'Maximum',
    time: 'Manual',
    color: 'red',
    icon: '🔨',
    details: 'Generates instructions for physical destruction of storage media.',
  },
};

function SanitizeWizard() {
  const {
    selectedDevices,
    createSanitizationPlan,
    executeSanitization,
    clearSelection,
  } = useDevices();

  const [currentStep, setCurrentStep] = useState(1);
  const [selectedMethod, setSelectedMethod] = useState('clear');
  const [sanitizationPlan, setSanitizationPlan] = useState(null);
  const [confirmChecks, setConfirmChecks] = useState({
    dataBackup: false,
    understand: false,
    responsibility: false,
  });
  const [isExecuting, setIsExecuting] = useState(false);

  useEffect(() => {
    if (selectedDevices.length === 0) {
      setCurrentStep(1);
    }
  }, [selectedDevices]);

  const handleMethodSelect = (method) => {
    setSelectedMethod(method);
    setSanitizationPlan(null);
  };

  const handleCreatePlan = async () => {
    try {
      const plan = await createSanitizationPlan(selectedMethod);
      setSanitizationPlan(plan);
      setCurrentStep(3);
    } catch (error) {
      // Error handled in context
    }
  };

  const handleConfirmChange = (check) => {
    setConfirmChecks(prev => ({ ...prev, [check]: !prev[check] }));
  };

  const allConfirmsChecked = Object.values(confirmChecks).every(Boolean);

  const handleExecute = async () => {
    if (!allConfirmsChecked) {
      toast.error('Please confirm all safety checks before proceeding');
      return;
    }

    try {
      setIsExecuting(true);
      await executeSanitization(selectedMethod);
      toast.success('Sanitization started successfully!');
      clearSelection();
      setCurrentStep(1);
      setSanitizationPlan(null);
      setConfirmChecks({
        dataBackup: false,
        understand: false,
        responsibility: false,
      });
    } catch (error) {
      // Error handled in context
    } finally {
      setIsExecuting(false);
    }
  };

  const formatDuration = (seconds) => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  // Step 1: Device Selection
  if (selectedDevices.length === 0) {
    return (
      <div className="max-w-4xl mx-auto">
        <div className="text-center py-12">
          <ShieldCheckIcon className="h-16 w-16 text-gray-400 mx-auto mb-6" />
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Sanitization Wizard</h1>
          <p className="text-gray-600 mb-8">
            Select storage devices from the Devices tab to begin the sanitization process.
          </p>
          <button
            onClick={() => window.location.href = '/devices'}
            className="bg-indigo-600 text-white px-6 py-3 rounded-lg hover:bg-indigo-700 transition-colors"
          >
            Go to Devices
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Sanitization Wizard</h1>
        <p className="text-gray-600 mt-1">
          Secure data erasure for {selectedDevices.length} selected device(s)
        </p>
      </div>

      {/* Progress Steps */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-6">
          {[
            { step: 1, title: 'Review Devices', active: currentStep >= 1 },
            { step: 2, title: 'Choose Method', active: currentStep >= 2 },
            { step: 3, title: 'Review Plan', active: currentStep >= 3 },
            { step: 4, title: 'Execute', active: currentStep >= 4 },
          ].map((item, index) => (
            <React.Fragment key={item.step}>
              <div className="flex items-center">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                    item.active
                      ? 'bg-indigo-600 text-white'
                      : 'bg-gray-200 text-gray-600'
                  }`}
                >
                  {item.step}
                </div>
                <span
                  className={`ml-2 text-sm font-medium ${
                    item.active ? 'text-indigo-600' : 'text-gray-500'
                  }`}
                >
                  {item.title}
                </span>
              </div>
              {index < 3 && (
                <div className={`flex-1 h-0.5 mx-4 ${
                  currentStep > item.step ? 'bg-indigo-600' : 'bg-gray-200'
                }`} />
              )}
            </React.Fragment>
          ))}
        </div>
      </div>

      {/* Step Content */}
      {currentStep === 1 && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Selected Devices</h2>

          <div className="space-y-3 mb-6">
            {selectedDevices.map((device) => (
              <div key={device.name} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                <div className="flex items-center space-x-3">
                  <div className="text-xl">
                    {device.device_type === 'HDD' ? '💾' : device.device_type === 'SSD' ? '💿' : '🔌'}
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">{device.name}</p>
                    <p className="text-sm text-gray-600">
                      {device.device_type} • {(device.size / 1e9).toFixed(1)} GB
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => clearSelection()}
                  className="text-red-600 hover:text-red-800 text-sm"
                >
                  Remove
                </button>
              </div>
            ))}
          </div>

          <div className="flex justify-end">
            <button
              onClick={() => setCurrentStep(2)}
              className="bg-indigo-600 text-white px-6 py-2 rounded-lg hover:bg-indigo-700 transition-colors"
            >
              Continue
            </button>
          </div>
        </div>
      )}

      {currentStep === 2 && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Choose Sanitization Method</h2>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            {Object.entries(sanitizationMethods).map(([key, method]) => (
              <div
                key={key}
                onClick={() => handleMethodSelect(key)}
                className={`cursor-pointer p-4 rounded-lg border-2 transition-all ${
                  selectedMethod === key
                    ? `border-${method.color}-500 bg-${method.color}-50`
                    : 'border-gray-200 hover:border-gray-300'
                }`}
              >
                <div className="text-2xl mb-2">{method.icon}</div>
                <h3 className="font-semibold text-gray-900 mb-1">{method.name}</h3>
                <p className="text-sm text-gray-600 mb-3">{method.description}</p>

                <div className="space-y-1">
                  <div className="flex justify-between text-xs">
                    <span className="text-gray-500">Security:</span>
                    <span className={`font-medium ${
                      method.security === 'Maximum' ? 'text-red-600' :
                      method.security === 'High' ? 'text-green-600' : 'text-blue-600'
                    }`}>
                      {method.security}
                    </span>
                  </div>
                  <div className="flex justify-between text-xs">
                    <span className="text-gray-500">Time:</span>
                    <span className="font-medium text-gray-700">{method.time}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {selectedMethod && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
              <h4 className="font-medium text-blue-900 mb-2">Method Details</h4>
              <p className="text-blue-800 text-sm">
                {sanitizationMethods[selectedMethod].details}
              </p>
            </div>
          )}

          <div className="flex justify-between">
            <button
              onClick={() => setCurrentStep(1)}
              className="bg-gray-300 text-gray-700 px-6 py-2 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Back
            </button>
            <button
              onClick={handleCreatePlan}
              className="bg-indigo-600 text-white px-6 py-2 rounded-lg hover:bg-indigo-700 transition-colors"
            >
              Create Plan
            </button>
          </div>
        </div>
      )}

      {currentStep === 3 && sanitizationPlan && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Review Sanitization Plan</h2>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
            <div>
              <h3 className="font-medium text-gray-900 mb-3">Plan Summary</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">Method:</span>
                  <span className="font-medium">{sanitizationMethods[selectedMethod].name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Devices:</span>
                  <span className="font-medium">{sanitizationPlan.devices.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Estimated Time:</span>
                  <span className="font-medium">
                    {formatDuration(sanitizationPlan.estimated_duration.secs)}
                  </span>
                </div>
              </div>
            </div>

            <div>
              <h3 className="font-medium text-gray-900 mb-3">Safety Warnings</h3>
              <div className="space-y-2">
                {sanitizationPlan.safety_warnings.map((warning, index) => (
                  <div key={index} className="flex items-start space-x-2">
                    <ExclamationTriangleIcon className="h-4 w-4 text-yellow-600 mt-0.5 flex-shrink-0" />
                    <span className="text-sm text-gray-700">{warning}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Confirmation Checks */}
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
            <h3 className="font-medium text-yellow-900 mb-3">Safety Confirmation</h3>
            <div className="space-y-3">
              {[
                {
                  key: 'dataBackup',
                  label: 'I have backed up all important data from the selected devices',
                },
                {
                  key: 'understand',
                  label: 'I understand this operation will permanently destroy all data',
                },
                {
                  key: 'responsibility',
                  label: 'I take full responsibility for this sanitization operation',
                },
              ].map((check) => (
                <label key={check.key} className="flex items-start space-x-3">
                  <input
                    type="checkbox"
                    checked={confirmChecks[check.key]}
                    onChange={() => handleConfirmChange(check.key)}
                    className="mt-1 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                  />
                  <span className="text-sm text-gray-700">{check.label}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="flex justify-between">
            <button
              onClick={() => setCurrentStep(2)}
              className="bg-gray-300 text-gray-700 px-6 py-2 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Back
            </button>
            <button
              onClick={handleExecute}
              disabled={!allConfirmsChecked || isExecuting}
              className={`px-6 py-2 rounded-lg transition-colors flex items-center space-x-2 ${
                allConfirmsChecked && !isExecuting
                  ? 'bg-red-600 text-white hover:bg-red-700'
                  : 'bg-gray-300 text-gray-500 cursor-not-allowed'
              }`}
            >
              <PlayIcon className="h-4 w-4" />
              <span>{isExecuting ? 'Starting...' : 'Execute Sanitization'}</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default SanitizeWizard;

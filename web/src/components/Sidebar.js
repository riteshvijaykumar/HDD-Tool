import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  ComputerDesktopIcon,
  TrashIcon,
  ChartBarIcon,
  DocumentTextIcon,
  Cog6ToothIcon,
} from '@heroicons/react/24/outline';

const navigationItems = [
  {
    name: 'Devices',
    href: '/devices',
    icon: ComputerDesktopIcon,
    description: 'Scan & View Storage Devices',
  },
  {
    name: 'Sanitize',
    href: '/sanitize',
    icon: TrashIcon,
    description: 'Secure Data Erasure',
  },
  {
    name: 'Monitor',
    href: '/monitor',
    icon: ChartBarIcon,
    description: 'Operation Progress',
  },
  {
    name: 'Reports',
    href: '/reports',
    icon: DocumentTextIcon,
    description: 'Compliance Reports',
  },
];

function Sidebar({ isOpen, onToggle, currentView, onViewChange }) {
  const navigate = useNavigate();
  const location = useLocation();

  const handleNavigation = (item) => {
    navigate(item.href);
    onViewChange(item.name.toLowerCase());
  };

  return (
    <div
      className={`fixed inset-y-0 left-0 z-50 bg-white shadow-lg border-r border-gray-200 transition-all duration-300 ${
        isOpen ? 'w-64' : 'w-16'
      }`}
    >
      <div className="flex flex-col h-full">
        {/* Logo Section */}
        <div className="flex items-center justify-center h-16 border-b border-gray-200">
          {isOpen ? (
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-gradient-safewipe rounded-lg flex items-center justify-center">
                <span className="text-white font-bold text-sm">SW</span>
              </div>
              <span className="font-semibold text-gray-900">SafeWipe</span>
            </div>
          ) : (
            <div className="w-8 h-8 bg-gradient-safewipe rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">SW</span>
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 px-2 py-4 space-y-2">
          {navigationItems.map((item) => {
            const isActive = location.pathname === item.href;
            return (
              <button
                key={item.name}
                onClick={() => handleNavigation(item)}
                className={`w-full flex items-center px-3 py-3 text-sm font-medium rounded-lg transition-all duration-200 ${
                  isActive
                    ? 'bg-indigo-50 text-indigo-700 border border-indigo-200'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <item.icon
                  className={`h-5 w-5 ${isActive ? 'text-indigo-600' : 'text-gray-500'}`}
                />
                {isOpen && (
                  <div className="ml-3 flex-1 text-left">
                    <div className="font-medium">{item.name}</div>
                    <div className="text-xs text-gray-500 mt-0.5">
                      {item.description}
                    </div>
                  </div>
                )}
              </button>
            );
          })}
        </nav>

        {/* Settings */}
        <div className="border-t border-gray-200 p-2">
          <button
            className={`w-full flex items-center px-3 py-3 text-sm font-medium text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-lg transition-colors ${
              !isOpen && 'justify-center'
            }`}
          >
            <Cog6ToothIcon className="h-5 w-5 text-gray-500" />
            {isOpen && <span className="ml-3">Settings</span>}
          </button>
        </div>

        {/* Collapse Button */}
        <div className="border-t border-gray-200 p-2">
          <button
            onClick={onToggle}
            className="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-500 hover:text-gray-700 transition-colors"
          >
            {isOpen ? '← Collapse' : '→'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default Sidebar;

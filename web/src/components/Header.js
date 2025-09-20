import React from 'react';
import { Bars3Icon, ShieldCheckIcon } from '@heroicons/react/24/outline';

function Header({ onSidebarToggle }) {
  return (
    <header className="bg-white shadow-sm border-b border-gray-200">
      <div className="flex items-center justify-between px-6 py-4">
        <div className="flex items-center space-x-4">
          <button
            onClick={onSidebarToggle}
            className="p-2 rounded-md text-gray-500 hover:text-gray-600 hover:bg-gray-100 transition-colors"
          >
            <Bars3Icon className="h-6 w-6" />
          </button>

          <div className="flex items-center space-x-3">
            <ShieldCheckIcon className="h-8 w-8 text-indigo-600" />
            <div>
              <h1 className="text-xl font-bold text-gray-900">SafeWipe</h1>
              <p className="text-sm text-gray-500">NIST SP 800-88 Compliant</p>
            </div>
          </div>
        </div>

        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <div className="h-2 w-2 bg-green-500 rounded-full animate-pulse"></div>
            <span className="text-sm text-gray-600">Engine Online</span>
          </div>

          <div className="bg-gradient-safewipe text-white px-3 py-1 rounded-full text-sm font-medium">
            Professional Edition
          </div>
        </div>
      </div>
    </header>
  );
}

export default Header;

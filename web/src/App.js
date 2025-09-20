import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import Header from './components/Header';
import Sidebar from './components/Sidebar';
import DeviceList from './components/DeviceList';
import SanitizeWizard from './components/SanitizeWizard';
import OperationMonitor from './components/OperationMonitor';
import Reports from './components/Reports';
import { DeviceProvider } from './context/DeviceContext';
import './index.css';

function App() {
  const [currentView, setCurrentView] = useState('devices');
  const [sidebarOpen, setSidebarOpen] = useState(true);

  return (
    <DeviceProvider>
      <Router>
        <div className="flex h-screen bg-gray-50">
          <Toaster position="top-right" />

          <Sidebar
            isOpen={sidebarOpen}
            onToggle={() => setSidebarOpen(!sidebarOpen)}
            currentView={currentView}
            onViewChange={setCurrentView}
          />

          <div className={`flex-1 flex flex-col overflow-hidden transition-all duration-300 ${
            sidebarOpen ? 'ml-64' : 'ml-16'
          }`}>
            <Header
              onSidebarToggle={() => setSidebarOpen(!sidebarOpen)}
            />

            <main className="flex-1 overflow-auto p-6">
              <Routes>
                <Route path="/" element={<Navigate to="/devices" replace />} />
                <Route path="/devices" element={<DeviceList />} />
                <Route path="/sanitize" element={<SanitizeWizard />} />
                <Route path="/monitor" element={<OperationMonitor />} />
                <Route path="/reports" element={<Reports />} />
              </Routes>
            </main>
          </div>
        </div>
      </Router>
    </DeviceProvider>
  );
}

export default App;

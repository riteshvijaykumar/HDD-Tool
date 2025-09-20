# SafeWipe Web GUI

A modern React-based GUI for the SafeWipe data sanitization engine.

## Quick Start

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm start
```

3. Build for production:
```bash
npm run build
```

## Features

- **Device Discovery**: Real-time scanning and visualization of storage devices
- **Smart Recommendations**: AI-powered sanitization method suggestions
- **Progress Monitoring**: Live progress tracking with detailed analytics
- **Compliance Reporting**: NIST SP 800-88 compliant reports generation
- **Safety Controls**: System drive protection and confirmation workflows

## Architecture

The GUI communicates with the Rust SafeWipe engine via HTTP API:
- Frontend: React 18 + Tailwind CSS
- Backend: Rust Axum web server
- Communication: REST API + Server-Sent Events for real-time updates

## Usage

1. **Scan Devices**: Navigate to the Devices tab to discover storage devices
2. **Select for Sanitization**: Choose devices (system drives are protected)
3. **Choose Method**: Select Clear, Purge, or Destroy sanitization method
4. **Review Plan**: Confirm sanitization plan and safety warnings
5. **Execute**: Start sanitization with real-time progress monitoring
6. **Generate Reports**: Download compliance reports in JSON or text format

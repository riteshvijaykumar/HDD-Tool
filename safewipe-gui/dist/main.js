// main.js - SafeWipe GUI logic with Tauri backend integration
const { invoke } = window.__TAURI__.tauri;

// UI Elements
const deviceSelect = document.getElementById('deviceSelect');
const methodSelect = document.getElementById('methodSelect');
const cleanTypeSelect = document.getElementById('cleanTypeSelect');
const startBtn = document.getElementById('startWipeBtn');
const progressBar = document.getElementById('progressBar');
const progressText = document.getElementById('progressText');
const progressContainer = document.getElementById('progressContainer');
const statusMessage = document.getElementById('statusMessage');
const certSection = document.getElementById('certificateSection');
const downloadPdfBtn = document.getElementById('downloadPdfBtn');
const downloadJsonBtn = document.getElementById('downloadJsonBtn');
const verifyLink = document.getElementById('verifyLink');

let selectedDevice = null;
let operationId = null;

// Populate device dropdown from backend
async function loadDevices() {
  deviceSelect.innerHTML = '';
  try {
    const resp = await invoke('list_devices');
    if (resp && resp.data && resp.data.length > 0) {
      resp.data.forEach(device => {
        const opt = document.createElement('option');
        opt.value = device.id;
        opt.textContent = `${device.name} (${device.size})`;
        deviceSelect.appendChild(opt);
      });
      selectedDevice = deviceSelect.value;
      startBtn.disabled = false;
    } else {
      deviceSelect.innerHTML = '<option>No devices found</option>';
      selectedDevice = null;
      startBtn.disabled = true;
      statusMessage.textContent = 'No devices found. Please connect a device or check backend.';
    }
  } catch (e) {
    deviceSelect.innerHTML = '<option>Failed to load devices</option>';
    selectedDevice = null;
    startBtn.disabled = true;
    statusMessage.textContent = 'Failed to load devices. Backend may be unavailable.';
  }
}

deviceSelect.addEventListener('change', () => {
  selectedDevice = deviceSelect.value;
  startBtn.disabled = !selectedDevice || deviceSelect.options[0].text.includes('No devices') || deviceSelect.options[0].text.includes('Failed');
});

// Populate method dropdown (optional, since static in HTML)
function loadMethods() {
  // Optionally, repopulate or update methodSelect here if needed
}

// Populate clean type dropdown (optional, since static in HTML)
function loadCleanTypes() {
  // Optionally, repopulate or update cleanTypeSelect here if needed
}

// Start wipe process
startBtn.onclick = async function() {
  if (!selectedDevice) {
    statusMessage.textContent = 'Please select a device.';
    return;
  }
  const method = methodSelect.value;
  const cleanType = cleanTypeSelect.value;
  progressContainer.classList.remove('hidden');
  statusMessage.textContent = 'Wiping in progress...';
  progressBar.style.width = '0%';
  progressText.textContent = '0%';
  certSection.classList.add('hidden');
  try {
    const resp = await invoke('start_wipe', { deviceId: selectedDevice, method, cleanType });
    if (resp && resp.data) {
      operationId = resp.data;
      pollProgress();
    } else {
      statusMessage.textContent = resp.error || 'Failed to start wipe.';
    }
  } catch (e) {
    statusMessage.textContent = 'Error starting wipe.';
  }
};

// Poll progress
async function pollProgress() {
  let done = false;
  while (!done) {
    try {
      const updates = await invoke('get_progress_updates');
      if (updates && updates.length > 0) {
        const last = updates[updates.length - 1];
        progressBar.style.width = `${last.percent}%`;
        progressText.textContent = `${last.percent}%`;
        if (last.percent >= 100) {
          done = true;
          statusMessage.textContent = 'Wipe complete!';
          certSection.classList.remove('hidden');
        }
      }
    } catch (e) {
      statusMessage.textContent = 'Error fetching progress.';
      break;
    }
    await new Promise(r => setTimeout(r, 1000));
  }
}

// Download certificate/report
if (downloadPdfBtn) downloadPdfBtn.onclick = () => exportReport('pdf');
if (downloadJsonBtn) downloadJsonBtn.onclick = () => exportReport('json');

async function exportReport(format) {
  if (!operationId) return;
  try {
    const resp = await invoke('export_report', { operationId, format });
    if (resp && resp.success) {
      statusMessage.textContent = `Certificate exported as ${format.toUpperCase()}`;
    } else {
      statusMessage.textContent = resp.error || 'Failed to export certificate.';
    }
  } catch (e) {
    statusMessage.textContent = 'Error exporting certificate.';
  }
}

// Verification link (placeholder)
if (verifyLink) verifyLink.onclick = () => {
  statusMessage.textContent = 'Verification feature coming soon.';
};

// On load
window.addEventListener('DOMContentLoaded', () => {
  loadDevices();
  // loadMethods(); // Not needed if static
  // loadCleanTypes(); // Not needed if static
});

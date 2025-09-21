const invoke = window.__TAURI__.invoke;

const devicePathInput = document.getElementById('devicePath');
const methodSelect = document.getElementById('method');
const wipeBtn = document.getElementById('wipeBtn');
const reportBtn = document.getElementById('reportBtn');
const verifyBtn = document.getElementById('verifyBtn');
const resultDiv = document.getElementById('result');

wipeBtn.onclick = async () => {
  resultDiv.textContent = 'Wiping...';
  try {
    const res = await invoke('tauri_wipe_device', {
      devicePath: devicePathInput.value,
      method: methodSelect.value
    });
    resultDiv.textContent = JSON.stringify(res, null, 2);
  } catch (e) {
    resultDiv.textContent = 'Error: ' + e;
  }
};

reportBtn.onclick = async () => {
  resultDiv.textContent = 'Generating report...';
  try {
    const res = await invoke('tauri_generate_wipe_report', {
      devicePath: devicePathInput.value
    });
    resultDiv.textContent = res;
  } catch (e) {
    resultDiv.textContent = 'Error: ' + e;
  }
};

verifyBtn.onclick = async () => {
  resultDiv.textContent = 'Verifying...';
  try {
    const res = await invoke('tauri_verify_device_wipe', {
      devicePath: devicePathInput.value
    });
    resultDiv.textContent = 'Verification result: ' + res;
  } catch (e) {
    resultDiv.textContent = 'Error: ' + e;
  }
};
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>SafeWipe GUI</title>
  <script defer src="main.js"></script>
  <style>
    body { font-family: sans-serif; margin: 2em; }
    input, select, button { margin: 0.5em; }
    #result { margin-top: 1em; white-space: pre-wrap; }
  </style>
</head>
<body>
  <h1>SafeWipe GUI</h1>
  <label>Device Path: <input id="devicePath" type="text" value="/dev/null" /></label><br>
  <label>Method:
    <select id="method">
      <option value="Clear">Clear</option>
      <option value="Purge">Purge</option>
      <option value="Destroy">Destroy</option>
    </select>
  </label><br>
  <button id="wipeBtn">Wipe Device</button>
  <button id="reportBtn">Generate Report</button>
  <button id="verifyBtn">Verify Wipe</button>
  <div id="result"></div>
</body>
</html>


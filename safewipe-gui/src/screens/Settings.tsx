import React, { useState } from 'react';
import { Box, Typography, Paper, FormControlLabel, Switch, Button, MenuItem, TextField, Select, InputLabel, FormControl, Stack } from '@mui/material';

const Settings: React.FC = () => {
  const [darkMode, setDarkMode] = useState(false);
  const [defaultMethod, setDefaultMethod] = useState('clear');
  const [defaultPasses, setDefaultPasses] = useState(1);
  const [language, setLanguage] = useState('en');
  const [logLevel, setLogLevel] = useState('basic');
  const [reportLocation, setReportLocation] = useState('~/SafeWipeReports');

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Settings</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={3}>
          <Box flex={1} minWidth={220}>
            <FormControlLabel
              control={<Switch checked={darkMode} onChange={e => setDarkMode(e.target.checked)} />}
              label="Dark Mode"
            />
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Default Wipe Method</InputLabel>
              <Select
                value={defaultMethod}
                label="Default Wipe Method"
                onChange={e => setDefaultMethod(e.target.value)}
              >
                <MenuItem value="clear">Clear (Overwrite)</MenuItem>
                <MenuItem value="purge">Purge (Secure Erase)</MenuItem>
                <MenuItem value="destroy">Destroy (Instructions Only)</MenuItem>
              </Select>
            </FormControl>
            <TextField
              label="Default Passes"
              type="number"
              inputProps={{ min: 1, max: 35 }}
              value={defaultPasses}
              onChange={e => setDefaultPasses(Number(e.target.value))}
              fullWidth
              sx={{ mt: 2 }}
            />
          </Box>
          <Box flex={1} minWidth={220}>
            <FormControl fullWidth>
              <InputLabel>Language</InputLabel>
              <Select
                value={language}
                label="Language"
                onChange={e => setLanguage(e.target.value)}
              >
                <MenuItem value="en">English</MenuItem>
                <MenuItem value="hi">Hindi</MenuItem>
                <MenuItem value="ta">Tamil</MenuItem>
                {/* Add more languages as needed */}
              </Select>
            </FormControl>
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Log Level</InputLabel>
              <Select
                value={logLevel}
                label="Log Level"
                onChange={e => setLogLevel(e.target.value)}
              >
                <MenuItem value="basic">Basic</MenuItem>
                <MenuItem value="detailed">Detailed</MenuItem>
              </Select>
            </FormControl>
            <TextField
              label="Report Storage Location"
              value={reportLocation}
              onChange={e => setReportLocation(e.target.value)}
              fullWidth
              sx={{ mt: 2 }}
            />
          </Box>
        </Stack>
        <Box mt={4} textAlign="right">
          <Button variant="contained" color="primary">Save Settings</Button>
        </Box>
      </Paper>
      <Box mt={2} textAlign="right">
        <Button variant="outlined" color="primary" onClick={() => window.dispatchEvent(new CustomEvent('navigate', { detail: 'dashboard' }))}>
          Back to Dashboard
        </Button>
      </Box>
    </Box>
  );
};

export default Settings;

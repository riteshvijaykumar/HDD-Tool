import React, { useEffect, useState } from 'react';
import { Box, Typography, Paper, FormControlLabel, Switch, Button, MenuItem, TextField, Select, InputLabel, FormControl, Stack } from '@mui/material';
import { fetchSettings, updateSettings } from '../api/settings';

const Settings: React.FC = () => {
  const [settings, setSettings] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchSettings().then((data) => {
      setSettings(data);
      setLoading(false);
    });
  }, []);

  const handleChange = (key: string, value: any) => {
    setSettings((prev: any) => ({ ...prev, [key]: value }));
  };

  const handleSave = () => {
    updateSettings(settings);
  };

  if (loading) return <Typography>Loading...</Typography>;
  if (!settings) return <Typography color="error">Failed to load settings.</Typography>;

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Settings</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={3}>
          <Box flex={1} minWidth={220}>
            <FormControlLabel
              control={<Switch checked={settings.darkMode} onChange={e => handleChange('darkMode', e.target.checked)} />}
              label="Dark Mode"
            />
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Default Wipe Method</InputLabel>
              <Select
                value={settings.defaultMethod}
                label="Default Wipe Method"
                onChange={e => handleChange('defaultMethod', e.target.value)}
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
              value={settings.defaultPasses}
              onChange={e => handleChange('defaultPasses', Number(e.target.value))}
              fullWidth
              sx={{ mt: 2 }}
            />
          </Box>
          <Box flex={1} minWidth={220}>
            <FormControl fullWidth>
              <InputLabel>Language</InputLabel>
              <Select
                value={settings.language}
                label="Language"
                onChange={e => handleChange('language', e.target.value)}
              >
                <MenuItem value="en">English</MenuItem>
                <MenuItem value="es">Spanish</MenuItem>
                {/* Add more languages as needed */}
              </Select>
            </FormControl>
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Log Level</InputLabel>
              <Select
                value={settings.logLevel}
                label="Log Level"
                onChange={e => handleChange('logLevel', e.target.value)}
              >
                <MenuItem value="basic">Basic</MenuItem>
                <MenuItem value="verbose">Verbose</MenuItem>
                <MenuItem value="debug">Debug</MenuItem>
              </Select>
            </FormControl>
            <TextField
              label="Report Location"
              value={settings.reportLocation}
              onChange={e => handleChange('reportLocation', e.target.value)}
              fullWidth
              sx={{ mt: 2 }}
            />
          </Box>
        </Stack>
        <Box mt={3} textAlign="right">
          <Button variant="contained" color="primary" onClick={handleSave}>Save Settings</Button>
        </Box>
      </Paper>
    </Box>
  );
};

export default Settings;

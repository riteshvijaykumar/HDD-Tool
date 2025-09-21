import React, { useState, useEffect } from 'react';
import { Box, Typography, Card, CardContent, Button, Chip, Alert, ToggleButton, ToggleButtonGroup, InputAdornment, TextField, CircularProgress, Stack } from '@mui/material';
import StorageIcon from '@mui/icons-material/Storage';
import UsbIcon from '@mui/icons-material/Usb';
import SdStorageIcon from '@mui/icons-material/SdStorage';
import SearchIcon from '@mui/icons-material/Search';
import { fetchDevices, DeviceInfo } from '../api/devices';

const typeIcon = (type: string) => {
  switch (type) {
    case 'SSD':
    case 'HDD':
      return <StorageIcon color="primary" />;
    case 'USB':
      return <UsbIcon color="secondary" />;
    case 'SD':
      return <SdStorageIcon color="success" />;
    default:
      return <StorageIcon />;
  }
};

const DeviceSelection: React.FC = () => {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');

  useEffect(() => {
    fetchDevices()
      .then(setDevices)
      .catch(() => setError('Failed to load devices'))
      .finally(() => setLoading(false));
  }, []);

  const filteredDevices = devices.filter(d =>
    (filter === 'all' || d.type === filter) &&
    (d.name.toLowerCase().includes(search.toLowerCase()) || d.path.toLowerCase().includes(search.toLowerCase()))
  );

  const handleNext = () => {
    window.dispatchEvent(new CustomEvent('navigate', { detail: 'wipe-method' }));
  };

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Select Device to Wipe</Typography>
      <Alert severity="error" sx={{ mb: 2 }}>
        This operation is <b>irreversible</b>. All data on the selected device will be permanently destroyed.
      </Alert>
      <Box display="flex" alignItems="center" gap={2} mb={2}>
        <ToggleButtonGroup
          value={filter}
          exclusive
          onChange={(_, v) => setFilter(v || 'all')}
          aria-label="device type filter"
        >
          <ToggleButton value="all">All</ToggleButton>
          <ToggleButton value="SSD">SSD</ToggleButton>
          <ToggleButton value="HDD">HDD</ToggleButton>
          <ToggleButton value="USB">USB</ToggleButton>
          <ToggleButton value="SD">SD</ToggleButton>
        </ToggleButtonGroup>
        <TextField
          size="small"
          placeholder="Search devices..."
          value={search}
          onChange={e => setSearch(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon />
              </InputAdornment>
            ),
          }}
        />
      </Box>
      {loading ? (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight={200}>
          <CircularProgress />
        </Box>
      ) : error ? (
        <Alert severity="error">{error}</Alert>
      ) : filteredDevices.length === 0 ? (
        <Alert severity="info">No devices found.</Alert>
      ) : (
        <Stack direction={{ xs: 'column', sm: 'row' }} flexWrap="wrap" spacing={2} useFlexGap>
          {filteredDevices.map(device => (
            <Card
              key={device.path}
              variant={selected === device.path ? 'outlined' : undefined}
              sx={{ cursor: 'pointer', borderColor: selected === device.path ? 'primary.main' : undefined, minWidth: 260, flex: '1 1 260px' }}
              onClick={() => setSelected(device.path)}
            >
              <CardContent>
                <Box display="flex" alignItems="center" gap={2}>
                  {typeIcon(device.type)}
                  <Box>
                    <Typography variant="h6">{device.name}</Typography>
                    <Typography variant="body2" color="text.secondary">{device.path}</Typography>
                    <Chip label={device.size} size="small" sx={{ mt: 1 }} />
                    {device.removable && <Chip label="Removable" color="warning" size="small" sx={{ mt: 1, ml: 1 }} />}
                  </Box>
                </Box>
              </CardContent>
            </Card>
          ))}
        </Stack>
      )}
      <Box mt={4} display="flex" justifyContent="flex-end">
        <Button variant="contained" color="primary" disabled={!selected} onClick={handleNext}>
          Next
        </Button>
      </Box>
    </Box>
  );
};

export default DeviceSelection;

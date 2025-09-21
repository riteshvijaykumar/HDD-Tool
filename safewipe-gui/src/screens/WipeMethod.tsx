import React, { useState } from 'react';
import { Box, Typography, Button, Tooltip, Radio, RadioGroup, FormControlLabel, FormControl, FormLabel, TextField, MenuItem, IconButton, Stack } from '@mui/material';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';

const wipeMethods = [
  { value: 'clear', label: 'Clear (Overwrite)', tooltip: 'Overwrites all user data with zeros or random data.' },
  { value: 'purge', label: 'Purge (Secure Erase)', tooltip: 'Uses device firmware to securely erase all data.' },
  { value: 'destroy', label: 'Destroy (Instructions Only)', tooltip: 'Provides physical destruction instructions.' },
];

const overwritePatterns = [
  { value: 'zeros', label: 'Zeros' },
  { value: 'random', label: 'Random' },
  { value: 'nist', label: 'NIST Standard' },
  { value: 'dod', label: 'DoD 5220.22-M' },
  { value: 'gutmann', label: 'Gutmann' },
];

const WipeMethod: React.FC = () => {
  const [method, setMethod] = useState('clear');
  const [passes, setPasses] = useState(1);
  const [pattern, setPattern] = useState('zeros');

  const handleNext = () => {
    window.dispatchEvent(new CustomEvent('navigate', { detail: 'wipe-progress' }));
  };

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Select Wipe Method</Typography>
      <FormControl component="fieldset" sx={{ mb: 3 }}>
        <FormLabel component="legend">Wipe Method</FormLabel>
        <RadioGroup
          value={method}
          onChange={e => setMethod(e.target.value)}
          row
        >
          {wipeMethods.map((m) => (
            <FormControlLabel
              key={m.value}
              value={m.value}
              control={<Radio />}
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  {m.label}
                  <Tooltip title={m.tooltip} placement="top">
                    <IconButton size="small"><InfoOutlinedIcon fontSize="small" /></IconButton>
                  </Tooltip>
                </Box>
              }
            />
          ))}
        </RadioGroup>
      </FormControl>
      {method === 'clear' && (
        <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} alignItems="center" sx={{ mb: 3 }}>
          <TextField
            label="Number of Passes"
            type="number"
            inputProps={{ min: 1, max: 35 }}
            value={passes}
            onChange={e => setPasses(Number(e.target.value))}
            sx={{ minWidth: 160 }}
          />
          <TextField
            label="Overwrite Pattern"
            select
            value={pattern}
            onChange={e => setPattern(e.target.value)}
            sx={{ minWidth: 180 }}
          >
            {overwritePatterns.map((p) => (
              <MenuItem key={p.value} value={p.value}>{p.label}</MenuItem>
            ))}
          </TextField>
        </Stack>
      )}
      {method === 'destroy' && (
        <Box mb={3}>
          <Typography color="error" variant="body1">
            For physical destruction, follow manufacturer and e-waste guidelines. This cannot be reversed.
          </Typography>
        </Box>
      )}
      <Box mt={4} textAlign="right">
        <Button
          variant="contained"
          color="primary"
          size="large"
          onClick={handleNext}
        >
          Start Wipe
        </Button>
      </Box>
    </Box>
  );
};

export default WipeMethod;

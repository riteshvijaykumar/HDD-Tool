import React, { useEffect, useState } from 'react';
import { Box, Typography, Button, Tooltip, Radio, RadioGroup, FormControlLabel, FormControl, FormLabel, TextField, MenuItem, IconButton } from '@mui/material';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';
import { fetchWipeMethods } from '../api/wipemethod';

const WipeMethod: React.FC = () => {
  const [methods, setMethods] = useState<any[]>([]);
  const [method, setMethod] = useState('');
  const [passes, setPasses] = useState(1);
  const [pattern, setPattern] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchWipeMethods().then((data) => {
      if (data && data.length > 0) {
        setMethods(data);
        setMethod(data[0]?.value || '');
        setPattern(data[0]?.default_pattern || '');
      }
      setLoading(false);
    });
  }, []);

  const handleNext = () => {
    window.dispatchEvent(new CustomEvent('navigate', { detail: 'wipe-progress' }));
  };

  if (loading) return <Typography>Loading...</Typography>;
  if (!methods.length) return <Typography color="error">No wipe methods available.</Typography>;

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
          {methods.map((m) => (
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
      <TextField
        label="Number of Passes"
        type="number"
        inputProps={{ min: 1, max: 35 }}
        value={passes}
        onChange={e => setPasses(Number(e.target.value))}
        sx={{ mb: 3 }}
      />
      <TextField
        label="Pattern"
        select
        value={pattern}
        onChange={e => setPattern(e.target.value)}
        sx={{ mb: 3, ml: 2 }}
      >
        {(methods.find(m => m.value === method)?.patterns || []).map((p: any) => (
          <MenuItem key={p.value} value={p.value}>{p.label}</MenuItem>
        ))}
      </TextField>
      <Box mt={3} textAlign="right">
        <Button variant="contained" color="primary" onClick={handleNext}>Next</Button>
      </Box>
    </Box>
  );
};

export default WipeMethod;

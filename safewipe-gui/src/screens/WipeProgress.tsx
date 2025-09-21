import React, { useState, useEffect } from 'react';
import { Box, Typography, LinearProgress, Button, Paper, Chip, List, ListItem, ListItemText, Stack } from '@mui/material';
import PauseIcon from '@mui/icons-material/Pause';
import CancelIcon from '@mui/icons-material/Cancel';

const mockProgress = {
  percent: 60,
  currentPass: 2,
  totalPasses: 3,
  speed: 120,
  status: 'Overwriting block 1023...'
};

const mockLog = [
  'Started wipe operation',
  'Pass 1 complete',
  'Pass 2 in progress',
  'Overwriting block 1023...'
];

const WipeProgress: React.FC = () => {
  const [progress] = useState(mockProgress);
  const [log] = useState(mockLog);
  const [completed, setCompleted] = useState(false);

  useEffect(() => {
    if (progress.percent >= 100) {
      setCompleted(true);
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent('navigate', { detail: 'verification' }));
      }, 1000);
    }
  }, [progress.percent]);

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Wipe In Progress</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} alignItems="center" justifyContent="space-between">
          <Box flex={1} minWidth={240}>
            <Typography variant="subtitle1">{progress.status}</Typography>
            <LinearProgress variant="determinate" value={progress.percent} sx={{ height: 12, borderRadius: 6, mt: 2 }} />
            <Box display="flex" alignItems="center" gap={2} mt={2}>
              <Chip label={`Pass: ${progress.currentPass} of ${progress.totalPasses}`} color="primary" />
              <Chip label={`Speed: ${progress.speed} MB/s`} color="secondary" />
              <Chip label={`Progress: ${progress.percent}%`} color="success" />
            </Box>
          </Box>
          <Box flexShrink={0} minWidth={180} textAlign="right">
            <Button variant="outlined" color="warning" startIcon={<PauseIcon />} sx={{ mr: 1 }}>Pause</Button>
            <Button variant="contained" color="error" startIcon={<CancelIcon />}>Cancel</Button>
          </Box>
        </Stack>
      </Paper>
      <Typography variant="h6" gutterBottom>Log</Typography>
      <Paper elevation={1} sx={{ maxHeight: 200, overflow: 'auto', p: 2 }}>
        <List dense>
          {log.map((entry, idx) => (
            <ListItem key={idx}>
              <ListItemText primary={entry} />
            </ListItem>
          ))}
        </List>
      </Paper>
      {completed && <Typography color="success.main" mt={2}>Wipe complete! Redirecting to verification...</Typography>}
    </Box>
  );
};

export default WipeProgress;

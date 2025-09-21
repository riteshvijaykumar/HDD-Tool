import React, { useEffect, useState } from 'react';
import { Box, Typography, LinearProgress, Button, Paper, Chip, List, ListItem, ListItemText, Stack } from '@mui/material';
import PauseIcon from '@mui/icons-material/Pause';
import CancelIcon from '@mui/icons-material/Cancel';

// Replace with your actual API call
async function fetchWipeProgress() {
  return await window.api.get_wipe_progress();
}
async function fetchWipeLog() {
  return await window.api.get_wipe_log();
}

const WipeProgress: React.FC = () => {
  const [progress, setProgress] = useState<any>(null);
  const [log, setLog] = useState<string[]>([]);
  const [completed, setCompleted] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let interval: any;
    fetchWipeProgress().then((resp) => {
      setProgress(resp.data);
      setLoading(false);
      if (resp.data.percent >= 100) {
        setCompleted(true);
        setTimeout(() => {
          window.dispatchEvent(new CustomEvent('navigate', { detail: 'verification' }));
        }, 1000);
      }
    });
    fetchWipeLog().then((resp) => setLog(resp.data));
    interval = setInterval(() => {
      fetchWipeProgress().then((resp) => {
        setProgress(resp.data);
        if (resp.data.percent >= 100) {
          setCompleted(true);
          clearInterval(interval);
          setTimeout(() => {
            window.dispatchEvent(new CustomEvent('navigate', { detail: 'verification' }));
          }, 1000);
        }
      });
      fetchWipeLog().then((resp) => setLog(resp.data));
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  if (loading) return <Typography>Loading...</Typography>;
  if (!progress) return <Typography color="error">Failed to load progress.</Typography>;

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
            <Button variant="outlined" color="error" startIcon={<CancelIcon />}>Cancel</Button>
          </Box>
        </Stack>
        <List sx={{ mt: 2, maxHeight: 180, overflow: 'auto' }}>
          {log.map((entry, idx) => (
            <ListItem key={idx}><ListItemText primary={entry} /></ListItem>
          ))}
        </List>
      </Paper>
    </Box>
  );
};

export default WipeProgress;

import React from 'react';
import { Box, Typography, Paper, Button, Chip, Divider, Stack } from '@mui/material';
import DownloadIcon from '@mui/icons-material/Download';
import PrintIcon from '@mui/icons-material/Print';
import ShareIcon from '@mui/icons-material/Share';

const mockReport = {
  device: 'Samsung SSD 970 EVO',
  path: '/dev/sda',
  size: '512GB',
  method: 'Clear (Overwrite)',
  status: 'Success',
  date: '2025-09-21 14:32',
  certificateId: 'SW-20250921-001',
};

const Report: React.FC = () => {
  const handleDownload = (type: 'json' | 'pdf') => {
    // TODO: Implement download logic
    alert(`Download ${type.toUpperCase()} not implemented in mockup.`);
  };

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Wipe Report / Certificate</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="flex-start" justifyContent="space-between">
          <Box flex={1} minWidth={220}>
            <Typography variant="subtitle1"><b>Device:</b> {mockReport.device}</Typography>
            <Typography variant="body2" color="text.secondary">{mockReport.path} • {mockReport.size}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Method:</b> {mockReport.method}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Date:</b> {mockReport.date}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Certificate ID:</b> {mockReport.certificateId}</Typography>
          </Box>
          <Box flexShrink={0} minWidth={180} textAlign="right">
            <Chip label={mockReport.status} color={mockReport.status === 'Success' ? 'success' : 'error'} size="medium" sx={{ fontSize: 18, p: 2 }} />
          </Box>
        </Stack>
        <Divider sx={{ my: 2 }} />
        <Box display="flex" gap={2} justifyContent="flex-end">
          <Button variant="outlined" startIcon={<DownloadIcon />} onClick={() => handleDownload('json')}>Download JSON</Button>
          <Button variant="outlined" startIcon={<DownloadIcon />} onClick={() => handleDownload('pdf')}>Download PDF</Button>
          <Button variant="outlined" startIcon={<PrintIcon />}>Print</Button>
          <Button variant="outlined" startIcon={<ShareIcon />}>Share</Button>
        </Box>
      </Paper>
      <Box mt={4} textAlign="right">
        <Button variant="contained" color="primary" size="large" onClick={() => window.dispatchEvent(new CustomEvent('navigate', { detail: 'dashboard' }))}>
          Finish
        </Button>
      </Box>
    </Box>
  );
};

export default Report;

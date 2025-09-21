import React, { useEffect, useState } from 'react';
import { Box, Typography, Paper, Button, Chip, Divider, Stack } from '@mui/material';
import DownloadIcon from '@mui/icons-material/Download';
import PrintIcon from '@mui/icons-material/Print';
import ShareIcon from '@mui/icons-material/Share';
import { fetchReport } from '../api/report';

const Report: React.FC = () => {
  const [report, setReport] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchReport().then((data) => {
      setReport(data);
      setLoading(false);
    });
  }, []);

  const handleDownload = (type: 'json' | 'pdf') => {
    // TODO: Implement download logic
    alert(`Download ${type.toUpperCase()} not implemented.`);
  };

  if (loading) return <Typography>Loading...</Typography>;
  if (!report) return <Typography color="error">Failed to load report.</Typography>;

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Wipe Report / Certificate</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="flex-start" justifyContent="space-between">
          <Box flex={1} minWidth={220}>
            <Typography variant="subtitle1"><b>Device:</b> {report.device_name}</Typography>
            <Typography variant="body2" color="text.secondary">{report.device_path} • {report.device_size}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Method:</b> {report.method}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Date:</b> {report.date}</Typography>
            <Typography variant="subtitle1" mt={2}><b>Certificate ID:</b> {report.certificate_id}</Typography>
          </Box>
          <Box flexShrink={0} minWidth={180} textAlign="right">
            <Chip label={report.status} color={report.status === 'Success' ? 'success' : 'error'} size="medium" sx={{ fontSize: 18, p: 2 }} />
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

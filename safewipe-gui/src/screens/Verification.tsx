import React, { useEffect, useState } from 'react';
import { Box, Typography, Paper, Button, Chip, Stack } from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';

// Replace with your actual API call
async function fetchVerification() {
  return await window.api.get_verification();
}

const Verification: React.FC = () => {
  const [verification, setVerification] = useState<any>(null);
  const [verifying, setVerifying] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchVerification().then((resp) => {
      setVerification(resp.data);
      setLoading(false);
    });
  }, []);

  const handleRerun = () => {
    setVerifying(true);
    fetchVerification().then((resp) => {
      setVerification(resp.data);
      setVerifying(false);
    });
  };

  const handleNext = () => {
    window.dispatchEvent(new CustomEvent('navigate', { detail: 'report' }));
  };

  if (loading) return <Typography>Loading...</Typography>;
  if (!verification) return <Typography color="error">Failed to load verification.</Typography>;

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>Verification</Typography>
      <Paper elevation={3} sx={{ p: 3, mb: 3 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="center" justifyContent="space-between">
          <Box flex={1} minWidth={220}>
            <Box display="flex" alignItems="center" gap={1}>
              {verification.randomBlockCheck ? <CheckCircleIcon color="success" /> : <ErrorIcon color="error" />}
              <Typography variant="subtitle1">Random block check: {verification.randomBlockCheck ? 'passed' : 'failed'}</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={1} mt={2}>
              {verification.cryptoKeyInvalidation ? <CheckCircleIcon color="success" /> : <ErrorIcon color="error" />}
              <Typography variant="subtitle1">Crypto key invalidation: {verification.cryptoKeyInvalidation ? 'confirmed' : 'not confirmed'}</Typography>
            </Box>
          </Box>
          <Box flexShrink={0} minWidth={180} textAlign="right">
            <Chip label={verification.status === 'passed' ? 'Verification Passed' : 'Verification Failed'} color={verification.status === 'passed' ? 'success' : 'error'} size="medium" />
          </Box>
        </Stack>
      </Paper>
      <Box mt={4} textAlign="right">
        <Button variant="outlined" color="primary" onClick={handleRerun} disabled={verifying} sx={{ mr: 2 }}>
          {verifying ? 'Verifying...' : 'Re-run Verification'}
        </Button>
        <Button variant="contained" color="primary" onClick={handleNext}>
          Next
        </Button>
      </Box>
    </Box>
  );
};

export default Verification;

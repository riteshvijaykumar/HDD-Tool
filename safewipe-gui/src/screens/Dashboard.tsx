import React from 'react';
import { Box, Typography, Button, Card, CardContent, List, ListItem, ListItemText, Chip, Stack } from '@mui/material';
import StorageIcon from '@mui/icons-material/Storage';
import PlayCircleFilledWhiteIcon from '@mui/icons-material/PlayCircleFilledWhite';

const mockDrives = [
  { name: 'Samsung SSD 970 EVO', status: 'Not sanitized', type: 'SSD', size: '512GB' },
  { name: 'Seagate Barracuda', status: 'Sanitized', type: 'HDD', size: '1TB' },
  { name: 'Kingston USB', status: 'In progress', type: 'USB', size: '64GB' },
];

const mockActivity = [
  { device: 'Seagate Barracuda', action: 'Wiped', date: '2025-09-20' },
  { device: 'Kingston USB', action: 'Started', date: '2025-09-21' },
];

const statusColor = (status: string) => {
  switch (status) {
    case 'Sanitized': return 'success';
    case 'In progress': return 'warning';
    default: return 'error';
  }
};

const Dashboard: React.FC = () => {
  // TODO: Replace with navigation prop or context
  const handleStartWipe = () => {
    // Navigate to device selection
    window.dispatchEvent(new CustomEvent('navigate', { detail: 'device-selection' }));
  };

  return (
    <Box sx={{ p: 4 }}>
      <Typography variant="h4" gutterBottom>SafeWipe Dashboard</Typography>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={3} alignItems="flex-start">
        <Box flex={2} minWidth={300}>
          <Typography variant="h6" gutterBottom>System Drives</Typography>
          <Stack direction="row" spacing={2} flexWrap="wrap" useFlexGap>
            {mockDrives.map((drive, idx) => (
              <Card elevation={3} key={idx} sx={{ minWidth: 220, flex: '1 1 220px' }}>
                <CardContent>
                  <Box display="flex" alignItems="center" gap={1}>
                    <StorageIcon color="primary" />
                    <Typography variant="subtitle1">{drive.name}</Typography>
                  </Box>
                  <Typography variant="body2" color="text.secondary">{drive.type} • {drive.size}</Typography>
                  <Chip label={drive.status} color={statusColor(drive.status)} size="small" sx={{ mt: 1 }} />
                </CardContent>
              </Card>
            ))}
          </Stack>
          <Button variant="contained" color="primary" startIcon={<PlayCircleFilledWhiteIcon />} sx={{ mt: 3 }} onClick={handleStartWipe}>
            Start New Wipe
          </Button>
        </Box>
        <Box flex={1} minWidth={220}>
          <Typography variant="h6" gutterBottom>Recent Activity</Typography>
          <List dense>
            {mockActivity.map((activity, idx) => (
              <ListItem key={idx}>
                <ListItemText
                  primary={`${activity.device} - ${activity.action}`}
                  secondary={activity.date}
                />
              </ListItem>
            ))}
          </List>
        </Box>
      </Stack>
    </Box>
  );
};

export default Dashboard;

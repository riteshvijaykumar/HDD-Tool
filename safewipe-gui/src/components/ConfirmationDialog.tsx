import React from 'react';

interface ConfirmationDialogProps {
  open: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  deviceName: string;
}

const ConfirmationDialog: React.FC<ConfirmationDialogProps> = ({ open, onCancel, onConfirm, deviceName }) => {
  const [input, setInput] = React.useState('');
  return open ? (
    <div className="confirmation-dialog">
      <div className="dialog-content">
        <h2>Are you sure?</h2>
        <p className="warning">This operation is irreversible!</p>
        <p>Type <b>{deviceName}</b> to confirm:</p>
        <input value={input} onChange={e => setInput(e.target.value)} />
        <div className="dialog-actions">
          <button onClick={onCancel}>Cancel</button>
          <button onClick={onConfirm} disabled={input !== deviceName}>Confirm</button>
        </div>
      </div>
    </div>
  ) : null;
};

export default ConfirmationDialog;


import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { DlqRecord } from '../../types';

interface DlqState {
  items: DlqRecord[];
  isLoading: boolean;
  isRetryingAll: boolean;
  successMessage: string | null;
  error: string | null;
}

const initialState: DlqState = {
  items: [],
  isLoading: false,
  isRetryingAll: false,
  successMessage: null,
  error: null,
};

const dlqSlice = createSlice({
  name: 'dlq',
  initialState,
  reducers: {
    fetchDlqRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchDlqSuccess: (state, action: PayloadAction<DlqRecord[]>) => {
      state.isLoading = false;
      state.items = action.payload;
    },
    fetchDlqFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    retryAllDlqRequest: (state) => {
      state.isRetryingAll = true;
      state.successMessage = null;
      state.error = null;
    },
    retryAllDlqSuccess: (state, action: PayloadAction<number>) => {
      state.isRetryingAll = false;
      state.successMessage = `Successfully requeued ${action.payload} deliveries for retry.`;
    },
    retryAllDlqFailure: (state, action: PayloadAction<string>) => {
      state.isRetryingAll = false;
      state.error = action.payload;
    },
    discardDlqItemRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
    discardDlqItemSuccess: (state) => {
      state.isLoading = false;
    },
    clearDlqMessage: (state) => {
      state.successMessage = null;
      state.error = null;
    },
  },
});

export const {
  fetchDlqRequest,
  fetchDlqSuccess,
  fetchDlqFailure,
  retryAllDlqRequest,
  retryAllDlqSuccess,
  retryAllDlqFailure,
  discardDlqItemRequest,
  discardDlqItemSuccess,
  clearDlqMessage,
} = dlqSlice.actions;

export default dlqSlice.reducer;

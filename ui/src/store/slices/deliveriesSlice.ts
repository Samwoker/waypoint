import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Delivery, DeliveryAttempt } from '../../types';

interface DeliveriesState {
  deliveries: Delivery[];
  selectedDelivery: Delivery | null;
  attempts: DeliveryAttempt[];
  statusFilter: string;
  isLoading: boolean;
  isReplaying: boolean;
  replaySuccess: boolean;
  error: string | null;
}

const initialState: DeliveriesState = {
  deliveries: [],
  selectedDelivery: null,
  attempts: [],
  statusFilter: 'all',
  isLoading: false,
  isReplaying: false,
  replaySuccess: false,
  error: null,
};

const deliveriesSlice = createSlice({
  name: 'deliveries',
  initialState,
  reducers: {
    fetchDeliveriesRequest: (state, _action: PayloadAction<{ status?: string } | undefined>) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchDeliveriesSuccess: (state, action: PayloadAction<Delivery[]>) => {
      state.isLoading = false;
      state.deliveries = action.payload;
    },
    fetchDeliveriesFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    selectDeliveryRequest: (state, action: PayloadAction<Delivery>) => {
      state.selectedDelivery = action.payload;
      state.replaySuccess = false;
    },
    fetchAttemptsSuccess: (state, action: PayloadAction<DeliveryAttempt[]>) => {
      state.attempts = action.payload;
    },
    setStatusFilter: (state, action: PayloadAction<string>) => {
      state.statusFilter = action.payload;
    },
    replayDeliveryRequest: (state, _action: PayloadAction<string>) => {
      state.isReplaying = true;
      state.replaySuccess = false;
    },
    replayDeliverySuccess: (state) => {
      state.isReplaying = false;
      state.replaySuccess = true;
    },
    replayDeliveryFailure: (state, action: PayloadAction<string>) => {
      state.isReplaying = false;
      state.error = action.payload;
    },
    clearSelectedDelivery: (state) => {
      state.selectedDelivery = null;
      state.attempts = [];
    },
  },
});

export const {
  fetchDeliveriesRequest,
  fetchDeliveriesSuccess,
  fetchDeliveriesFailure,
  selectDeliveryRequest,
  fetchAttemptsSuccess,
  setStatusFilter,
  replayDeliveryRequest,
  replayDeliverySuccess,
  replayDeliveryFailure,
  clearSelectedDelivery,
} = deliveriesSlice.actions;

export default deliveriesSlice.reducer;

import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Destination } from '../../types';

interface DestinationsState {
  destinations: Destination[];
  isLoading: boolean;
  error: string | null;
}

const initialState: DestinationsState = {
  destinations: [],
  isLoading: false,
  error: null,
};

const destinationsSlice = createSlice({
  name: 'destinations',
  initialState,
  reducers: {
    fetchDestinationsRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchDestinationsSuccess: (state, action: PayloadAction<Destination[]>) => {
      state.isLoading = false;
      state.destinations = action.payload;
    },
    fetchDestinationsFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    createDestinationRequest: (
      state,
      _action: PayloadAction<{
        name: string;
        url: string;
        description?: string;
        timeout_ms?: number;
        max_retries?: number;
        rate_limit_rps?: number;
      }>
    ) => {
      state.isLoading = true;
    },
    createDestinationSuccess: (state, action: PayloadAction<Destination>) => {
      state.isLoading = false;
      state.destinations.unshift(action.payload);
    },
    resetCircuitRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
    deleteDestinationRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
  },
});

export const {
  fetchDestinationsRequest,
  fetchDestinationsSuccess,
  fetchDestinationsFailure,
  createDestinationRequest,
  createDestinationSuccess,
  resetCircuitRequest,
  deleteDestinationRequest,
} = destinationsSlice.actions;

export default destinationsSlice.reducer;

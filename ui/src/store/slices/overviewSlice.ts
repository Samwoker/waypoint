import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { OverviewStats, SystemStats, TimeseriesPoint } from '../../types';

interface OverviewState {
  overviewStats: OverviewStats | null;
  systemStats: SystemStats | null;
  timeseries: TimeseriesPoint[];
  period: '1h' | '24h' | '7d' | '30d';
  isLoading: boolean;
  error: string | null;
}

const initialState: OverviewState = {
  overviewStats: null,
  systemStats: null,
  timeseries: [],
  period: '24h',
  isLoading: false,
  error: null,
};

const overviewSlice = createSlice({
  name: 'overview',
  initialState,
  reducers: {
    fetchOverviewRequest: (state, _action: PayloadAction<{ period?: string } | undefined>) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchOverviewSuccess: (
      state,
      action: PayloadAction<{
        overviewStats: OverviewStats;
        systemStats: SystemStats;
        timeseries: TimeseriesPoint[];
      }>
    ) => {
      state.isLoading = false;
      state.overviewStats = action.payload.overviewStats;
      state.systemStats = action.payload.systemStats;
      state.timeseries = action.payload.timeseries;
    },
    fetchOverviewFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    setPeriod: (state, action: PayloadAction<'1h' | '24h' | '7d' | '30d'>) => {
      state.period = action.payload;
    },
  },
});

export const {
  fetchOverviewRequest,
  fetchOverviewSuccess,
  fetchOverviewFailure,
  setPeriod,
} = overviewSlice.actions;

export default overviewSlice.reducer;

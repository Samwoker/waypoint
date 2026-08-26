import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Source, VerificationLog } from '../../types';

interface SourcesState {
  sources: Source[];
  selectedSource: Source | null;
  verificationLogs: VerificationLog[];
  generatedSecret: string | null;
  isLoading: boolean;
  error: string | null;
}

const initialState: SourcesState = {
  sources: [],
  selectedSource: null,
  verificationLogs: [],
  generatedSecret: null,
  isLoading: false,
  error: null,
};

const sourcesSlice = createSlice({
  name: 'sources',
  initialState,
  reducers: {
    fetchSourcesRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchSourcesSuccess: (state, action: PayloadAction<Source[]>) => {
      state.isLoading = false;
      state.sources = action.payload;
    },
    fetchSourcesFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    createSourceRequest: (
      state,
      _action: PayloadAction<{
        name: string;
        slug: string;
        provider: string;
        verification_type: string;
        secret?: string;
      }>
    ) => {
      state.isLoading = true;
    },
    createSourceSuccess: (state, action: PayloadAction<Source>) => {
      state.isLoading = false;
      state.sources.unshift(action.payload);
    },
    rotateSecretRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
    rotateSecretSuccess: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.generatedSecret = action.payload;
    },
    fetchVerificationLogsRequest: (state, action: PayloadAction<Source>) => {
      state.selectedSource = action.payload;
      state.isLoading = true;
    },
    fetchVerificationLogsSuccess: (state, action: PayloadAction<VerificationLog[]>) => {
      state.isLoading = false;
      state.verificationLogs = action.payload;
    },
    clearGeneratedSecret: (state) => {
      state.generatedSecret = null;
    },
    clearSelectedSource: (state) => {
      state.selectedSource = null;
      state.verificationLogs = [];
    },
  },
});

export const {
  fetchSourcesRequest,
  fetchSourcesSuccess,
  fetchSourcesFailure,
  createSourceRequest,
  createSourceSuccess,
  rotateSecretRequest,
  rotateSecretSuccess,
  fetchVerificationLogsRequest,
  fetchVerificationLogsSuccess,
  clearGeneratedSecret,
  clearSelectedSource,
} = sourcesSlice.actions;

export default sourcesSlice.reducer;

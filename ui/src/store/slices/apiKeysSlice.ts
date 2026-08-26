import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { ApiKey, ApiKeyCreated, TenantUsage } from '../../types';

interface ApiKeysState {
  apiKeys: ApiKey[];
  newlyCreatedKey: ApiKeyCreated | null;
  tenantUsage: TenantUsage | null;
  isLoading: boolean;
  error: string | null;
}

const initialState: ApiKeysState = {
  apiKeys: [],
  newlyCreatedKey: null,
  tenantUsage: null,
  isLoading: false,
  error: null,
};

const apiKeysSlice = createSlice({
  name: 'apiKeys',
  initialState,
  reducers: {
    fetchApiKeysRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchApiKeysSuccess: (state, action: PayloadAction<ApiKey[]>) => {
      state.isLoading = false;
      state.apiKeys = action.payload;
    },
    fetchApiKeysFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    createApiKeyRequest: (
      state,
      _action: PayloadAction<{ name: string; expiresInDays?: number }>
    ) => {
      state.isLoading = true;
    },
    createApiKeySuccess: (state, action: PayloadAction<ApiKeyCreated>) => {
      state.isLoading = false;
      state.newlyCreatedKey = action.payload;
    },
    revokeApiKeyRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
    fetchTenantUsageRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
    fetchTenantUsageSuccess: (state, action: PayloadAction<TenantUsage>) => {
      state.isLoading = false;
      state.tenantUsage = action.payload;
    },
    clearNewlyCreatedKey: (state) => {
      state.newlyCreatedKey = null;
    },
  },
});

export const {
  fetchApiKeysRequest,
  fetchApiKeysSuccess,
  fetchApiKeysFailure,
  createApiKeyRequest,
  createApiKeySuccess,
  revokeApiKeyRequest,
  fetchTenantUsageRequest,
  fetchTenantUsageSuccess,
  clearNewlyCreatedKey,
} = apiKeysSlice.actions;

export default apiKeysSlice.reducer;

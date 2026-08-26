import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Tenant, User } from '../../types';

interface AuthState {
  user: User | null;
  currentTenant: Tenant | null;
  tenants: Tenant[];
  token: string | null;
  isLoading: boolean;
  error: string | null;
}

const initialState: AuthState = {
  user: null,
  currentTenant: null,
  tenants: [],
  token: localStorage.getItem('waypoint_token'),
  isLoading: false,
  error: null,
};

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    registerRequest: (
      state,
      _action: PayloadAction<{ email: string; pass: string; tenantName: string }>
    ) => {
      state.isLoading = true;
      state.error = null;
    },
    loginRequest: (state, _action: PayloadAction<{ email: string; pass: string }>) => {
      state.isLoading = true;
      state.error = null;
    },
    loginSuccess: (state, action: PayloadAction<{ user: User; token: string; tenants: Tenant[] }>) => {
      state.isLoading = false;
      state.user = action.payload.user;
      state.token = action.payload.token;
      state.tenants = action.payload.tenants;
      if (action.payload.tenants.length > 0 && !state.currentTenant) {
        state.currentTenant = action.payload.tenants[0];
      }
      localStorage.setItem('waypoint_token', action.payload.token);
    },
    loginFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    clearAuthError: (state) => {
      state.error = null;
    },
    checkAuthRequest: (state) => {
      state.isLoading = true;
    },
    checkAuthSuccess: (state, action: PayloadAction<{ user: User; tenants: Tenant[] }>) => {
      state.isLoading = false;
      state.user = action.payload.user;
      state.tenants = action.payload.tenants;
      if (action.payload.tenants.length > 0 && !state.currentTenant) {
        state.currentTenant = action.payload.tenants[0];
      }
    },
    checkAuthFailure: (state) => {
      state.isLoading = false;
    },
    switchTenant: (state, action: PayloadAction<Tenant>) => {
      state.currentTenant = action.payload;
    },
    fetchTenantsSuccess: (state, action: PayloadAction<Tenant[]>) => {
      state.tenants = action.payload;
      if (action.payload.length > 0 && !state.currentTenant) {
        state.currentTenant = action.payload[0];
      }
    },
    logout: (state) => {
      state.user = null;
      state.currentTenant = null;
      state.token = null;
      localStorage.removeItem('waypoint_token');
      localStorage.removeItem('waypoint_api_key');
    },
  },
});

export const {
  registerRequest,
  loginRequest,
  loginSuccess,
  loginFailure,
  clearAuthError,
  checkAuthRequest,
  checkAuthSuccess,
  checkAuthFailure,
  switchTenant,
  fetchTenantsSuccess,
  logout,
} = authSlice.actions;

export default authSlice.reducer;

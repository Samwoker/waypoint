import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Subscription } from '../../types';

interface SubscriptionsState {
  subscriptions: Subscription[];
  isLoading: boolean;
  error: string | null;
}

const initialState: SubscriptionsState = {
  subscriptions: [],
  isLoading: false,
  error: null,
};

const subscriptionsSlice = createSlice({
  name: 'subscriptions',
  initialState,
  reducers: {
    fetchSubscriptionsRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchSubscriptionsSuccess: (state, action: PayloadAction<Subscription[]>) => {
      state.isLoading = false;
      state.subscriptions = action.payload;
    },
    fetchSubscriptionsFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    createSubscriptionRequest: (
      state,
      _action: PayloadAction<{
        source_id: string;
        destination_id: string;
        event_types: string[];
        filter_expression?: string;
      }>
    ) => {
      state.isLoading = true;
    },
    createSubscriptionSuccess: (state, action: PayloadAction<Subscription>) => {
      state.isLoading = false;
      state.subscriptions.unshift(action.payload);
    },
    toggleSubscriptionRequest: (state, _action: PayloadAction<{ id: string; is_active: boolean }>) => {
      state.isLoading = true;
    },
    deleteSubscriptionRequest: (state, _action: PayloadAction<string>) => {
      state.isLoading = true;
    },
  },
});

export const {
  fetchSubscriptionsRequest,
  fetchSubscriptionsSuccess,
  fetchSubscriptionsFailure,
  createSubscriptionRequest,
  createSubscriptionSuccess,
  toggleSubscriptionRequest,
  deleteSubscriptionRequest,
} = subscriptionsSlice.actions;

export default subscriptionsSlice.reducer;

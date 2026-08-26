import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { EventItem } from '../../types';

interface EventsState {
  events: EventItem[];
  selectedEvent: EventItem | null;
  isLoading: boolean;
  isSending: boolean;
  sendResult: any | null;
  error: string | null;
}

const initialState: EventsState = {
  events: [],
  selectedEvent: null,
  isLoading: false,
  isSending: false,
  sendResult: null,
  error: null,
};

const eventsSlice = createSlice({
  name: 'events',
  initialState,
  reducers: {
    fetchEventsRequest: (state) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchEventsSuccess: (state, action: PayloadAction<EventItem[]>) => {
      state.isLoading = false;
      state.events = action.payload;
      if (action.payload.length > 0 && !state.selectedEvent) {
        state.selectedEvent = action.payload[0];
      }
    },
    fetchEventsFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    selectEvent: (state, action: PayloadAction<EventItem>) => {
      state.selectedEvent = action.payload;
    },
    sendWebhookRequest: (
      state,
      _action: PayloadAction<{ slug: string; payload: any; headers?: Record<string, string> }>
    ) => {
      state.isSending = true;
      state.sendResult = null;
      state.error = null;
    },
    sendWebhookSuccess: (state, action: PayloadAction<any>) => {
      state.isSending = false;
      state.sendResult = action.payload;
    },
    sendWebhookFailure: (state, action: PayloadAction<string>) => {
      state.isSending = false;
      state.error = action.payload;
    },
    clearSendResult: (state) => {
      state.sendResult = null;
      state.error = null;
    },
  },
});

export const {
  fetchEventsRequest,
  fetchEventsSuccess,
  fetchEventsFailure,
  selectEvent,
  sendWebhookRequest,
  sendWebhookSuccess,
  sendWebhookFailure,
  clearSendResult,
} = eventsSlice.actions;

export default eventsSlice.reducer;

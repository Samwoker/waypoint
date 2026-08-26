import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { EventItem } from '../../types';
import {
  fetchEventsFailure,
  fetchEventsRequest,
  fetchEventsSuccess,
  sendWebhookFailure,
  sendWebhookRequest,
  sendWebhookSuccess,
} from '../slices/eventsSlice';

function* handleFetchEvents(): Generator<any, void, any> {
  try {
    const events: EventItem[] = yield call([api, api.listEvents], 50);
    yield put(fetchEventsSuccess(events));
  } catch (error: any) {
    yield put(fetchEventsFailure(error.message || 'Failed to fetch events'));
  }
}

function* handleSendWebhook(
  action: PayloadAction<{ slug: string; payload: any; headers?: Record<string, string> }>
): Generator<any, void, any> {
  try {
    const res: any = yield call(
      [api, api.sendWebhook],
      action.payload.slug,
      action.payload.payload,
      action.payload.headers
    );
    yield put(sendWebhookSuccess(res));
    yield put(fetchEventsRequest());
  } catch (error: any) {
    yield put(sendWebhookFailure(error.message || 'Webhook dispatch failed'));
  }
}

export function* eventsSaga() {
  yield takeLatest(fetchEventsRequest.type, handleFetchEvents);
  yield takeLatest(sendWebhookRequest.type, handleSendWebhook);
}

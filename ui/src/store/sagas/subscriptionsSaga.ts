import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Subscription } from '../../types';
import {
  createSubscriptionRequest,
  createSubscriptionSuccess,
  deleteSubscriptionRequest,
  fetchSubscriptionsFailure,
  fetchSubscriptionsRequest,
  fetchSubscriptionsSuccess,
  toggleSubscriptionRequest,
} from '../slices/subscriptionsSlice';

function* handleFetchSubscriptions(): Generator<any, void, any> {
  try {
    const subscriptions: Subscription[] = yield call([api, api.listSubscriptions]);
    yield put(fetchSubscriptionsSuccess(subscriptions));
  } catch (error: any) {
    yield put(fetchSubscriptionsFailure(error.message || 'Failed to fetch subscriptions'));
  }
}

function* handleCreateSubscription(
  action: PayloadAction<{
    source_id: string;
    destination_id: string;
    event_types: string[];
    filter_expression?: string;
  }>
): Generator<any, void, any> {
  try {
    const created: Subscription = yield call([api, api.createSubscription], action.payload);
    yield put(createSubscriptionSuccess(created));
  } catch (error: any) {
    yield put(fetchSubscriptionsFailure(error.message || 'Failed to create subscription'));
  }
}

function* handleToggleSubscription(
  action: PayloadAction<{ id: string; is_active: boolean }>
): Generator<any, void, any> {
  try {
    yield call([api, api.updateSubscription], action.payload.id, { is_active: action.payload.is_active });
    yield put(fetchSubscriptionsRequest());
  } catch (error: any) {
    yield put(fetchSubscriptionsFailure(error.message || 'Failed to toggle subscription'));
  }
}

function* handleDeleteSubscription(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.deleteSubscription], action.payload);
    yield put(fetchSubscriptionsRequest());
  } catch (error: any) {
    yield put(fetchSubscriptionsFailure(error.message || 'Failed to delete subscription'));
  }
}

export function* subscriptionsSaga() {
  yield takeLatest(fetchSubscriptionsRequest.type, handleFetchSubscriptions);
  yield takeLatest(createSubscriptionRequest.type, handleCreateSubscription);
  yield takeLatest(toggleSubscriptionRequest.type, handleToggleSubscription);
  yield takeLatest(deleteSubscriptionRequest.type, handleDeleteSubscription);
}

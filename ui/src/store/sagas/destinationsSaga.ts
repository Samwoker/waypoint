import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Destination } from '../../types';
import {
  createDestinationRequest,
  createDestinationSuccess,
  deleteDestinationRequest,
  fetchDestinationsFailure,
  fetchDestinationsRequest,
  fetchDestinationsSuccess,
  resetCircuitRequest,
} from '../slices/destinationsSlice';

function* handleFetchDestinations(): Generator<any, void, any> {
  try {
    const destinations: Destination[] = yield call([api, api.listDestinations]);
    yield put(fetchDestinationsSuccess(destinations));
  } catch (error: any) {
    yield put(fetchDestinationsFailure(error.message || 'Failed to fetch destinations'));
  }
}

function* handleCreateDestination(
  action: PayloadAction<{
    name: string;
    url: string;
    timeout_ms?: number;
    max_retries?: number;
    rate_limit_rps?: number;
  }>
): Generator<any, void, any> {
  try {
    const created: Destination = yield call([api, api.createDestination], action.payload);
    yield put(createDestinationSuccess(created));
  } catch (error: any) {
    yield put(fetchDestinationsFailure(error.message || 'Failed to create destination'));
  }
}

function* handleResetCircuit(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.resumeDestination], action.payload);
    yield put(fetchDestinationsRequest());
  } catch (error: any) {
    yield put(fetchDestinationsFailure(error.message || 'Failed to reset circuit'));
  }
}

function* handleDeleteDestination(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.deleteDestination], action.payload);
    yield put(fetchDestinationsRequest());
  } catch (error: any) {
    yield put(fetchDestinationsFailure(error.message || 'Failed to delete destination'));
  }
}

export function* destinationsSaga() {
  yield takeLatest(fetchDestinationsRequest.type, handleFetchDestinations);
  yield takeLatest(createDestinationRequest.type, handleCreateDestination);
  yield takeLatest(resetCircuitRequest.type, handleResetCircuit);
  yield takeLatest(deleteDestinationRequest.type, handleDeleteDestination);
}

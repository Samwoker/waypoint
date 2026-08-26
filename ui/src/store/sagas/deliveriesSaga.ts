import { call, put, select, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Delivery, DeliveryAttempt } from '../../types';
import {
  fetchAttemptsSuccess,
  fetchDeliveriesFailure,
  fetchDeliveriesRequest,
  fetchDeliveriesSuccess,
  replayDeliveryFailure,
  replayDeliveryRequest,
  replayDeliverySuccess,
  selectDeliveryRequest,
  setStatusFilter,
} from '../slices/deliveriesSlice';

function* handleFetchDeliveries(): Generator<any, void, any> {
  try {
    const status: string = yield select((state: any) => state.deliveries.statusFilter);
    const deliveries: Delivery[] = yield call([api, api.listDeliveries], {
      status: status === 'all' ? undefined : status,
      limit: 50,
    });
    yield put(fetchDeliveriesSuccess(deliveries));
  } catch (error: any) {
    yield put(fetchDeliveriesFailure(error.message || 'Failed to fetch deliveries'));
  }
}

function* handleSelectDelivery(action: PayloadAction<Delivery>): Generator<any, void, any> {
  try {
    const attempts: DeliveryAttempt[] = yield call([api, api.listDeliveryAttempts], action.payload.id);
    yield put(fetchAttemptsSuccess(attempts));
  } catch (error: any) {
    yield put(fetchAttemptsSuccess([]));
  }
}

function* handleReplayDelivery(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.replayDelivery], action.payload);
    yield put(replayDeliverySuccess());
    yield put(fetchDeliveriesRequest());
    const selected: Delivery | null = yield select((state: any) => state.deliveries.selectedDelivery);
    if (selected && selected.id === action.payload) {
      yield put(selectDeliveryRequest(selected));
    }
  } catch (error: any) {
    yield put(replayDeliveryFailure(error.message || 'Replay failed'));
  }
}

export function* deliveriesSaga() {
  yield takeLatest(fetchDeliveriesRequest.type, handleFetchDeliveries);
  yield takeLatest(setStatusFilter.type, handleFetchDeliveries);
  yield takeLatest(selectDeliveryRequest.type, handleSelectDelivery);
  yield takeLatest(replayDeliveryRequest.type, handleReplayDelivery);
}

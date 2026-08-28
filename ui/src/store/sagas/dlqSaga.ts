import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { DlqRecord, PaginatedDlq } from '../../types';
import {
  discardDlqItemRequest,
  discardDlqItemSuccess,
  fetchDlqFailure,
  fetchDlqRequest,
  fetchDlqSuccess,
  retryAllDlqFailure,
  retryAllDlqRequest,
  retryAllDlqSuccess,
} from '../slices/dlqSlice';

function* handleFetchDlq(): Generator<any, void, any> {
  try {
    const res: PaginatedDlq = yield call([api, api.listDlq], 50);
    yield put(fetchDlqSuccess(res.items || []));
  } catch (error: any) {
    yield put(fetchDlqFailure(error.message || 'Failed to fetch DLQ items'));
  }
}

function* handleRetryAllDlq(): Generator<any, void, any> {
  try {
    const res: { replayed_count: number } = yield call([api, api.retryAllDlq]);
    yield put(retryAllDlqSuccess(res.replayed_count));
    yield put(fetchDlqRequest());
  } catch (error: any) {
    yield put(retryAllDlqFailure(error.message || 'Failed to retry all DLQ items'));
  }
}

function* handleDiscardDlqItem(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.discardDlqItem], action.payload);
    yield put(discardDlqItemSuccess());
    yield put(fetchDlqRequest());
  } catch (error: any) {
    yield put(fetchDlqFailure(error.message || 'Failed to discard item'));
  }
}

export function* dlqSaga() {
  yield takeLatest(fetchDlqRequest.type, handleFetchDlq);
  yield takeLatest(retryAllDlqRequest.type, handleRetryAllDlq);
  yield takeLatest(discardDlqItemRequest.type, handleDiscardDlqItem);
}

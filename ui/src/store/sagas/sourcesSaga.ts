import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Source, VerificationLog } from '../../types';
import {
  createSourceRequest,
  createSourceSuccess,
  fetchSourcesFailure,
  fetchSourcesRequest,
  fetchSourcesSuccess,
  fetchVerificationLogsRequest,
  fetchVerificationLogsSuccess,
  rotateSecretRequest,
  rotateSecretSuccess,
} from '../slices/sourcesSlice';

function* handleFetchSources(): Generator<any, void, any> {
  try {
    const sources: Source[] = yield call([api, api.listSources]);
    yield put(fetchSourcesSuccess(sources));
  } catch (error: any) {
    yield put(fetchSourcesFailure(error.message || 'Failed to fetch sources'));
  }
}

function* handleCreateSource(
  action: PayloadAction<{
    name: string;
    slug: string;
    provider: string;
    verification_type: string;
    secret?: string;
  }>
): Generator<any, void, any> {
  try {
    const created: Source = yield call([api, api.createSource], action.payload);
    yield put(createSourceSuccess(created));
  } catch (error: any) {
    yield put(fetchSourcesFailure(error.message || 'Failed to create source'));
  }
}

function* handleRotateSecret(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    const res: { secret: string } = yield call([api, api.rotateSourceSecret], action.payload);
    yield put(rotateSecretSuccess(res.secret));
    yield put(fetchSourcesRequest());
  } catch (error: any) {
    yield put(fetchSourcesFailure(error.message || 'Failed to rotate secret'));
  }
}

function* handleFetchLogs(action: PayloadAction<Source>): Generator<any, void, any> {
  try {
    const logs: VerificationLog[] = yield call([api, api.getSourceVerificationLog], action.payload.id, 20);
    yield put(fetchVerificationLogsSuccess(logs));
  } catch (error: any) {
    yield put(fetchVerificationLogsSuccess([]));
  }
}

export function* sourcesSaga() {
  yield takeLatest(fetchSourcesRequest.type, handleFetchSources);
  yield takeLatest(createSourceRequest.type, handleCreateSource);
  yield takeLatest(rotateSecretRequest.type, handleRotateSecret);
  yield takeLatest(fetchVerificationLogsRequest.type, handleFetchLogs);
}

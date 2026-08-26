import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { ApiKey, ApiKeyCreated, TenantUsage } from '../../types';
import {
  createApiKeyRequest,
  createApiKeySuccess,
  fetchApiKeysFailure,
  fetchApiKeysRequest,
  fetchApiKeysSuccess,
  fetchTenantUsageRequest,
  fetchTenantUsageSuccess,
  revokeApiKeyRequest,
} from '../slices/apiKeysSlice';

function* handleFetchApiKeys(): Generator<any, void, any> {
  try {
    const keys: ApiKey[] = yield call([api, api.listApiKeys]);
    yield put(fetchApiKeysSuccess(keys));
  } catch (error: any) {
    yield put(fetchApiKeysFailure(error.message || 'Failed to fetch API keys'));
  }
}

function* handleCreateApiKey(
  action: PayloadAction<{ name: string; expiresInDays?: number }>
): Generator<any, void, any> {
  try {
    const created: ApiKeyCreated = yield call(
      [api, api.createApiKey],
      action.payload.name,
      action.payload.expiresInDays
    );
    yield put(createApiKeySuccess(created));
    yield put(fetchApiKeysRequest());
  } catch (error: any) {
    yield put(fetchApiKeysFailure(error.message || 'Failed to create API key'));
  }
}

function* handleRevokeApiKey(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    yield call([api, api.revokeApiKey], action.payload);
    yield put(fetchApiKeysRequest());
  } catch (error: any) {
    yield put(fetchApiKeysFailure(error.message || 'Failed to revoke API key'));
  }
}

function* handleFetchUsage(action: PayloadAction<string>): Generator<any, void, any> {
  try {
    const usage: TenantUsage = yield call([api, api.getTenantUsage], action.payload);
    yield put(fetchTenantUsageSuccess(usage));
  } catch (error: any) {
    // optional usage
  }
}

export function* apiKeysSaga() {
  yield takeLatest(fetchApiKeysRequest.type, handleFetchApiKeys);
  yield takeLatest(createApiKeyRequest.type, handleCreateApiKey);
  yield takeLatest(revokeApiKeyRequest.type, handleRevokeApiKey);
  yield takeLatest(fetchTenantUsageRequest.type, handleFetchUsage);
}

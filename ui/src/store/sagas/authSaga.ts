import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Tenant, User } from '../../types';
import {
  checkAuthFailure,
  checkAuthRequest,
  checkAuthSuccess,
  loginFailure,
  loginRequest,
  loginSuccess,
  registerRequest,
} from '../slices/authSlice';

function* handleRegister(
  action: PayloadAction<{ email: string; pass: string; tenantName: string }>
): Generator<any, void, any> {
  try {
    const res: { access_token: string; refresh_token: string } = yield call(
      [api, api.register],
      action.payload.email,
      action.payload.pass,
      action.payload.tenantName
    );
    api.setToken(res.access_token);
    const user: User = yield call([api, api.getMe]);
    const tenants: Tenant[] = yield call([api, api.listTenants]);
    yield put(loginSuccess({ user, token: res.access_token, tenants }));
  } catch (error: any) {
    yield put(loginFailure(error.message || 'Registration failed'));
  }
}

function* handleLogin(action: PayloadAction<{ email: string; pass: string }>): Generator<any, void, any> {
  try {
    const res: { access_token: string; refresh_token: string } = yield call(
      [api, api.login],
      action.payload.email,
      action.payload.pass
    );
    api.setToken(res.access_token);
    const user: User = yield call([api, api.getMe]);
    const tenants: Tenant[] = yield call([api, api.listTenants]);
    yield put(loginSuccess({ user, token: res.access_token, tenants }));
  } catch (error: any) {
    yield put(loginFailure(error.message || 'Login failed'));
  }
}

function* handleCheckAuth(): Generator<any, void, any> {
  try {
    const token = api.getToken();
    if (!token) {
      yield put(checkAuthFailure());
      return;
    }
    const user: User = yield call([api, api.getMe]);
    const tenants: Tenant[] = yield call([api, api.listTenants]);
    yield put(checkAuthSuccess({ user, tenants }));
  } catch (error) {
    yield put(checkAuthFailure());
  }
}

export function* authSaga() {
  yield takeLatest(registerRequest.type, handleRegister);
  yield takeLatest(loginRequest.type, handleLogin);
  yield takeLatest(checkAuthRequest.type, handleCheckAuth);
}

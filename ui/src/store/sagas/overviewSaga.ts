import { all, call, put, select, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { OverviewStats, SystemStats, TimeseriesPoint } from '../../types';
import {
  fetchOverviewFailure,
  fetchOverviewRequest,
  fetchOverviewSuccess,
  setPeriod,
} from '../slices/overviewSlice';

function* handleFetchOverview(action?: PayloadAction<{ period?: string } | undefined>): Generator<any, void, any> {
  try {
    const currentPeriod: '1h' | '24h' | '7d' | '30d' = yield select((state: any) => state.overview.period);
    const period = action?.payload?.period || currentPeriod;

    const [overviewStats, systemStats, timeseries]: [OverviewStats, SystemStats, TimeseriesPoint[]] = yield all([
      call([api, api.getOverviewStats], period),
      call([api, api.getSystemStats]),
      call([api, api.getTimeseriesStats], period),
    ]);

    yield put(fetchOverviewSuccess({ overviewStats, systemStats, timeseries }));
  } catch (error: any) {
    yield put(fetchOverviewFailure(error.message || 'Failed to load telemetry'));
  }
}

export function* overviewSaga() {
  yield takeLatest(fetchOverviewRequest.type, handleFetchOverview);
  yield takeLatest(setPeriod.type, handleFetchOverview);
}

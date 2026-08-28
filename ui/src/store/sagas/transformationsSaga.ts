import { call, put, takeLatest } from 'redux-saga/effects';
import { PayloadAction } from '@reduxjs/toolkit';
import { api } from '../../api/client';
import { Transformation } from '../../types';
import {
  fetchTransformationsFailure,
  fetchTransformationsRequest,
  fetchTransformationsSuccess,
  testTransformationFailure,
  testTransformationRequest,
  testTransformationSuccess,
} from '../slices/transformationsSlice';

function* handleFetchTransformations(): Generator<any, void, any> {
  try {
    const transformations: Transformation[] = yield call([api, api.listTransformations]);
    yield put(fetchTransformationsSuccess(transformations));
  } catch (error: any) {
    yield put(fetchTransformationsFailure(error.message || 'Failed to fetch transformations'));
  }
}

function* handleTestTransformation(
  action: PayloadAction<{ template: string; payload: any }>
): Generator<any, void, any> {
  try {
    const res: { transformed_payload: any } = yield call(
      [api, api.testTransformation],
      action.payload.template,
      action.payload.payload
    );
    yield put(testTransformationSuccess(res.transformed_payload));
  } catch (error: any) {
    yield put(testTransformationFailure(error.message || 'Transformation failed'));
  }
}

export function* transformationsSaga() {
  yield takeLatest(fetchTransformationsRequest.type, handleFetchTransformations);
  yield takeLatest(testTransformationRequest.type, handleTestTransformation);
}

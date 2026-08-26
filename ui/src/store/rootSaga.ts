import { all, fork } from 'redux-saga/effects';
import { apiKeysSaga } from './sagas/apiKeysSaga';
import { authSaga } from './sagas/authSaga';
import { deliveriesSaga } from './sagas/deliveriesSaga';
import { destinationsSaga } from './sagas/destinationsSaga';
import { dlqSaga } from './sagas/dlqSaga';
import { eventsSaga } from './sagas/eventsSaga';
import { overviewSaga } from './sagas/overviewSaga';
import { sourcesSaga } from './sagas/sourcesSaga';
import { subscriptionsSaga } from './sagas/subscriptionsSaga';
import { transformationsSaga } from './sagas/transformationsSaga';

export function* rootSaga() {
  yield all([
    fork(authSaga),
    fork(overviewSaga),
    fork(eventsSaga),
    fork(deliveriesSaga),
    fork(dlqSaga),
    fork(sourcesSaga),
    fork(destinationsSaga),
    fork(subscriptionsSaga),
    fork(transformationsSaga),
    fork(apiKeysSaga),
  ]);
}

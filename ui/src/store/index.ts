import { configureStore } from '@reduxjs/toolkit';
import createSagaMiddleware from 'redux-saga';
import { rootSaga } from './rootSaga';
import apiKeysReducer from './slices/apiKeysSlice';
import authReducer from './slices/authSlice';
import deliveriesReducer from './slices/deliveriesSlice';
import destinationsReducer from './slices/destinationsSlice';
import dlqReducer from './slices/dlqSlice';
import eventsReducer from './slices/eventsSlice';
import overviewReducer from './slices/overviewSlice';
import sourcesReducer from './slices/sourcesSlice';
import subscriptionsReducer from './slices/subscriptionsSlice';
import transformationsReducer from './slices/transformationsSlice';

const sagaMiddleware = createSagaMiddleware();

export const store = configureStore({
  reducer: {
    auth: authReducer,
    overview: overviewReducer,
    events: eventsReducer,
    deliveries: deliveriesReducer,
    dlq: dlqReducer,
    sources: sourcesReducer,
    destinations: destinationsReducer,
    subscriptions: subscriptionsReducer,
    transformations: transformationsReducer,
    apiKeys: apiKeysReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({ thunk: false, serializableCheck: false }).concat(sagaMiddleware),
});

sagaMiddleware.run(rootSaga);

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Transformation } from '../../types';

interface TransformationsState {
  transformations: Transformation[];
  evaluatedOutput: any | null;
  isLoading: boolean;
  isEvaluating: boolean;
  error: string | null;
}

const initialState: TransformationsState = {
  transformations: [],
  evaluatedOutput: null,
  isLoading: false,
  isEvaluating: false,
  error: null,
};

const transformationsSlice = createSlice({
  name: 'transformations',
  initialState,
  reducers: {
    fetchTransformationsRequest: (state, _action: PayloadAction<string | undefined>) => {
      state.isLoading = true;
      state.error = null;
    },
    fetchTransformationsSuccess: (state, action: PayloadAction<Transformation[]>) => {
      state.isLoading = false;
      state.transformations = action.payload;
    },
    fetchTransformationsFailure: (state, action: PayloadAction<string>) => {
      state.isLoading = false;
      state.error = action.payload;
    },
    testTransformationRequest: (
      state,
      _action: PayloadAction<{ template: string; payload: any }>
    ) => {
      state.isEvaluating = true;
      state.error = null;
    },
    testTransformationSuccess: (state, action: PayloadAction<any>) => {
      state.isEvaluating = false;
      state.evaluatedOutput = action.payload;
    },
    testTransformationFailure: (state, action: PayloadAction<string>) => {
      state.isEvaluating = false;
      state.error = action.payload;
      state.evaluatedOutput = null;
    },
    clearEvaluatedOutput: (state) => {
      state.evaluatedOutput = null;
      state.error = null;
    },
  },
});

export const {
  fetchTransformationsRequest,
  fetchTransformationsSuccess,
  fetchTransformationsFailure,
  testTransformationRequest,
  testTransformationSuccess,
  testTransformationFailure,
  clearEvaluatedOutput,
} = transformationsSlice.actions;

export default transformationsSlice.reducer;

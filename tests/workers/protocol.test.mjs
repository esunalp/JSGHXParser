import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  WorkerMessageType,
  createLoadGraphPayload,
  createUpdateSliderPayload,
  createEvaluationResultPayload,
  isWorkerResponse,
  isEvaluationResult,
  isLoadGraphResult,
} from '../../poc-ghx-three/workers/protocol.js';

test('WorkerMessageType bevat de nieuwe protocolwaarden', () => {
  assert.equal(WorkerMessageType.LOAD_GHX, 'ghx/load');
  assert.equal(WorkerMessageType.LOAD_GHX_RESULT, 'ghx/load/result');
  assert.equal(WorkerMessageType.UPDATE_SLIDER, 'ghx/update-slider');
  assert.equal(WorkerMessageType.EVALUATION_RESULT, 'ghx/evaluation/result');
  assert.ok(Object.isFrozen(WorkerMessageType));
});

test('createLoadGraphPayload valideert en kopieert invoer', () => {
  assert.throws(() => createLoadGraphPayload({ contents: 42 }), /contents/);
  const metadata = { foo: 'bar' };
  const payload = createLoadGraphPayload({
    contents: '<xml />',
    name: 'test.ghx',
    graphId: 10,
    metadata,
    prefix: 'wireframe',
    setActive: false,
  });
  assert.deepEqual(payload, {
    contents: '<xml />',
    name: 'test.ghx',
    graphId: '10',
    metadata: { foo: 'bar' },
    prefix: 'wireframe',
    setActive: false,
  });
  assert.notEqual(payload.metadata, metadata, 'metadata wordt gekopieerd');
});

test('createUpdateSliderPayload filtert en normaliseert sliders', () => {
  assert.throws(() => createUpdateSliderPayload({ sliderValues: [] }), /graphId/);
  const payload = createUpdateSliderPayload({
    graphId: 5,
    sliderValues: [
      { nodeId: 'A', value: '10.5' },
      { id: 'B', value: 3, graphId: 8 },
      { nodeId: '', value: 2 },
      { nodeId: 'C', value: 'NaN' },
      null,
    ],
    setActive: false,
  });
  assert.deepEqual(payload, {
    graphId: '5',
    sliderValues: [
      { nodeId: 'A', value: 10.5 },
      { nodeId: 'B', value: 3, graphId: '8' },
    ],
    setActive: false,
  });
});

test('createEvaluationResultPayload normaliseert logs en errors', () => {
  const payload = createEvaluationResultPayload({
    graphId: 'wire',
    display: { buffers: [] },
    sliders: [{ id: 'A', value: 1 }],
    summary: 123,
    logs: [
      { level: 'info', message: 'ok', timestamp: 1 },
      { message: 12 },
      'invalid',
    ],
    errors: [
      { message: 'fail' },
    ],
  });
  assert.equal(payload.graphId, 'wire');
  assert.deepEqual(payload.display, { buffers: [] });
  assert.deepEqual(payload.sliders, [{ id: 'A', value: 1 }]);
  assert.equal(payload.summary, '123');
  assert.equal(payload.logs.length, 3);
  assert.deepEqual(payload.logs[0], { level: 'info', message: 'ok', timestamp: 1 });
  assert.equal(payload.logs[1].level, 'info');
  assert.equal(payload.logs[1].message, '12');
  assert.equal(payload.logs[1].timestamp, null);
  assert.deepEqual(payload.logs[2], { level: 'info', message: '', timestamp: null });
  assert.equal(payload.errors.length, 1);
  assert.equal(payload.errors[0].level, 'error');
  assert.equal(payload.errors[0].message, 'fail');
});

test('hulpfuncties herkennen responses op basis van type', () => {
  const evaluationMessage = {
    id: 1,
    type: WorkerMessageType.EVALUATION_RESULT,
    payload: createEvaluationResultPayload({ graphId: 'abc' }),
  };
  const loadMessage = {
    id: 2,
    type: WorkerMessageType.LOAD_GHX_RESULT,
    payload: { graphId: 'def' },
  };
  assert.ok(isWorkerResponse(evaluationMessage));
  assert.ok(isEvaluationResult(evaluationMessage));
  assert.ok(isLoadGraphResult(loadMessage));
  assert.ok(!isEvaluationResult(loadMessage));
  assert.ok(!isLoadGraphResult({ type: WorkerMessageType.LOG }));
});

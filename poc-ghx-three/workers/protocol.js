export const WorkerMessageType = Object.freeze({
  INIT: 'ghx/init',
  INIT_RESULT: 'ghx/init/result',
  PARSE_GHX: 'ghx/parse',
  PARSE_GHX_RESULT: 'ghx/parse/result',
  EVALUATE_GRAPH: 'ghx/evaluate',
  EVALUATE_GRAPH_RESULT: 'ghx/evaluate/result',
  LOG: 'ghx/log',
  ERROR: 'ghx/error',
});

export function isWorkerResponse(message) {
  return Boolean(message && typeof message.type === 'string' && 'payload' in message);
}

export function createRequest(type, payload = {}) {
  return { type, payload };
}

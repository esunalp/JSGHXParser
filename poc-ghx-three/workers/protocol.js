export const WorkerMessageType = Object.freeze({
  INIT: 'ghx/init',
  INIT_RESULT: 'ghx/init/result',
  LOAD_GHX: 'ghx/load',
  LOAD_GHX_RESULT: 'ghx/load/result',
  UPDATE_SLIDER: 'ghx/update-slider',
  EVALUATION_RESULT: 'ghx/evaluation/result',
  LOG: 'ghx/log',
  ERROR: 'ghx/error',
});

export function isWorkerResponse(message) {
  return Boolean(message && typeof message.type === 'string' && 'payload' in message);
}

export function createLoadGraphPayload({
  contents,
  name,
  graphId,
  metadata,
  prefix,
  setActive,
} = {}) {
  if (typeof contents !== 'string') {
    throw new TypeError('createLoadGraphPayload vereist een string contents.');
  }
  const payload = {
    contents,
    setActive: typeof setActive === 'boolean' ? setActive : true,
  };
  if (typeof name === 'string' && name.trim().length > 0) {
    payload.name = name;
  }
  if (graphId !== undefined && graphId !== null) {
    payload.graphId = String(graphId);
  }
  if (typeof prefix === 'string' && prefix.length > 0) {
    payload.prefix = prefix;
  }
  if (metadata && typeof metadata === 'object') {
    payload.metadata = { ...metadata };
  }
  return payload;
}

function normalizeSliderValue(entry) {
  if (!entry) {
    return null;
  }
  const nodeId = entry.nodeId ?? entry.id;
  if (!nodeId) {
    return null;
  }
  const value = Number(entry.value);
  if (!Number.isFinite(value)) {
    return null;
  }
  const normalized = {
    nodeId: String(nodeId),
    value,
  };
  if (entry.graphId !== undefined && entry.graphId !== null) {
    normalized.graphId = String(entry.graphId);
  }
  return normalized;
}

export function createUpdateSliderPayload({ graphId, sliderValues, setActive } = {}) {
  if (graphId === undefined || graphId === null || graphId === '') {
    throw new TypeError('createUpdateSliderPayload vereist een graphId.');
  }
  const normalizedValues = Array.isArray(sliderValues)
    ? sliderValues.map((entry) => normalizeSliderValue(entry)).filter(Boolean)
    : [];
  return {
    graphId: String(graphId),
    sliderValues: normalizedValues,
    setActive: typeof setActive === 'boolean' ? setActive : true,
  };
}

function normalizeLogEntry(entry, fallbackLevel = 'info') {
  if (!entry || (typeof entry !== 'object' && typeof entry !== 'function')) {
    return { level: fallbackLevel, message: '', timestamp: null };
  }
  const level = typeof entry.level === 'string' && entry.level ? entry.level : fallbackLevel;
  let message;
  if (typeof entry.message === 'string') {
    message = entry.message;
  } else if (entry.message !== undefined && entry.message !== null) {
    message = String(entry.message);
  } else {
    message = '';
  }
  const timestamp = Number.isFinite(entry.timestamp) ? entry.timestamp : null;
  const normalized = { level, message, timestamp };
  if (entry.stack !== undefined) {
    normalized.stack = typeof entry.stack === 'string' ? entry.stack : String(entry.stack ?? '');
  }
  return normalized;
}

export function createEvaluationResultPayload({
  graphId,
  display,
  sliders,
  summary,
  logs,
  errors,
  metadata,
} = {}) {
  if (graphId === undefined || graphId === null || graphId === '') {
    throw new TypeError('createEvaluationResultPayload vereist een graphId.');
  }
  const normalizedSliders = Array.isArray(sliders)
    ? sliders.map((slider) => (slider && typeof slider === 'object' ? { ...slider } : slider)).filter(Boolean)
    : [];
  return {
    graphId: String(graphId),
    display: display ?? null,
    sliders: normalizedSliders,
    summary: typeof summary === 'string' ? summary : summary == null ? '' : String(summary),
    logs: Array.isArray(logs) ? logs.map((entry) => normalizeLogEntry(entry, 'info')) : [],
    errors: Array.isArray(errors) ? errors.map((entry) => normalizeLogEntry(entry, 'error')) : [],
    metadata: metadata && typeof metadata === 'object' ? { ...metadata } : null,
  };
}

export function isEvaluationResult(message) {
  return (
    isWorkerResponse(message) &&
    message.type === WorkerMessageType.EVALUATION_RESULT &&
    message.payload &&
    typeof message.payload.graphId === 'string'
  );
}

export function isLoadGraphResult(message) {
  return (
    isWorkerResponse(message) &&
    message.type === WorkerMessageType.LOAD_GHX_RESULT &&
    message.payload &&
    typeof message.payload.graphId === 'string'
  );
}

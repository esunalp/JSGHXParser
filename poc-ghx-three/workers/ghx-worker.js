import { withVersion } from '../version.js';
import { serializeDisplayPayload } from '../renderable-transfer.js';
import {
  WorkerMessageType,
  createEvaluationResultPayload,
  createUpdateSliderPayload,
} from './protocol.js';

const versionedImport = (path) => import(withVersion(path));

let initialized = false;
let parseGHXImpl = null;
let createEngineImpl = null;
let defaultRegistry = null;
let createGraphRegistryImpl = null;
let engine = null;
let graphRegistry = null;
let lastDisplayPayload = null;
let pendingLogs = [];
let pendingErrors = [];

function clearLogBuffers() {
  pendingLogs = [];
  pendingErrors = [];
}

function recordLog(level, message) {
  if (!message) {
    return;
  }
  const target = level === 'error' ? pendingErrors : pendingLogs;
  target.push({
    level,
    message,
    timestamp: Date.now(),
  });
}

async function ensureInitialized() {
  if (initialized) {
    return;
  }
  const [loaderModule, engineModule, registryModule, graphRegistryModule] = await Promise.all([
    versionedImport('../ghx-loader.js'),
    versionedImport('../engine.js'),
    versionedImport('../registry.js'),
    versionedImport('../graph-registry.js'),
  ]);

  parseGHXImpl = loaderModule.parseGHX;
  createEngineImpl = engineModule.createEngine;
  defaultRegistry = registryModule.defaultRegistry;
  createGraphRegistryImpl = graphRegistryModule.createGraphRegistry;

  graphRegistry = createGraphRegistryImpl();

  engine = createEngineImpl({
    registry: defaultRegistry,
    graphRegistry,
    updateMesh: (payload) => {
      lastDisplayPayload = payload;
    },
    onLog: (message) => recordLog('info', message),
    onError: (message) => recordLog('error', message),
  });

  initialized = true;
}

function summarizeGraph(graph, sliderCount) {
  const nodeCount = Array.isArray(graph?.nodes) ? graph.nodes.length : 0;
  const wireCount = Array.isArray(graph?.wires) ? graph.wires.length : 0;
  const summaryParts = [];
  summaryParts.push(`${nodeCount} nodes`);
  summaryParts.push(sliderCount === 1 ? '1 slider' : `${sliderCount} sliders`);
  summaryParts.push(`${wireCount} wires`);
  return summaryParts.join(', ');
}

function collectSlidersForGraph(graphId) {
  if (!engine || !graphId) {
    return [];
  }
  const state = engine.graphStates.get(graphId);
  if (!state) {
    return [];
  }
  const sliders = [];
  for (const node of state.graph?.nodes ?? []) {
    if (!node) continue;
    const implementation = engine.componentRegistry.lookup(node);
    if (!implementation || implementation.type !== 'slider') continue;
    let nodeState = state.nodeStates.get(node.id);
    if (!nodeState && typeof implementation.createState === 'function') {
      nodeState = implementation.createState(node);
      if (nodeState) {
        state.nodeStates.set(node.id, nodeState);
      }
    }
    if (!nodeState) {
      nodeState = {};
    }
    sliders.push({
      id: node.id,
      label: nodeState.label ?? node.name ?? node.id,
      value: nodeState.value,
      min: nodeState.min,
      max: nodeState.max,
      step: nodeState.step,
      graphId,
    });
  }
  return sliders;
}

function toVirtualFile({ name, contents }) {
  return {
    name,
    text: () => Promise.resolve(typeof contents === 'string' ? contents : ''),
  };
}

function postResponse({ id, type, payload, error }, transferables = []) {
  const message = { id, type, payload: payload ?? null };
  if (error) {
    message.error = {
      message: error.message ?? String(error),
      stack: error.stack ?? null,
    };
  }
  self.postMessage(message, transferables);
}

async function handleInit({ id }) {
  await ensureInitialized();
  postResponse({ id, type: WorkerMessageType.INIT_RESULT, payload: { status: 'ok' } });
}

async function handleLoadGHX({ id, payload }) {
  await ensureInitialized();
  const name = payload?.name ?? 'graph.ghx';
  const contents = payload?.contents;
  if (typeof contents !== 'string') {
    throw new Error('parse: contents ontbreekt of is niet van het type string.');
  }
  const virtualFile = toVirtualFile({ name, contents });
  const graph = await parseGHXImpl(virtualFile);
  const metadata = { ...(graph?.metadata ?? {}), ...(payload?.metadata ?? {}) };
  const registration = graphRegistry.registerGraph(graph, {
    id: payload?.graphId,
    metadata,
    prefix: payload?.prefix,
  });
  const graphId = registration.entry.id;
  if (payload?.setActive !== false) {
    graphRegistry.setActiveGraph(graphId);
  }
  const sliders = collectSlidersForGraph(graphId);
  const sliderCount = sliders.length;
  registration.entry.metadata.sliderCount = sliderCount;
  registration.entry.graph.metadata = {
    ...(registration.entry.graph.metadata ?? {}),
    sliderCount,
  };
  const summary = summarizeGraph(registration.entry.graph, sliderCount);
  postResponse({
    id,
    type: WorkerMessageType.LOAD_GHX_RESULT,
    payload: {
      graph: registration.entry.graph,
      metadata: registration.entry.metadata,
      graphId,
      status: registration.status,
      sliders,
      summary,
    },
  });
}

async function applySliderValues(graphId, sliderValues) {
  if (!graphId || !Array.isArray(sliderValues)) {
    return;
  }
  for (const entry of sliderValues) {
    if (!entry) continue;
    const nodeId = entry.nodeId ?? entry.id;
    if (!nodeId) continue;
    const value = Number(entry.value);
    if (!Number.isFinite(value)) continue;
    const targetGraphId = entry.graphId ?? graphId;
    engine.setSliderValue(nodeId, value, { graphId: targetGraphId, silent: true });
  }
  engine.refreshSliderLinks?.({ emit: false });
}

async function handleUpdateSlider({ id, payload }) {
  await ensureInitialized();
  const sanitizedPayload = createUpdateSliderPayload(payload);
  const graphId = sanitizedPayload.graphId;
  const existing = graphRegistry.getGraph(graphId);
  if (!existing) {
    throw new Error(`evaluate: Onbekende graphId ${graphId}.`);
  }
  if (sanitizedPayload.setActive !== false) {
    graphRegistry.setActiveGraph(graphId);
  }

  await applySliderValues(graphId, sanitizedPayload.sliderValues ?? []);

  clearLogBuffers();
  lastDisplayPayload = null;
  let evaluationError = null;
  try {
    engine.evaluate({ emitStartEvent: false });
  } catch (error) {
    evaluationError = error;
    recordLog('error', error?.message ?? String(error));
  }
  const { payload: serializedDisplay, transferables } = serializeDisplayPayload(lastDisplayPayload);
  const sliders = collectSlidersForGraph(graphId);
  const sliderCount = sliders.length;
  const summary = summarizeGraph(existing.graph, sliderCount);
  const responsePayload = createEvaluationResultPayload({
    graphId,
    display: serializedDisplay,
    sliders,
    summary,
    logs: pendingLogs.slice(),
    errors: pendingErrors.slice(),
    metadata: existing.metadata,
  });
  if (evaluationError) {
    responsePayload.errors.push(
      createEvaluationResultPayload({
        graphId,
        errors: [
          {
            level: 'error',
            message: evaluationError.message ?? String(evaluationError),
            stack: evaluationError.stack ?? null,
          },
        ],
      }).errors[0],
    );
  }
  postResponse(
    {
      id,
      type: WorkerMessageType.EVALUATION_RESULT,
      payload: responsePayload,
    },
    transferables,
  );
  clearLogBuffers();
}

self.addEventListener('message', (event) => {
  const message = event.data;
  if (!message || typeof message.type !== 'string') {
    return;
  }
  const { id, type } = message;
  (async () => {
    switch (type) {
      case WorkerMessageType.INIT:
        await handleInit({ id });
        break;
      case WorkerMessageType.LOAD_GHX:
        await handleLoadGHX(message);
        break;
      case WorkerMessageType.UPDATE_SLIDER:
        await handleUpdateSlider(message);
        break;
      default:
        throw new Error(`Onbekend worker commando: ${type}`);
    }
  })().catch((error) => {
    postResponse({
      id,
      type: WorkerMessageType.ERROR,
      payload: {
        originType: type,
        message: error?.message ?? String(error),
      },
      error,
    });
  });
});

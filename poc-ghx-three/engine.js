import { withVersion } from './version.js';

const { surfaceToGeometry, isSurfaceDefinition } = await import(withVersion('./surface-mesher.js'));
const { normalizeGraph } = await import(withVersion('./graph-registry.js'));

function topoSort(nodes, wires) {
  const inDegree = new Map();
  const outgoing = new Map();

  for (const node of nodes) {
    inDegree.set(node.id, 0);
    outgoing.set(node.id, []);
  }

  for (const wire of wires) {
    if (!wire?.from?.node || !wire?.to?.node) continue;
    if (!outgoing.has(wire.from.node)) continue;
    if (!inDegree.has(wire.to.node)) continue;
    outgoing.get(wire.from.node).push(wire.to.node);
    inDegree.set(wire.to.node, inDegree.get(wire.to.node) + 1);
  }

  const queue = [];
  for (const node of nodes) {
    if (inDegree.get(node.id) === 0) {
      queue.push(node.id);
    }
  }

  const sorted = [];
  while (queue.length > 0) {
    const current = queue.shift();
    sorted.push(current);
    for (const next of outgoing.get(current) ?? []) {
      inDegree.set(next, inDegree.get(next) - 1);
      if (inDegree.get(next) === 0) {
        queue.push(next);
      }
    }
  }

  if (sorted.length !== nodes.length) {
    return { order: sorted, hasCycle: true };
  }

  return { order: sorted, hasCycle: false };
}

function createInputIndex(wires) {
  const index = new Map();
  for (const wire of wires) {
    if (!wire?.to?.node || !wire?.to?.pin) continue;
    const nodeId = wire.to.node;
    const pinName = wire.to.pin;
    if (!index.has(nodeId)) {
      index.set(nodeId, new Map());
    }
    if (!index.get(nodeId).has(pinName)) {
      index.get(nodeId).set(pinName, []);
    }
    index.get(nodeId).get(pinName).push(wire);
  }
  return index;
}

function resolveInputs(node, inputIndex, outputs) {
  const pins = new Map();
  const nodeIndex = inputIndex.get(node.id);
  if (nodeIndex) {
    for (const [pin, wires] of nodeIndex.entries()) {
      const values = [];
      for (const wire of wires) {
        const upstreamOutputs = outputs.get(wire.from.node);
        if (!upstreamOutputs) continue;
        const value = upstreamOutputs[wire.from.pin] ?? upstreamOutputs[wire.from.pin?.toLowerCase?.()];
        if (value !== undefined) {
          values.push(value);
        }
      }
      if (values.length === 1) {
        pins.set(pin, values[0]);
      } else if (values.length > 1) {
        pins.set(pin, values);
      }
    }
  }

  if (node?.inputs) {
    for (const [pin, value] of Object.entries(node.inputs)) {
      if (!pins.has(pin)) {
        pins.set(pin, value);
      }
    }
  }

  const result = {};
  for (const [pin, value] of pins.entries()) {
    result[pin] = value;
  }
  return result;
}

function isRenderableCandidate(value) {
  return Boolean(
    value?.isMesh
    || value?.isLine
    || value?.isLineSegments
    || value?.isPoints
    || value?.isSprite
    || value?.isBufferGeometry
    || value?.isGeometry
    || value?.type === 'field-display'
  );
}

function collectRenderables(value, results = [], visited = new Set()) {
  if (value === undefined || value === null) {
    return results;
  }

  const valueType = typeof value;
  if (valueType !== 'object' && valueType !== 'function') {
    return results;
  }

  if (visited.has(value)) {
    return results;
  }
  visited.add(value);

  if (ArrayBuffer.isView(value) && !(value instanceof DataView)) {
    return results;
  }

  if (isRenderableCandidate(value)) {
    results.push(value);
    return results;
  }

  const direct = value?.mesh ?? value?.geom ?? value?.geometry ?? null;
  if (direct) {
    collectRenderables(direct, results, visited);
  }

  const isSurface = isSurfaceDefinition(value);
  if (isSurface) {
    const surfaceGeometry = surfaceToGeometry(value);
    if (surfaceGeometry) {
      results.push(surfaceGeometry);
    }
  }

  if (Array.isArray(value)) {
    for (const entry of value) {
      collectRenderables(entry, results, visited);
    }
    return results;
  }

  const skipKeys = new Set();
  if (value && typeof value === 'object') {
    if (value.mesh) skipKeys.add('mesh');
    if (value.geom) skipKeys.add('geom');
    if (value.geometry) skipKeys.add('geometry');
    if (isSurface) skipKeys.add('surface');
  }

  for (const [key, entry] of Object.entries(value)) {
    if (skipKeys.has(key)) {
      continue;
    }
    collectRenderables(entry, results, visited);
  }

  return results;
}

function collectOverlayData(value, overlay, visited = new Set()) {
  if (!overlay || value === undefined || value === null) {
    return;
  }

  if (value?.isVector3) {
    overlay.points.push(value.clone());
    return;
  }

  const valueType = typeof value;
  if (valueType !== 'object' && valueType !== 'function') {
    return;
  }

  if (visited.has(value)) {
    return;
  }
  visited.add(value);

  if (value?.isObject3D || value?.isBufferGeometry || value?.isMaterial || value?.isTexture) {
    return;
  }

  if (ArrayBuffer.isView(value) && !(value instanceof DataView)) {
    return;
  }

  const hasStartEnd = value?.start?.isVector3 && value?.end?.isVector3;
  if (hasStartEnd) {
    const startPoint = value.start.clone();
    const endPoint = value.end.clone();
    overlay.segments.push({ start: startPoint, end: endPoint });
    overlay.points.push(startPoint.clone());
    overlay.points.push(endPoint.clone());
  }

  if (value?.point?.isVector3) {
    overlay.points.push(value.point.clone());
  }

  const hasPointsArray = Array.isArray(value?.points);
  if (hasPointsArray) {
    const polyPoints = value.points.filter((pt) => pt?.isVector3);
    if (polyPoints.length === 1) {
      overlay.points.push(polyPoints[0].clone());
    } else if (polyPoints.length > 1) {
      for (let index = 0; index < polyPoints.length - 1; index += 1) {
        overlay.segments.push({
          start: polyPoints[index].clone(),
          end: polyPoints[index + 1].clone(),
        });
      }
      if (value.closed && polyPoints.length > 2) {
        overlay.segments.push({
          start: polyPoints[polyPoints.length - 1].clone(),
          end: polyPoints[0].clone(),
        });
      }
    }
  }

  if (Array.isArray(value)) {
    for (const entry of value) {
      collectOverlayData(entry, overlay, visited);
    }
    return;
  }

  const skipKeys = new Set();
  if (hasStartEnd) {
    skipKeys.add('start');
    skipKeys.add('end');
  }
  if (hasPointsArray) {
    skipKeys.add('points');
  }

  for (const [key, entry] of Object.entries(value)) {
    if (skipKeys.has(key)) {
      continue;
    }
    collectOverlayData(entry, overlay, visited);
  }
}

class Engine {
  constructor({ registry, componentRegistry, graphRegistry, updateMesh, onLog, onError } = {}) {
    this.componentRegistry = componentRegistry ?? registry;
    if (!this.componentRegistry) {
      throw new Error('Engine vereist een component registry.');
    }
    if (!graphRegistry) {
      throw new Error('Engine vereist een GraphRegistry instance.');
    }

    this.registry = this.componentRegistry;
    this.graphRegistry = graphRegistry;
    this.updateMesh = updateMesh;
    this.onLog = onLog ?? (() => {});
    this.onError = onError ?? (() => {});

    this.listeners = new Map();
    this.graphStates = new Map();
    this.activeGraphId = null;
    this.pendingEvaluation = false;
    this.nodeOutputs = new Map();

    const activeGraph = this.graphRegistry.getActiveGraph();
    if (activeGraph) {
      this.activeGraphId = activeGraph.id;
      const state = this.prepareGraphState(activeGraph.id, activeGraph.graph);
      this.nodeOutputs = state.nodeOutputs;
    }

    this.graphRegistry.on('graph-added', ({ id, graph }) => {
      this.prepareGraphState(id, graph);
    });

    this.graphRegistry.on('graph-updated', ({ id, graph }) => {
      this.prepareGraphState(id, graph);
      if (this.activeGraphId === id) {
        this.emit('sliders-changed');
        this.emit('evaluation', { message: 'Grafiek bijgewerkt. Klaar voor evaluatie.' });
      }
    });

    this.graphRegistry.on('graph-removed', ({ id }) => {
      this.graphStates.delete(id);
      if (this.activeGraphId === id) {
        this.activeGraphId = null;
        this.nodeOutputs = new Map();
        if (typeof this.updateMesh === 'function') {
          this.updateMesh(null);
        }
        this.emit('sliders-changed');
        this.emit('evaluation', { message: 'Actieve grafiek verwijderd.' });
      }
    });

    this.graphRegistry.on('active-graph-changed', ({ id, graph }) => {
      if (id && graph && !this.graphStates.has(id)) {
        this.prepareGraphState(id, graph);
      }
      this.activeGraphId = id ?? null;
      const state = this.getActiveState();
      this.nodeOutputs = state?.nodeOutputs ?? new Map();
      if (typeof this.updateMesh === 'function') {
        this.updateMesh(null);
      }
      if (id) {
        this.emit('sliders-changed');
        this.emit('evaluation', { message: 'Grafiek geladen. Klaar voor evaluatie.' });
      } else {
        this.emit('sliders-changed');
        this.emit('evaluation', { message: 'Geen actieve grafiek.' });
      }
    });
  }

  emit(event, payload) {
    const list = this.listeners.get(event);
    if (!list) return;
    for (const listener of list) {
      try {
        listener(payload);
      } catch (error) {
        console.error('Engine listener error', error);
      }
    }
  }

  on(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event).add(callback);
    return () => {
      this.listeners.get(event)?.delete(callback);
    };
  }

  getActiveState() {
    if (!this.activeGraphId) {
      return null;
    }
    const existing = this.graphStates.get(this.activeGraphId);
    if (existing) {
      return existing;
    }
    const activeGraph = this.graphRegistry.getActiveGraph();
    if (activeGraph) {
      return this.prepareGraphState(activeGraph.id, activeGraph.graph);
    }
    return null;
  }

  prepareGraphState(graphId, graph) {
    const normalized = normalizeGraph(graph);
    const nodeById = new Map();
    const nodeStates = new Map();
    const nodeOutputs = new Map();

    for (const node of normalized.nodes) {
      if (!node?.id) continue;
      nodeById.set(node.id, node);
      const implementation = this.componentRegistry.lookup(node);
      if (implementation?.createState) {
        nodeStates.set(node.id, implementation.createState(node));
      }
    }

    const inputIndex = createInputIndex(normalized.wires);
    const topo = topoSort(normalized.nodes, normalized.wires);
    if (topo.hasCycle) {
      this.onError('Waarschuwing: cyclus gedetecteerd in graaf. Evaluatie kan onvoorspelbaar zijn.');
    }

    const state = {
      graph: normalized,
      nodeById,
      nodeStates,
      nodeOutputs,
      inputIndex,
      topology: topo.order,
      hasCycle: topo.hasCycle,
    };

    this.graphStates.set(graphId, state);
    if (this.activeGraphId === graphId) {
      this.nodeOutputs = state.nodeOutputs;
    }
    return state;
  }

  loadGraph(graph, options = {}) {
    const registration = this.graphRegistry.registerGraph(graph, options);
    this.graphRegistry.setActiveGraph(registration.entry.id);
    return registration.entry.id;
  }

  evaluate({ emitStartEvent = true } = {}) {
    const state = this.getActiveState();
    if (!state) {
      if (emitStartEvent) {
        this.emit('evaluation', { message: 'Geen actieve grafiek om te evalueren.' });
      }
      return;
    }

    if (emitStartEvent) {
      this.emit('evaluation-start', { reason: 'evaluate', graphId: this.activeGraphId });
    }
    try {
      if (!state.topology.length) {
        this.emit('evaluation', { message: 'Geen nodes om te evalueren.' });
        return;
      }

      const outputs = new Map();
      state.nodeOutputs.clear();
      const overlay = { segments: [], points: [] };
      const overlayVisited = new Set();
      const renderables = [];
      for (const nodeId of state.topology) {
        const node = state.nodeById.get(nodeId);
        if (!node) continue;
        const implementation = this.componentRegistry.lookup(node);
        if (!implementation) {
          this.onLog(`Geen registry-entry voor node: ${node.name ?? node.guid ?? node.id}`);
          continue;
        }

        const existingState = state.nodeStates.get(nodeId);
        const nodeState = existingState ?? (implementation.createState ? implementation.createState(node) : undefined);
        if (nodeState && !existingState) {
          state.nodeStates.set(nodeId, nodeState);
        }

        const resolvedInputs = resolveInputs(node, state.inputIndex, outputs);

        let result = {};
        try {
          if (typeof implementation.eval === 'function') {
            result = implementation.eval({ node, inputs: resolvedInputs, state: nodeState, engine: this }) || {};
          }
        } catch (error) {
          this.onError(`Evaluatie fout bij node ${node.name ?? node.id}: ${error.message}`);
          console.error(error);
          result = {};
        }

        outputs.set(nodeId, result);
        state.nodeOutputs.set(nodeId, result);

        if (node.hidden) {
          continue;
        }

        collectOverlayData(result, overlay, overlayVisited);

        const nodeRenderables = collectRenderables(result);
        for (const entry of nodeRenderables) {
          if (!entry) {
            continue;
          }
          renderables.push(entry);
        }
      }

      if (typeof this.updateMesh === 'function') {
        let main = null;
        if (renderables.length === 1) {
          [main] = renderables;
        } else if (renderables.length > 1) {
          main = renderables.slice();
        }
        this.updateMesh({ type: 'ghx-display', main, overlays: overlay });
      }

      const message = `Laatste evaluatie: ${new Date().toLocaleTimeString()}`;
      this.emit('evaluation', { message });
    } finally {
      this.emit('evaluation-complete', { reason: 'evaluate', graphId: this.activeGraphId });
    }
  }

  listSliders() {
    const state = this.getActiveState();
    if (!state) {
      return [];
    }

    const sliders = [];
    for (const node of state.graph.nodes) {
      const implementation = this.componentRegistry.lookup(node);
      if (!implementation || implementation.type !== 'slider') continue;
      const nodeState = state.nodeStates.get(node.id) ?? implementation.createState?.(node) ?? {};
      sliders.push({
        id: node.id,
        label: nodeState.label ?? node.name ?? node.id,
        value: nodeState.value ?? 0,
        min: nodeState.min ?? 0,
        max: nodeState.max ?? 1,
        step: nodeState.step ?? 0.01,
        graphId: this.activeGraphId,
      });
    }
    return sliders;
  }

  setSliderValue(nodeId, value, { graphId } = {}) {
    const targetGraphId = graphId ?? this.activeGraphId;
    if (!targetGraphId) return false;

    const state = this.graphStates.get(targetGraphId);
    if (!state) return false;

    const node = state.nodeById.get(nodeId);
    if (!node) return false;

    const implementation = this.componentRegistry.lookup(node);
    if (!implementation || implementation.type !== 'slider') return false;

    let nodeState = state.nodeStates.get(nodeId);
    if (!nodeState) {
      nodeState = implementation.createState?.(node) ?? { value };
      state.nodeStates.set(nodeId, nodeState);
    }
    nodeState.value = value;
    state.nodeStates.set(nodeId, nodeState);

    if (targetGraphId === this.activeGraphId) {
      this.emit('sliders-changed');
      this.scheduleEvaluation();
    }

    return true;
  }

  scheduleEvaluation() {
    if (!this.activeGraphId) {
      return;
    }
    const state = this.graphStates.get(this.activeGraphId);
    if (!state) {
      return;
    }
    if (this.pendingEvaluation) {
      return;
    }
    this.pendingEvaluation = true;
    this.emit('evaluation-start', { reason: 'queued', graphId: this.activeGraphId });
    const triggerEvaluation = () => {
      this.pendingEvaluation = false;
      this.evaluate({ emitStartEvent: false });
    };
    const raf = typeof globalThis !== 'undefined' ? globalThis.requestAnimationFrame : undefined;
    if (typeof raf === 'function') {
      raf(() => triggerEvaluation());
      return;
    }
    Promise.resolve().then(triggerEvaluation);
  }

  getNodeOutput(nodeId, { graphId } = {}) {
    const targetGraphId = graphId ?? this.activeGraphId;
    if (!targetGraphId) return null;
    const state = this.graphStates.get(targetGraphId) ?? null;
    if (!state) return null;
    return state.nodeOutputs.get(nodeId) ?? null;
  }
}

export function createEngine(options) {
  return new Engine(options);
}

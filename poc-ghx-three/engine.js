import { withVersion } from './version.js';

const { surfaceToGeometry, isSurfaceDefinition } = await import(withVersion('./surface-mesher.js'));

function normalizeGraph(graph) {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : [];
  const wires = Array.isArray(graph?.wires) ? graph.wires : [];
  return { nodes, wires };
}

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
    overlay.segments.push({ start: value.start.clone(), end: value.end.clone() });
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
  constructor({ registry, updateMesh, onLog, onError }) {
    this.registry = registry;
    this.updateMesh = updateMesh;
    this.onLog = onLog ?? (() => {});
    this.onError = onError ?? (() => {});

    this.graph = { nodes: [], wires: [] };
    this.nodeById = new Map();
    this.nodeStates = new Map();
    this.nodeOutputs = new Map();
    this.inputIndex = new Map();
    this.topology = [];
    this.listeners = new Map();
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

  loadGraph(graph) {
    const normalized = normalizeGraph(graph);
    this.graph = normalized;
    this.nodeById.clear();
    this.nodeStates.clear();
    this.nodeOutputs.clear();

    if (typeof this.updateMesh === 'function') {
      this.updateMesh(null);
    }

    for (const node of normalized.nodes) {
      if (!node?.id) continue;
      this.nodeById.set(node.id, node);
      const implementation = this.registry.lookup(node);
      if (implementation?.createState) {
        this.nodeStates.set(node.id, implementation.createState(node));
      }
    }

    this.inputIndex = createInputIndex(normalized.wires);
    const topo = topoSort(normalized.nodes, normalized.wires);
    this.topology = topo.order;
    if (topo.hasCycle) {
      this.onError('Waarschuwing: cyclus gedetecteerd in graaf. Evaluatie kan onvoorspelbaar zijn.');
    }
    this.emit('sliders-changed');
    this.emit('evaluation', { message: 'Grafiek geladen. Klaar voor evaluatie.' });
  }

  evaluate() {
    if (!this.topology.length) {
      this.emit('evaluation', { message: 'Geen nodes om te evalueren.' });
      return;
    }

    const outputs = new Map();
    const overlay = { segments: [], points: [] };
    const overlayVisited = new Set();
    const renderables = [];
    for (const nodeId of this.topology) {
      const node = this.nodeById.get(nodeId);
      if (!node) continue;
      const implementation = this.registry.lookup(node);
      if (!implementation) {
        this.onLog(`Geen registry-entry voor node: ${node.name ?? node.guid ?? node.id}`);
        continue;
      }

      const state = this.nodeStates.get(nodeId) ?? (implementation.createState ? implementation.createState(node) : undefined);
      if (state && !this.nodeStates.has(nodeId)) {
        this.nodeStates.set(nodeId, state);
      }

      const resolvedInputs = resolveInputs(node, this.inputIndex, outputs);

      let result = {};
      try {
        if (typeof implementation.eval === 'function') {
          result = implementation.eval({ node, inputs: resolvedInputs, state, engine: this }) || {};
        }
      } catch (error) {
        this.onError(`Evaluatie fout bij node ${node.name ?? node.id}: ${error.message}`);
        console.error(error);
        result = {};
      }

      outputs.set(nodeId, result);
      this.nodeOutputs.set(nodeId, result);

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
  }

  listSliders() {
    const sliders = [];
    for (const node of this.graph.nodes) {
      const implementation = this.registry.lookup(node);
      if (!implementation || implementation.type !== 'slider') continue;
      const state = this.nodeStates.get(node.id) ?? implementation.createState?.(node) ?? {};
      sliders.push({
        id: node.id,
        label: state.label ?? node.name ?? node.id,
        value: state.value ?? 0,
        min: state.min ?? 0,
        max: state.max ?? 1,
        step: state.step ?? 0.01,
      });
    }
    return sliders;
  }

  setSliderValue(nodeId, value) {
    const node = this.nodeById.get(nodeId);
    if (!node) return false;
    const implementation = this.registry.lookup(node);
    if (!implementation || implementation.type !== 'slider') return false;
    let state = this.nodeStates.get(nodeId);
    if (!state) {
      state = implementation.createState?.(node) ?? { value: value };
      this.nodeStates.set(nodeId, state);
    }
    state.value = value;
    this.nodeStates.set(nodeId, state);
    this.emit('sliders-changed');
    this.evaluate();
    return true;
  }
}

export function createEngine(options) {
  return new Engine(options);
}

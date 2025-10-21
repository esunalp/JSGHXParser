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
  return Boolean(value?.isMesh || value?.isBufferGeometry || value?.isGeometry);
}

function extractRenderable(value, visited = new Set()) {
  if (value === undefined || value === null) {
    return null;
  }

  if (typeof value === 'object' || typeof value === 'function') {
    if (visited.has(value)) {
      return null;
    }
    visited.add(value);
  }

  if (isRenderableCandidate(value)) {
    return value;
  }

  if (value && typeof value === 'object') {
    if (value.mesh || value.geom || value.geometry) {
      const direct = value.mesh ?? value.geom ?? value.geometry;
      const directRenderable = extractRenderable(direct, visited);
      if (directRenderable) {
        return directRenderable;
      }
    }

    if (isSurfaceDefinition(value)) {
      const surfaceGeometry = surfaceToGeometry(value);
      if (surfaceGeometry) {
        return surfaceGeometry;
      }
    }
  }

  if (Array.isArray(value)) {
    for (const entry of value) {
      const renderable = extractRenderable(entry, visited);
      if (renderable) {
        return renderable;
      }
    }
    return null;
  }

  if (value && typeof value === 'object') {
    for (const key of Object.keys(value)) {
      const renderable = extractRenderable(value[key], visited);
      if (renderable) {
        return renderable;
      }
    }
  }

  return null;
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

      let renderable = result.mesh ?? result.geom ?? result.geometry;
      if (!renderable) {
        renderable = extractRenderable(result);
      }
      if (renderable) {
        this.updateMesh?.(renderable);
      }
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

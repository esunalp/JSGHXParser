let graphIdCounter = 0;

export function normalizeGraph(graph) {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes.slice() : [];
  const wires = Array.isArray(graph?.wires) ? graph.wires.slice() : [];
  return { nodes, wires };
}

function createGraphId(prefix = 'graph') {
  graphIdCounter += 1;
  return `${prefix}-${graphIdCounter}`;
}

function cloneMetadata(metadata) {
  if (!metadata || typeof metadata !== 'object') {
    return {};
  }
  return { ...metadata };
}

class EventEmitter {
  constructor() {
    this.listeners = new Map();
  }

  on(event, listener) {
    if (typeof listener !== 'function') {
      return () => {};
    }
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event).add(listener);
    return () => {
      this.listeners.get(event)?.delete(listener);
    };
  }

  emit(event, payload) {
    const listeners = this.listeners.get(event);
    if (!listeners) return;
    for (const listener of listeners) {
      try {
        listener(payload);
      } catch (error) {
        console.error(`GraphRegistry listener for ${event} threw`, error);
      }
    }
  }
}

export class GraphRegistry {
  constructor({ onGraphAdded, onGraphRemoved } = {}) {
    this.graphs = new Map();
    this.activeGraphId = null;
    this.events = new EventEmitter();

    if (typeof onGraphAdded === 'function') {
      this.on('graph-added', onGraphAdded);
    }
    if (typeof onGraphRemoved === 'function') {
      this.on('graph-removed', onGraphRemoved);
    }
  }

  on(event, listener) {
    return this.events.on(event, listener);
  }

  emit(event, payload) {
    this.events.emit(event, payload);
  }

  registerGraph(graph, { id, metadata, prefix } = {}) {
    if (!graph) {
      throw new Error('Graph ontbreekt bij registratie.');
    }

    const normalizedGraph = normalizeGraph(graph);
    const providedId = id ?? graph?.id ?? null;
    const graphId = providedId ?? createGraphId(prefix);

    const previous = this.graphs.get(graphId);
    const combinedMetadata = {
      ...(previous?.metadata ? cloneMetadata(previous.metadata) : {}),
      ...(graph?.metadata ? cloneMetadata(graph.metadata) : {}),
      ...(metadata ? cloneMetadata(metadata) : {}),
    };

    const inferredLabel = graph?.label ?? graph?.name;
    if (inferredLabel && !combinedMetadata.label) {
      combinedMetadata.label = inferredLabel;
    }
    if (typeof graph?.source === 'string' && !combinedMetadata.source) {
      combinedMetadata.source = graph.source;
    }

    const entry = {
      id: graphId,
      graph: normalizedGraph,
      metadata: combinedMetadata,
      addedAt: previous?.addedAt ?? new Date(),
      updatedAt: new Date(),
    };

    this.graphs.set(graphId, entry);

    const payload = { id: graphId, graph: normalizedGraph, metadata: cloneMetadata(combinedMetadata) };
    if (previous) {
      this.emit('graph-updated', payload);
      return { entry, status: 'updated' };
    }

    this.emit('graph-added', payload);
    return { entry, status: 'added' };
  }

  getGraph(id) {
    if (!id) return null;
    const entry = this.graphs.get(id);
    if (!entry) return null;
    return {
      id: entry.id,
      graph: entry.graph,
      metadata: cloneMetadata(entry.metadata),
      addedAt: entry.addedAt,
      updatedAt: entry.updatedAt,
    };
  }

  listGraphs() {
    return Array.from(this.graphs.values()).map((entry) => ({
      id: entry.id,
      graph: entry.graph,
      metadata: cloneMetadata(entry.metadata),
      addedAt: entry.addedAt,
      updatedAt: entry.updatedAt,
    }));
  }

  removeGraph(id) {
    if (!id) return false;
    const entry = this.graphs.get(id);
    if (!entry) return false;

    this.graphs.delete(id);
    const wasActive = this.activeGraphId === id;
    this.emit('graph-removed', {
      id,
      graph: entry.graph,
      metadata: cloneMetadata(entry.metadata),
    });

    if (wasActive) {
      this.activeGraphId = null;
      const iterator = this.graphs.keys();
      const next = iterator.next();
      if (!next.done) {
        this.setActiveGraph(next.value);
      } else {
        this.emit('active-graph-changed', { id: null, graph: null, metadata: {} });
      }
    }

    return true;
  }

  setActiveGraph(id) {
    if (!id) {
      if (this.activeGraphId !== null) {
        this.activeGraphId = null;
        this.emit('active-graph-changed', { id: null, graph: null, metadata: {} });
      }
      return null;
    }

    const entry = this.graphs.get(id);
    if (!entry) {
      throw new Error(`Onbekende graph: ${id}`);
    }

    if (this.activeGraphId === id) {
      return entry;
    }

    this.activeGraphId = id;
    this.emit('active-graph-changed', {
      id,
      graph: entry.graph,
      metadata: cloneMetadata(entry.metadata),
    });
    return entry;
  }

  getActiveGraph() {
    if (!this.activeGraphId) return null;
    return this.getGraph(this.activeGraphId);
  }
}

export function createGraphRegistry(options) {
  return new GraphRegistry(options);
}

import { withVersion } from './version.js';
import { WorkerMessageType } from './workers/protocol.js';

function createWorkerInstance() {
  const workerUrl = new URL(withVersion('./workers/ghx-worker.js'), import.meta.url);
  return new Worker(workerUrl, { type: 'module' });
}

export class GHXWorkerManager {
  constructor(options = {}) {
    this.worker = options.worker ?? createWorkerInstance();
    this.pending = new Map();
    this.nextRequestId = 1;
    this.disposed = false;

    this.handleMessage = this.handleMessage.bind(this);
    this.worker.addEventListener('message', this.handleMessage);

    this.readyPromise = this.sendRequest(WorkerMessageType.INIT, {});
  }

  async ensureReady() {
    if (this.disposed) {
      throw new Error('WorkerManager is disposed.');
    }
    await this.readyPromise;
    return true;
  }

  handleMessage(event) {
    const message = event.data;
    if (!message || typeof message.type !== 'string') {
      return;
    }
    if (!message.id) {
      if (message.type === WorkerMessageType.LOG) {
        console.info('[ghx-worker]', message.payload?.message ?? '');
      } else if (message.type === WorkerMessageType.ERROR) {
        console.warn('[ghx-worker:error]', message.payload?.message ?? '');
      }
      return;
    }
    const pending = this.pending.get(message.id);
    if (!pending) {
      return;
    }
    this.pending.delete(message.id);
    if (message.error || message.type === WorkerMessageType.ERROR) {
      const errorInfo = message.error ?? message.payload ?? {};
      const error = new Error(errorInfo.message ?? 'Worker error');
      if (errorInfo.stack) {
        error.stack = errorInfo.stack;
      }
      pending.reject(error);
      return;
    }
    pending.resolve(message.payload);
  }

  sendRequest(type, payload = {}, transferables = []) {
    if (this.disposed) {
      return Promise.reject(new Error('WorkerManager is disposed.'));
    }
    const id = this.nextRequestId++;
    const message = { id, type, payload };
    const promise = new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });
    try {
      this.worker.postMessage(message, transferables);
    } catch (error) {
      this.pending.delete(id);
      return Promise.reject(error);
    }
    return promise;
  }

  async parseFile(file, options = {}) {
    if (!file || typeof file.text !== 'function') {
      throw new Error('parseFile vereist een File-achtig object met een text() methode.');
    }
    const contents = await file.text();
    return this.parseText(contents, { name: file.name, ...options });
  }

  parseText(contents, { name, graphId, metadata, prefix, setActive } = {}) {
    if (typeof contents !== 'string') {
      return Promise.reject(new Error('parseText vereist een string als contents.'));
    }
    const payload = { contents, name, graphId, metadata, prefix, setActive };
    return this.sendRequest(WorkerMessageType.PARSE_GHX, payload);
  }

  evaluateGraph({ graphId, sliderValues, setActive = true } = {}) {
    if (!graphId) {
      return Promise.reject(new Error('evaluateGraph vereist een graphId.'));
    }
    const payload = { graphId, sliderValues, setActive };
    return this.sendRequest(WorkerMessageType.EVALUATE_GRAPH, payload);
  }

  dispose() {
    if (this.disposed) {
      return;
    }
    this.disposed = true;
    this.worker.removeEventListener('message', this.handleMessage);
    this.worker.terminate();
    const error = new Error('WorkerManager disposed');
    for (const [, pending] of this.pending.entries()) {
      pending.reject(error);
    }
    this.pending.clear();
  }
}

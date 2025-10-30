import { withVersion } from './version.js';

const CDN_BASE = 'https://cdn.jsdelivr.net/npm/three@0.180.0/';
const CACHE = new Map();

async function importViaBlob(url) {
  const response = await fetch(url, { mode: 'cors' });
  if (!response.ok) {
    throw new Error(`Kon module niet ophalen: ${url} (HTTP ${response.status})`);
  }
  const source = await response.text();
  const blob = new Blob([source], { type: 'text/javascript' });
  const blobUrl = URL.createObjectURL(blob);
  try {
    return await import(blobUrl);
  } finally {
    URL.revokeObjectURL(blobUrl);
  }
}

async function importWithFallback(key, primary, fallback) {
  if (CACHE.has(key)) {
    return CACHE.get(key);
  }
  let module;
  try {
    module = await import(primary);
  } catch (primaryError) {
    if (!fallback) {
      throw primaryError;
    }
    try {
      module = await import(fallback);
    } catch (fallbackError) {
      try {
        if (fallback.startsWith('http')) {
          module = await importViaBlob(fallback);
        } else {
          throw fallbackError;
        }
      } catch (blobError) {
        console.warn('[three-loader] Kon module niet laden, val terug op standaardimplementatie:', key, primaryError, fallbackError, blobError);
        throw blobError;
      }
    }
  }
  CACHE.set(key, module);
  return module;
}

export async function loadThreeCore() {
  return importWithFallback('core', 'three', withVersion('./three.webgpu.js'));
}

export async function loadThreeWebGPU() {
  try {
    return await importWithFallback('webgpu', 'three/webgpu', `./three.webgpu.js`);
  } catch (error) {
    console.warn('[three-loader] three/webgpu niet beschikbaar, val terug op standaard three.', error);
    return loadThreeCore();
  }
}

export async function loadThreeTSL() {
  try {
    return await importWithFallback('tsl', 'three/tsl', `./three.tsl.js`);
  } catch (error) {
    console.warn('[three-loader] three/tsl niet beschikbaar; node-material functionaliteit wordt gedeactiveerd.', error);
    return null;
  }
}

export async function loadThreeAddon(path) {
  if (!path || typeof path !== 'string') {
    throw new Error('loadThreeAddon vereist een pad-string.');
  }
  const normalized = path.startsWith('/') ? path.slice(1) : path;
  return importWithFallback(`addon:${normalized}`, `three/addons/${normalized}`, `https://cdn.jsdelivr.net/npm/three@0.180.0/build/three.webgpu.js/examples/jsm/${normalized}`);
}

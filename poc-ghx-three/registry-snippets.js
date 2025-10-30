import * as THREE from 'three';
import { createEngine } from './engine.js';
import { defaultRegistry } from './registry.js';

function createSnippetEngine() {
  return createEngine({
    registry: defaultRegistry,
    updateMesh: () => {},
    onLog: () => {},
    onError: (message) => {
      throw new Error(message);
    },
  });
}

function extractGeometry(engine, nodeId) {
  const output = engine.nodeOutputs.get(nodeId);
  if (!output) return null;
  return output.geom ?? output.geometry ?? output.mesh ?? null;
}

export function runAddBoxSnippet() {
  const engine = createSnippetEngine();
  const nodes = [
    {
      id: 'slider-a',
      guid: '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}',
      name: 'Number Slider',
      meta: { value: 2, min: 0, max: 10, step: 1 },
    },
    {
      id: 'slider-b',
      guid: '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}',
      name: 'Number Slider',
      meta: { value: 3, min: 0, max: 10, step: 1 },
    },
    {
      id: 'math-add',
      guid: '{a0d62394-a118-422d-abb3-6af115c75b25}',
      name: 'Addition',
    },
    {
      id: 'box-node',
      guid: '{56f1d440-0b71-44de-93d5-3c96bf53b78f}',
      name: 'Box',
      inputs: { H: 1, D: 1 },
    },
  ];

  const wires = [
    { from: { node: 'slider-a', pin: 'value' }, to: { node: 'math-add', pin: 'A' } },
    { from: { node: 'slider-b', pin: 'value' }, to: { node: 'math-add', pin: 'B' } },
    { from: { node: 'math-add', pin: 'R' }, to: { node: 'box-node', pin: 'W' } },
  ];

  engine.loadGraph({ nodes, wires });
  engine.evaluate();

  const geometry = extractGeometry(engine, 'box-node');
  if (!geometry || !(geometry instanceof THREE.BoxGeometry)) {
    throw new Error('Verwachtte BoxGeometry uit Add → Box keten.');
  }

  return geometry.parameters;
}

export function runCircleExtrudeSnippet() {
  const engine = createSnippetEngine();
  const nodes = [
    {
      id: 'radius-slider',
      guid: '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}',
      name: 'Number Slider',
      meta: { value: 2, min: 0, max: 10, step: 0.5 },
    },
    {
      id: 'height-slider',
      guid: '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}',
      name: 'Number Slider',
      meta: { value: 4, min: 0, max: 10, step: 0.5 },
    },
    {
      id: 'circle-node',
      guid: '{807b86e3-be8d-4970-92b5-f8cdcb45b06b}',
      name: 'Circle',
    },
    {
      id: 'extrude-node',
      guid: '{962034e9-cc27-4394-afc4-5c16e3447cf9}',
      name: 'Extrude',
    },
  ];

  const wires = [
    { from: { node: 'radius-slider', pin: 'value' }, to: { node: 'circle-node', pin: 'R' } },
    { from: { node: 'circle-node', pin: 'C' }, to: { node: 'extrude-node', pin: 'B' } },
    { from: { node: 'height-slider', pin: 'value' }, to: { node: 'extrude-node', pin: 'D' } },
  ];

  engine.loadGraph({ nodes, wires });
  engine.evaluate();

  const geometry = extractGeometry(engine, 'extrude-node');
  if (!geometry || !(geometry instanceof THREE.ExtrudeGeometry)) {
    throw new Error('Verwachtte ExtrudeGeometry uit Circle → Extrude keten.');
  }

  return {
    depth: geometry.parameters?.depth ?? null,
    settings: geometry.parameters,
  };
}

export function runPointVectorMathSnippet() {
  const engine = createSnippetEngine();
  const nodes = [
    {
      id: 'point-node',
      guid: '{3581f42a-9592-4549-bd6b-1c0fc39d067b}',
      name: 'Construct Point',
      inputs: { X: 1, Y: 2, Z: 3 },
    },
    {
      id: 'vector-node',
      guid: '{56b92eab-d121-43f7-94d3-6cd8f0ddead8}',
      name: 'Vector XYZ',
      inputs: { X: 4, Y: 5, Z: 6 },
    },
    {
      id: 'multiply-node',
      guid: '{b8963bb1-aa57-476e-a20e-ed6cf635a49c}',
      name: 'Multiplication',
    },
  ];

  const wires = [
    { from: { node: 'point-node', pin: 'Pt' }, to: { node: 'multiply-node', pin: 'A' } },
    { from: { node: 'vector-node', pin: 'V' }, to: { node: 'multiply-node', pin: 'B' } },
  ];

  engine.loadGraph({ nodes, wires });
  engine.evaluate();

  const output = engine.nodeOutputs.get('multiply-node');
  const result = output?.result ?? output?.R;
  if (!result || !result.isVector3) {
    throw new Error('Verwachtte Vector3 uit Point/Vector vermenigvuldiging.');
  }

  return result;
}

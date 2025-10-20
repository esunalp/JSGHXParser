const KNOWN_COMPONENT_GUIDS = new Set([
  '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}', // Number Slider
  '{56f1d440-0b71-44de-93d5-3c96bf53b78f}', // Box
]);

const SLIDER_GUIDS = new Set(['{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}']);

const SLIDER_DEFAULTS = {
  value: 1,
  min: 0,
  max: 10,
  step: 0.01,
};

function normalizeGuid(guid) {
  if (!guid) return null;
  const trimmed = guid.trim();
  if (!trimmed) return null;
  return trimmed.startsWith('{') && trimmed.endsWith('}')
    ? trimmed.toLowerCase()
    : `{${trimmed.toLowerCase()}}`;
}

function getFirstText(root, selectors) {
  for (const selector of selectors) {
    const element = root.querySelector(selector);
    if (element && element.textContent) {
      const value = element.textContent.trim();
      if (value) return value;
    }
  }
  return null;
}

function collectByName(root, names) {
  if (!root) return undefined;
  const lowerNames = new Set(names.map((name) => name.toLowerCase()));
  const elements = root.querySelectorAll('*[name]');
  for (const element of elements) {
    const attr = element.getAttribute('name');
    if (!attr || !lowerNames.has(attr.toLowerCase())) continue;
    const text = element.textContent?.trim();
    if (!text) continue;
    const normalized = text.replace(',', '.');
    const numeric = Number(normalized);
    if (!Number.isNaN(numeric)) {
      return numeric;
    }
  }
  return undefined;
}

function parseSliderMeta(objectChunk, fallbackName) {
  const meta = {
    label: fallbackName || 'Number Slider',
    ...SLIDER_DEFAULTS,
  };

  const sliderRelatedChunks = [
    objectChunk.querySelector('chunk[name="Slider"]'),
    objectChunk.querySelector('chunk[name="SliderData"]'),
    objectChunk.querySelector('chunk[name="SliderDomain"]'),
    objectChunk.querySelector('chunk[name="PersistentData"]'),
  ].filter(Boolean);

  const maybeValue = collectByName(objectChunk, ['Value', 'Current', 'SliderValue', 'Val']);
  const maybeMin = collectByName(objectChunk, ['LowerLimit', 'Min', 'Minimum', 'Low']);
  const maybeMax = collectByName(objectChunk, ['UpperLimit', 'Max', 'Maximum', 'High']);
  const maybeStep = collectByName(objectChunk, ['Step', 'Increment']);

  for (const chunk of sliderRelatedChunks) {
    if (maybeValue === undefined) {
      const value = collectByName(chunk, ['Value', 'Current', 'SliderValue', 'Val']);
      if (value !== undefined) meta.value = value;
    }
    if (maybeMin === undefined) {
      const min = collectByName(chunk, ['LowerLimit', 'Min', 'Minimum', 'Low']);
      if (min !== undefined) meta.min = min;
    }
    if (maybeMax === undefined) {
      const max = collectByName(chunk, ['UpperLimit', 'Max', 'Maximum', 'High']);
      if (max !== undefined) meta.max = max;
    }
    if (maybeStep === undefined) {
      const step = collectByName(chunk, ['Step', 'Increment']);
      if (step !== undefined) meta.step = step;
    }
  }

  if (maybeValue !== undefined) meta.value = maybeValue;
  if (maybeMin !== undefined) meta.min = maybeMin;
  if (maybeMax !== undefined) meta.max = maybeMax;
  if (maybeStep !== undefined) meta.step = maybeStep;

  if (meta.step <= 0 || Number.isNaN(meta.step)) {
    const range = meta.max - meta.min;
    meta.step = range > 0 ? range / 100 : SLIDER_DEFAULTS.step;
  }

  return meta;
}

function describeObjectChunk(chunk, index) {
  const allGuidElements = Array.from(chunk.querySelectorAll('guid'));
  let instanceGuid = null;
  let componentGuid = null;

  for (const guidElement of allGuidElements) {
    const attr = guidElement.getAttribute('name')?.toLowerCase();
    const text = guidElement.textContent?.trim();
    if (!text) continue;
    if (!attr || attr === 'id' || attr === 'instanceid' || attr === 'instanceguid') {
      if (!instanceGuid) {
        instanceGuid = text;
      }
    }
    if (attr === 'definitionguid' || attr === 'componentguid' || attr === 'componentid' || attr === 'classid' || attr === 'id') {
      if (!componentGuid) {
        componentGuid = text;
      }
    }
  }

  if (!componentGuid) {
    componentGuid = getFirstText(chunk, [
      'chunk[name="Proxy"] > guid',
      'chunk[name="Definition"] > guid',
    ]);
  }

  const normalizedComponentGuid = normalizeGuid(componentGuid);
  const normalizedInstanceGuid = normalizeGuid(instanceGuid);

  const name =
    getFirstText(chunk, [
      'string[name="NickName"]',
      'string[name="Name"]',
      'string[name="UserString"]',
      'chunk[name="Definition"] > string[name="Name"]',
      'chunk[name="Definition"] > string[name="NickName"]',
    ]) ||
    chunk.getAttribute('name') ||
    'Onbekende node';

  return {
    id: normalizedInstanceGuid ?? `node-${index + 1}`,
    guid: normalizedComponentGuid,
    name,
    chunk,
  };
}

function detectSliders(node) {
  if (!node) return false;
  const normalizedGuid = normalizeGuid(node.guid);
  if (normalizedGuid && SLIDER_GUIDS.has(normalizedGuid)) {
    return true;
  }
  const name = node.name?.toLowerCase?.() ?? '';
  return name.includes('slider');
}

function buildNodeDescriptor(nodeInfo) {
  const { id, guid, name, chunk } = nodeInfo;
  const descriptor = {
    id,
    guid,
    name,
    inputs: {},
    outputs: {},
    meta: {},
  };

  if (detectSliders(nodeInfo)) {
    const sliderMeta = parseSliderMeta(chunk, name);
    descriptor.meta = { ...sliderMeta };
  }

  return descriptor;
}

export async function parseGHX(file) {
  if (!file) {
    throw new Error('Geen bestand aangeleverd.');
  }
  const text = await file.text();
  const parser = new DOMParser();
  const doc = parser.parseFromString(text, 'application/xml');
  const parseError = doc.querySelector('parsererror');
  if (parseError) {
    throw new Error('Kon GHX-bestand niet parsen. Controleer of het valide XML is.');
  }

  const objectChunks = doc.querySelectorAll('chunk[name="Object"], chunk[name="Objects"] > chunk');

  const nodes = [];
  const unknownNodes = [];

  objectChunks.forEach((chunk, index) => {
    const info = describeObjectChunk(chunk, index);
    const descriptor = buildNodeDescriptor(info);
    if (descriptor.guid) {
      descriptor.guid = normalizeGuid(descriptor.guid);
    }
    nodes.push(descriptor);

    const guidKey = descriptor.guid ?? '';
    const nameKey = descriptor.name?.toLowerCase?.() ?? '';
    const isKnown =
      detectSliders(info) ||
      (guidKey && KNOWN_COMPONENT_GUIDS.has(guidKey)) ||
      nameKey === 'box';
    if (!isKnown) {
      unknownNodes.push({ id: descriptor.id, name: descriptor.name, guid: descriptor.guid });
    }
  });

  if (unknownNodes.length) {
    console.warn('parseGHX: Onbekende nodes aangetroffen', unknownNodes);
  }

  if (!nodes.length) {
    console.warn('parseGHX: Geen nodes gevonden in GHX-document.');
  }

  const wires = [];
  console.info('parseGHX: Wires worden in deze iteratie overgeslagen.');

  return { nodes, wires };
}

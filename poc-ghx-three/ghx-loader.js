import { withVersion } from './version.js';

const versionedImport = (path) => import(withVersion(path));

const [
  { COMPLEX_COMPONENTS },
  { CURVE_COMPONENTS },
  { DISPLAY_COMPONENTS },
  { INTERSECT_COMPONENTS },
  { MATHS_COMPONENTS },
  { MESH_COMPONENTS },
  { PARAMS_COMPONENTS },
  { SCALAR_COMPONENTS },
  { SETS_COMPONENTS },
  { SURFACE_COMPONENTS },
  { TRANSFORM_COMPONENTS },
  { VECTOR_COMPONENTS },
] = await Promise.all([
  versionedImport('./component-metadata-complex.js'),
  versionedImport('./component-metadata-curve.js'),
  versionedImport('./component-metadata-display.js'),
  versionedImport('./component-metadata-intersect.js'),
  versionedImport('./component-metadata-maths.js'),
  versionedImport('./component-metadata-mesh.js'),
  versionedImport('./component-metadata-params.js'),
  versionedImport('./component-metadata-scalar.js'),
  versionedImport('./component-metadata-sets.js'),
  versionedImport('./component-metadata-surface.js'),
  versionedImport('./component-metadata-transform.js'),
  versionedImport('./component-metadata-vector.js'),
]);

function normalizeGuid(guid) {
  if (!guid) return null;
  const trimmed = guid.trim();
  if (!trimmed) return null;
  return trimmed.startsWith('{') && trimmed.endsWith('}')
    ? trimmed.toLowerCase()
    : `{${trimmed.toLowerCase()}}`;
}

function addGuid(set, guid) {
  const normalized = normalizeGuid(guid);
  if (normalized) {
    set.add(normalized);
  }
}

const KNOWN_COMPONENT_GUIDS = new Set();
const PARAMETER_LIKE_GUIDS = new Set();
const SLIDER_GUIDS = new Set();
const KNOWN_COMPONENT_NAMES = new Set();
const COMPONENT_METADATA = new Map();

function addKnownName(name) {
  if (!name) return;
  KNOWN_COMPONENT_NAMES.add(String(name).toLowerCase());
}

function registerComponentMetadata(list) {
  if (!Array.isArray(list)) return;
  for (const component of list) {
    if (!component) continue;
    addGuid(KNOWN_COMPONENT_GUIDS, component.guid);
    const normalizedGuid = normalizeGuid(component.guid);
    if (normalizedGuid) {
      COMPONENT_METADATA.set(normalizedGuid, component);
      if (component.category?.toLowerCase?.() === 'params') {
        PARAMETER_LIKE_GUIDS.add(normalizedGuid);
      }
    }
    addKnownName(component.name);
    addKnownName(component.nickname);
  }
}

addGuid(KNOWN_COMPONENT_GUIDS, '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}'); // Number Slider
addGuid(KNOWN_COMPONENT_GUIDS, '{56f1d440-0b71-44de-93d5-3c96bf53b78f}'); // Box
addGuid(KNOWN_COMPONENT_GUIDS, '{59e0b89a-e487-49f8-bab8-b5bab16be14c}'); // Panel

addGuid(PARAMETER_LIKE_GUIDS, '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}'); // Number Slider
addGuid(PARAMETER_LIKE_GUIDS, '{59e0b89a-e487-49f8-bab8-b5bab16be14c}'); // Panel

addGuid(SLIDER_GUIDS, '{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}');

addKnownName('Number Slider');
addKnownName('Slider');
addKnownName('Box');
addKnownName('Panel');

registerComponentMetadata(COMPLEX_COMPONENTS);
registerComponentMetadata(CURVE_COMPONENTS);
registerComponentMetadata(DISPLAY_COMPONENTS);
registerComponentMetadata(INTERSECT_COMPONENTS);
registerComponentMetadata(MATHS_COMPONENTS);
registerComponentMetadata(MESH_COMPONENTS);
registerComponentMetadata(PARAMS_COMPONENTS);
registerComponentMetadata(SCALAR_COMPONENTS);
registerComponentMetadata(SETS_COMPONENTS);
registerComponentMetadata(SURFACE_COMPONENTS);
registerComponentMetadata(TRANSFORM_COMPONENTS);
registerComponentMetadata(VECTOR_COMPONENTS);

const SLIDER_DEFAULTS = {
  value: 1,
  min: 0,
  max: 10,
  step: 0.01,
};

function getComponentMetadataByGuid(guid) {
  const normalized = normalizeGuid(guid);
  if (!normalized) return undefined;
  return COMPONENT_METADATA.get(normalized);
}

function toNumber(value) {
  if (value === null || value === undefined) return undefined;
  const normalized = String(value).replace(',', '.').trim();
  if (!normalized) return undefined;
  const numeric = Number(normalized);
  return Number.isNaN(numeric) ? undefined : numeric;
}

function getDirectChildElements(parent, tagName) {
  if (!parent) return [];
  const wanted = tagName?.toLowerCase?.();
  return Array.from(parent.children ?? []).filter((child) => child.tagName?.toLowerCase?.() === wanted);
}

function getDirectChildChunks(parent, name) {
  if (!parent) return [];
  const normalizedName = name?.toLowerCase?.();
  const results = [];
  const seen = new Set();

  const shouldInclude = (chunk) => {
    if (!chunk) return false;
    if (seen.has(chunk)) return false;
    if (!normalizedName) {
      seen.add(chunk);
      return true;
    }
    const childName = chunk.getAttribute?.('name');
    if (!childName || childName.toLowerCase() !== normalizedName) {
      return false;
    }
    seen.add(chunk);
    return true;
  };

  const directChunks = getDirectChildElements(parent, 'chunk');
  for (const chunk of directChunks) {
    if (shouldInclude(chunk)) {
      results.push(chunk);
    }
  }

  const chunkContainers = getDirectChildElements(parent, 'chunks');
  for (const container of chunkContainers) {
    const nestedChunks = getDirectChildElements(container, 'chunk');
    for (const chunk of nestedChunks) {
      if (shouldInclude(chunk)) {
        results.push(chunk);
      }
    }
  }

  return results;
}

function getItemsElement(chunk) {
  if (!chunk) return null;
  return getDirectChildElements(chunk, 'items')[0] ?? null;
}

function readItem(itemsElement, itemName) {
  if (!itemsElement) return null;
  const targetName = itemName?.toLowerCase?.();
  const items = Array.from(itemsElement.children ?? []);
  for (const item of items) {
    if (item.tagName?.toLowerCase?.() !== 'item') continue;
    const nameAttr = item.getAttribute('name');
    if (!nameAttr || nameAttr.toLowerCase() !== targetName) continue;
    const text = item.textContent?.trim();
    if (text !== undefined) {
      return text;
    }
  }
  return null;
}

function readItems(itemsElement, itemName) {
  if (!itemsElement) return [];
  const targetName = itemName?.toLowerCase?.();
  const result = [];
  const items = Array.from(itemsElement.children ?? []);
  for (const item of items) {
    if (item.tagName?.toLowerCase?.() !== 'item') continue;
    const nameAttr = item.getAttribute('name');
    if (!nameAttr || nameAttr.toLowerCase() !== targetName) continue;
    const text = item.textContent?.trim();
    if (text) {
      result.push(text);
    }
  }
  return result;
}

function parsePersistentValue(paramChunk) {
  if (!paramChunk) return undefined;
  const item = paramChunk.querySelector('chunk[name="PersistentData"] chunk[name="Item"] > items > item');
  if (!item) return undefined;
  const typeName = item.getAttribute('type_name')?.toLowerCase?.() ?? '';
  const rawText = item.textContent?.trim();
  if (!rawText) return undefined;
  if (typeName.startsWith('gh_double') || typeName.startsWith('gh_single') || typeName.startsWith('gh_int')) {
    const numeric = toNumber(rawText);
    if (numeric !== undefined) return numeric;
  }
  if (typeName === 'gh_bool') {
    return rawText.toLowerCase() === 'true';
  }
  return rawText;
}

function parseParamChunk(paramChunk) {
  const info = {
    index: Number(paramChunk.getAttribute('index')),
    name: null,
    nickName: null,
    instanceGuid: null,
    description: null,
    sources: [],
    defaultValue: undefined,
  };

  const items = getItemsElement(paramChunk);
  if (items) {
    info.instanceGuid = readItem(items, 'InstanceGuid');
    info.name = readItem(items, 'Name');
    info.nickName = readItem(items, 'NickName');
    info.description = readItem(items, 'Description');
    info.sources = readItems(items, 'Source');
  }

  info.defaultValue = parsePersistentValue(paramChunk);
  return info;
}

function collectParamChunks(containerChunk, type) {
  if (!containerChunk || !type) return [];

  const results = [];
  const seen = new Set();
  const addChunk = (chunk) => {
    if (!chunk || seen.has(chunk)) return;
    seen.add(chunk);
    results.push(chunk);
  };

  const normalizedType = String(type).toLowerCase();
  const legacyName = `param_${normalizedType}`;
  getDirectChildChunks(containerChunk, legacyName).forEach(addChunk);

  const parameterDataChunks = getDirectChildChunks(containerChunk, 'parameterdata');
  if (parameterDataChunks.length) {
    const altName = `${normalizedType}param`;
    parameterDataChunks.forEach((parameterDataChunk) => {
      getDirectChildChunks(parameterDataChunk, altName).forEach(addChunk);
    });
  }

  return results;
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

  const chunkItems = getItemsElement(objectChunk);
  const containerChunk = objectChunk?.querySelector?.('chunk[name="Container"]');
  const containerItems = getItemsElement(containerChunk);

  const labelCandidates = [
    readItem(containerItems, 'NickName'),
    readItem(chunkItems, 'NickName'),
    getFirstText(objectChunk, [
      'string[name="NickName"]',
      'string[name="Name"]',
      'chunk[name="Definition"] > string[name="NickName"]',
      'chunk[name="Definition"] > string[name="Name"]',
    ]),
  ];

  for (const candidate of labelCandidates) {
    if (candidate === null || candidate === undefined) continue;
    const label = String(candidate).trim();
    if (!label) continue;
    meta.label = label;
    meta.nickName = label;
    break;
  }

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
  const chunkItems = getItemsElement(chunk);
  const containerChunk = chunk.querySelector('chunk[name="Container"]');
  const containerItems = getItemsElement(containerChunk);

  let hidden = false;
  const hiddenSource =
    readItem(containerItems, 'Hidden') ??
    readItem(chunkItems, 'Hidden');
  if (hiddenSource !== null && hiddenSource !== undefined) {
    const normalizedHidden = String(hiddenSource).trim().toLowerCase();
    hidden = normalizedHidden === 'true' || normalizedHidden === '1' || normalizedHidden === 'yes';
  }

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

  if (!instanceGuid && chunkItems) {
    instanceGuid =
      readItem(chunkItems, 'InstanceGuid') ??
      readItem(chunkItems, 'InstanceID') ??
      readItem(chunkItems, 'ID') ??
      instanceGuid;
  }

  if (!instanceGuid && containerItems) {
    instanceGuid =
      readItem(containerItems, 'InstanceGuid') ??
      readItem(containerItems, 'InstanceID') ??
      readItem(containerItems, 'ID') ??
      instanceGuid;
  }

  if (!componentGuid && chunkItems) {
    componentGuid = readItem(chunkItems, 'GUID');
  }

  const normalizedComponentGuid = normalizeGuid(componentGuid);
  const normalizedInstanceGuid = normalizeGuid(instanceGuid);

  let name =
    getFirstText(chunk, [
      'string[name="NickName"]',
      'string[name="Name"]',
      'string[name="UserString"]',
      'chunk[name="Definition"] > string[name="Name"]',
      'chunk[name="Definition"] > string[name="NickName"]',
    ]) ||
    chunk.getAttribute('name') ||
    'Onbekende node';

  if (name === 'Object' || name === 'Onbekende node') {
    if (chunkItems) {
      const itemName = readItem(chunkItems, 'Name');
      const itemNick = readItem(chunkItems, 'NickName');
      name = itemNick || itemName || name;
    }
    if (name === 'Object' || name === 'Onbekende node') {
      if (containerItems) {
        const containerNick = readItem(containerItems, 'NickName');
        const containerName = readItem(containerItems, 'Name');
        name = containerNick || containerName || name;
      }
    }
  }

  return {
    id: normalizedInstanceGuid ?? `node-${index + 1}`,
    guid: normalizedComponentGuid,
    name,
    chunk,
    instanceGuid: normalizedInstanceGuid,
    hidden,
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

function pickPinName(info, fallbackPrefix, fallbackIndex) {
  if (!info) return `${fallbackPrefix}${fallbackIndex}`;
  if (info.nickName) return info.nickName;
  if (info.name) return info.name;
  if (info.description) return info.description;
  if (info.index !== undefined && info.index !== null && Number.isFinite(info.index)) {
    return `${fallbackPrefix}${info.index}`;
  }
  return `${fallbackPrefix}${fallbackIndex}`;
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
    hidden: Boolean(nodeInfo.hidden),
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
  const outputLookup = new Map();
  const pendingConnections = [];

  objectChunks.forEach((chunk, index) => {
    const info = describeObjectChunk(chunk, index);
    const descriptor = buildNodeDescriptor(info);
    if (descriptor.guid) {
      descriptor.guid = normalizeGuid(descriptor.guid);
    }

    const componentMeta = getComponentMetadataByGuid(descriptor.guid);
    if (componentMeta) {
      descriptor.meta = { ...descriptor.meta, component: componentMeta };
    }

    const containerChunk = chunk.querySelector('chunk[name="Container"]');
    const nodeId = descriptor.id;
    const containerItems = getItemsElement(containerChunk);

    if (containerItems) {
      const userText = readItem(containerItems, 'UserText');
      if (userText !== null && userText !== undefined) {
        descriptor.meta = { ...descriptor.meta, userText, value: userText };
      }
    }

    const outputChunks = collectParamChunks(containerChunk, 'output');
    if (outputChunks.length) {
      outputChunks.forEach((outputChunk, outputIndex) => {
        const paramInfo = parseParamChunk(outputChunk);
        const pinName = pickPinName(paramInfo, 'out', outputIndex);
        if (descriptor.outputs[pinName] === undefined) {
          descriptor.outputs[pinName] = null;
        }

        const normalizedParamGuid = normalizeGuid(paramInfo.instanceGuid);
        if (normalizedParamGuid) {
          outputLookup.set(normalizedParamGuid, { node: nodeId, pin: pinName });
        }
        if (outputIndex === 0) {
          outputLookup.set(nodeId, { node: nodeId, pin: pinName });
        }
      });
    }

    const inputChunks = collectParamChunks(containerChunk, 'input');
    if (inputChunks.length) {
      inputChunks.forEach((inputChunk, inputIndex) => {
        const paramInfo = parseParamChunk(inputChunk);
        const pinName = pickPinName(paramInfo, 'in', inputIndex);

        if (paramInfo.defaultValue !== undefined && descriptor.inputs[pinName] === undefined) {
          descriptor.inputs[pinName] = paramInfo.defaultValue;
        }

        if (paramInfo.sources?.length) {
          const normalizedSources = paramInfo.sources.map((source) => normalizeGuid(source)).filter(Boolean);
          if (normalizedSources.length) {
            pendingConnections.push({
              targetNode: nodeId,
              targetPin: pinName,
              sources: normalizedSources,
            });
          }
        }
      });
    }

    if (!outputChunks.length) {
      if (detectSliders(info)) {
        descriptor.outputs.value = null;
        const mapping = { node: nodeId, pin: 'value' };
        outputLookup.set(nodeId, mapping);
        if (info.instanceGuid) {
          outputLookup.set(info.instanceGuid, mapping);
        }
      } else if (PARAMETER_LIKE_GUIDS.has(descriptor.guid)) {
        descriptor.outputs.value = null;
        const mapping = { node: nodeId, pin: 'value' };
        outputLookup.set(nodeId, mapping);
        if (info.instanceGuid) {
          outputLookup.set(info.instanceGuid, mapping);
        }
      }
    }

    nodes.push(descriptor);

    const guidKey = descriptor.guid ?? '';
    const nameKey = descriptor.name?.toLowerCase?.() ?? '';
    const isKnown =
      detectSliders(info) ||
      (guidKey && KNOWN_COMPONENT_GUIDS.has(guidKey)) ||
      (nameKey && KNOWN_COMPONENT_NAMES.has(nameKey));
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
  const unresolved = [];

  pendingConnections.forEach((connection) => {
    connection.sources.forEach((sourceGuid) => {
      const mapping = outputLookup.get(sourceGuid);
      if (!mapping) {
        unresolved.push({
          sourceGuid,
          targetNode: connection.targetNode,
          targetPin: connection.targetPin,
        });
        return;
      }
      wires.push({
        from: { node: mapping.node, pin: mapping.pin },
        to: { node: connection.targetNode, pin: connection.targetPin },
      });
    });
  });

  if (unresolved.length) {
    const preview = unresolved.slice(0, 10);
    const remaining = unresolved.length - preview.length;
    if (remaining > 0) {
      console.warn('parseGHX: Kon niet alle verbindingen herleiden', preview, `(+${remaining} extra)`);
    } else {
      console.warn('parseGHX: Kon niet alle verbindingen herleiden', preview);
    }
  }

  return { nodes, wires };
}

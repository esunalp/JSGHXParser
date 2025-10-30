const PANEL_GUIDS = [
  '{59e0b89a-e487-49f8-bab8-b5bab16be14c}',
  'panel',
];

const RELAY_GUIDS = [
  '{b6236720-8d88-4289-93c3-ac4c99f9b97b}',
  'relay',
  'params:relay',
];

const GEOMETRY_COMPONENTS = [
  {
    guid: '04d3eace-deaa-475e-9e69-8f804d687998',
    name: 'Circular Arc',
    description: 'Contains a collection of circular arcs',
    aliases: ['Arc', 'A'],
  },
  {
    guid: '16ef3e75-e315-4899-b531-d3166b42dac9',
    name: 'Vector',
    description: 'Contains a collection of three-dimensional vectors',
    aliases: ['Vec', 'Vector', 'V'],
  },
  {
    guid: '1e936df3-0eea-4246-8549-514cb8862b7a',
    name: 'Mesh',
    description: 'Contains a collection of polygon meshes',
    aliases: ['Mesh', 'M'],
  },
  {
    guid: '28f40e48-e739-4211-91bd-f4aefa5965f8',
    name: 'Transform',
    description: 'Contains a collection of three-dimensional transformations',
    aliases: ['Transform', 'XForm', 'X'],
  },
  {
    guid: '3175e3eb-1ae0-4d0b-9395-53fd3e8f8a28',
    name: 'Field',
    description: 'Contains a collection of vector fields',
    aliases: ['Field', 'F'],
  },
  {
    guid: '4f8984c4-7c7a-4d69-b0a2-183cbb330d20',
    name: 'Plane',
    description: 'Contains a collection of three-dimensional axis-systems',
    aliases: ['Plane', 'Pl', 'P'],
  },
  {
    guid: '6db039c4-cad1-4549-bd45-e31cb0f71692',
    name: 'Twisted Box',
    description: 'Contains a collection of twisted boxes',
    aliases: ['TwistedBox', 'Twist', 'Box', 'B'],
  },
  {
    guid: '8529dbdf-9b6f-42e9-8e1f-c7a2bde56a70',
    name: 'Line',
    description: 'Contains a collection of line segments',
    aliases: ['Line', 'Ln', 'L'],
  },
  {
    guid: '87391af3-35fe-4a40-b001-2bd4547ccd45',
    name: 'Location',
    description: 'Contains a collection of latitude-longitude coordinates',
    aliases: ['Location', 'Loc', 'L'],
  },
  {
    guid: '89cd1a12-0007-4581-99ba-66578665e610',
    name: 'SubD',
    description: 'Contains a collection of SubDs',
    aliases: ['SubD', 'Subd', 'S'],
  },
  {
    guid: '919e146f-30ae-4aae-be34-4d72f555e7da',
    name: 'Brep',
    description: 'Contains a collection of Breps (Boundary REPresentations)',
    aliases: ['Brep', 'BRep', 'B'],
  },
  {
    guid: 'a80395af-f134-4d6a-9b89-15edf3161619',
    name: 'Atom',
    description: 'Contains a collection of atoms',
    aliases: ['Atom', 'A'],
  },
  {
    guid: 'bf9c670-5462-4cd8-acb3-f1ab0256dbf3',
    name: 'Rectangle',
    description: 'Contains a collection of rectangles',
    aliases: ['Rectangle', 'Rect', 'R'],
  },
  {
    guid: 'ac2bc2cb-70fb-4dd5-9c78-7e1ea97fe278',
    name: 'Geometry',
    description: 'Contains a collection of generic geometry',
    aliases: ['Geometry', 'Geom', 'G'],
  },
  {
    guid: 'b0851fc0-ab55-47d8-bdda-cc6306a40176',
    name: 'Group',
    description: 'Contains a collection of geometric groups',
    aliases: ['Group', 'Grp', 'G'],
  },
  {
    guid: 'b341e2e5-c4b3-49a3-b3a4-b4e6e2054516',
    name: 'Geometry Pipeline',
    description: 'Defines a geometry pipeline from Rhino to Grasshopper',
    aliases: ['Geometry', 'Geom', 'Pipeline', 'Pipe', 'G'],
  },
  {
    guid: 'c3407fda-b505-4686-9165-38fe7a9274cf',
    name: 'Mesher Settings',
    description: 'Represents a list of Meshing settings.',
    aliases: ['Mesher', 'Settings', 'Mesh', 'M'],
  },
  {
    guid: 'c9482db6-bea9-448d-98ff-fed6d69a8efc',
    name: 'Box',
    description: 'Contains a collection of boxes',
    aliases: ['Box', 'B'],
  },
  {
    guid: 'd1028c72-ff86-4057-9eb0-36c687a4d98c',
    name: 'Circle',
    description: 'Contains a collection of circles',
    aliases: ['Circle', 'Circ', 'C'],
  },
  {
    guid: 'd5967b9f-e8ee-436b-a8ad-29fdcecf32d5',
    name: 'Curve',
    description: 'Contains a collection of generic curves',
    aliases: ['Curve', 'Crv', 'C'],
  },
  {
    guid: 'deaf8653-5528-4286-807c-3de8b8dad781',
    name: 'Surface',
    description: 'Contains a collection of generic surfaces',
    aliases: ['Surface', 'Srf', 'S'],
  },
  {
    guid: 'e02b3da5-543a-46ac-a867-0ba6b0a524de',
    name: 'Mesh Face',
    description: 'Contains a collection of triangle or quad mesh faces',
    aliases: ['MeshFace', 'Face', 'F', 'M'],
  },
  {
    guid: 'f91778ca-2700-42fc-8ee6-74049a2292b5',
    name: 'Geometry Cache',
    description: 'Bake or Load geometry to and from the Rhino document',
    aliases: ['Geometry', 'Geom', 'Cache', 'G', 'C'],
  },
  {
    guid: 'fa20fe95-5775-417b-92ff-b77c13cbf40c',
    name: 'Mesh Point',
    description: 'Contains a collection of mesh points',
    aliases: ['MeshPoint', 'Point', 'Pt', 'P', 'M'],
  },
  {
    guid: 'fbac3e32-f100-4292-8692-77240a42fd1a',
    name: 'Point',
    description: 'Contains a collection of three-dimensional points',
    aliases: ['Point', 'Pt', 'P'],
  },
];

const DEFAULT_PIN_ALIAS_ENTRIES = [
  ['Value', 'value'],
  ['value', 'value'],
  ['Data', 'value'],
  ['data', 'value'],
  ['Geometry', 'value'],
  ['geometry', 'value'],
  ['Geom', 'value'],
  ['geom', 'value'],
  ['G', 'value'],
  ['g', 'value'],
  ['Input', 'value'],
  ['input', 'value'],
  ['In', 'value'],
  ['in', 'value'],
  ['Out', 'value'],
  ['out', 'value'],
  ['Result', 'value'],
  ['result', 'value'],
  ['D', 'value'],
  ['d', 'value'],
  ['values', 'values'],
  ['Values', 'values'],
];

function createGuidVariants(guid) {
  if (!guid && guid !== 0) {
    return [];
  }
  const text = String(guid).trim();
  if (!text) {
    return [];
  }
  const bare = text.replace(/^\{+/, '').replace(/\}+$/, '');
  if (!bare) {
    return [];
  }
  const variants = new Set([bare, `{${bare}}`]);
  if (bare.length === 35) {
    const padded = `0${bare}`;
    variants.add(padded);
    variants.add(`{${padded}}`);
  }
  return Array.from(variants);
}

function slugify(value) {
  if (!value && value !== 0) {
    return '';
  }
  return String(value)
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-');
}

function addAlias(target, alias, internal = 'value') {
  if (!alias && alias !== 0) {
    return;
  }
  const text = String(alias).trim();
  if (!text || target.has(text)) {
    return;
  }
  target.set(text, internal);
}

function createAliasMap(component) {
  const aliases = new Map(DEFAULT_PIN_ALIAS_ENTRIES);
  const candidateNames = [component.name, ...(component.aliases ?? [])];
  for (const candidate of candidateNames) {
    if (!candidate && candidate !== 0) {
      continue;
    }
    const text = String(candidate).trim();
    if (!text) {
      continue;
    }
    addAlias(aliases, text, 'value');
    addAlias(aliases, text.toLowerCase(), 'value');
    addAlias(aliases, text.replace(/\s+/g, ''), 'value');
  }
  return aliases;
}

function createPinMap(component) {
  const aliasMap = createAliasMap(component);
  const entries = {};
  for (const [alias, internal] of aliasMap.entries()) {
    entries[String(alias)] = internal;
  }
  return {
    inputs: { ...entries },
    outputs: { ...entries },
  };
}

function createComponentKeys(component) {
  const keys = new Set();
  for (const guid of createGuidVariants(component.guid)) {
    keys.add(guid);
  }
  if (component.name) {
    keys.add(component.name);
    keys.add(component.name.toLowerCase());
    keys.add(component.name.replace(/\s+/g, ''));
  }
  const slug = slugify(component.name);
  if (slug) {
    keys.add(`params:geometry:${slug}`);
  }
  return Array.from(keys);
}

function evaluateGeometryParams(inputs) {
  const incoming = collectIncomingValues(inputs);
  if (!incoming.length) {
    return {};
  }
  const value = incoming.length === 1 ? incoming[0] : incoming;
  return normalizeRelayValue(value);
}

export function registerParamsGeometryComponents({ register }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register params geometry components.');
  }

  for (const component of GEOMETRY_COMPONENTS) {
    const keys = createComponentKeys(component);
    const pinMap = createPinMap(component);
    register(keys, {
      type: 'params',
      description: component.description,
      pinMap,
      eval: ({ inputs }) => evaluateGeometryParams(inputs),
    });
  }
}

function isTypedArray(value) {
  return ArrayBuffer.isView(value) && !(value instanceof DataView);
}

function isPlainObject(value) {
  if (!value || typeof value !== 'object') {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function isDefined(value) {
  return value !== undefined && value !== null;
}

function panelValueToText(value, visited = new Set()) {
  if (value === undefined || value === null) {
    return '';
  }

  const type = typeof value;
  if (type === 'string') {
    return value;
  }

  if (type === 'number' || type === 'boolean' || type === 'bigint') {
    return String(value);
  }

  if (value?.isVector3) {
    const { x, y, z } = value;
    return `${x}, ${y}, ${z}`;
  }

  if (visited.has(value)) {
    return '[Circular]';
  }

  visited.add(value);

  if (Array.isArray(value)) {
    return value.map((entry) => panelValueToText(entry, visited)).join('\n');
  }

  if (type === 'object') {
    if (Object.prototype.hasOwnProperty.call(value, 'text')) {
      return panelValueToText(value.text, visited);
    }
    if (Object.prototype.hasOwnProperty.call(value, 'lines')) {
      const lines = value.lines;
      if (Array.isArray(lines)) {
        return lines.map((entry) => panelValueToText(entry, visited)).join('\n');
      }
      return panelValueToText(lines, visited);
    }
    if (Object.prototype.hasOwnProperty.call(value, 'value')) {
      return panelValueToText(value.value, visited);
    }
    if (Object.prototype.hasOwnProperty.call(value, 'values')) {
      return panelValueToText(value.values, visited);
    }
    if (typeof value.toString === 'function' && value.toString !== Object.prototype.toString) {
      const stringValue = value.toString();
      if (typeof stringValue === 'string' && stringValue !== '[object Object]') {
        return stringValue;
      }
    }
    try {
      return JSON.stringify(value);
    } catch (_error) {
      return String(value);
    }
  }

  return String(value);
}

function panelValueToLines(value, text = null) {
  const content = text ?? panelValueToText(value);
  if (!content) {
    return [];
  }
  return content.replace(/\r\n/g, '\n').split('\n');
}

function normalizePanelValue(value, lines) {
  if (value === undefined) {
    if (!lines.length) {
      return '';
    }
    if (lines.length === 1) {
      return lines[0];
    }
    return lines;
  }
  return value;
}

function buildPanelPresentation(rawValue) {
  const text = panelValueToText(rawValue);
  const lines = panelValueToLines(rawValue, text);
  const value = normalizePanelValue(rawValue, lines);
  return { value, text, lines };
}

function normalizeRelayValue(value) {
  if (value === undefined) {
    return {};
  }

  if (Array.isArray(value)) {
    return { value, values: value };
  }

  if (isTypedArray(value)) {
    return { value };
  }

  if (isPlainObject(value)) {
    const payload = { ...value };
    const hasValueProperty = Object.prototype.hasOwnProperty.call(payload, 'value');
    const hasValuesProperty = Object.prototype.hasOwnProperty.call(payload, 'values');

    if (!hasValueProperty) {
      payload.value = value;
    } else if (payload.value === undefined && hasValuesProperty) {
      payload.value = payload.values;
    }

    if (!hasValuesProperty && Array.isArray(payload.value)) {
      payload.values = payload.value;
    }

    return payload;
  }

  return { value };
}

function collectIncomingValues(inputs) {
  if (!inputs || typeof inputs !== 'object') {
    return [];
  }
  const values = [];
  for (const value of Object.values(inputs)) {
    if (value === undefined) {
      continue;
    }
    values.push(value);
  }
  return values;
}

function pickInitialPanelValue(node) {
  if (!node) {
    return '';
  }
  const meta = node.meta ?? {};
  const metaKeys = ['value', 'userText', 'text', 'initialValue', 'defaultValue'];
  for (const key of metaKeys) {
    if (isDefined(meta[key])) {
      return meta[key];
    }
  }
  const inputCandidates = node.inputs ?? {};
  const inputKeys = ['value', 'Value', 'values', 'Values', 'text', 'Text', 'data', 'Data'];
  for (const key of inputKeys) {
    if (isDefined(inputCandidates[key])) {
      return inputCandidates[key];
    }
  }
  return '';
}

function createPanelState(node) {
  const initialValue = pickInitialPanelValue(node);
  const presentation = buildPanelPresentation(initialValue);
  return {
    ...presentation,
    source: 'meta',
  };
}

function evaluatePanel({ node, inputs, state }) {
  const incoming = collectIncomingValues(inputs);
  if (incoming.length > 0) {
    const value = incoming.length === 1 ? incoming[0] : incoming;
    const presentation = buildPanelPresentation(value);
    if (state) {
      state.value = presentation.value;
      state.text = presentation.text;
      state.lines = presentation.lines;
      state.source = 'input';
    }
    return presentation;
  }

  if (!state) {
    return createPanelState(node);
  }

  if (!isDefined(state.value)) {
    Object.assign(state, createPanelState(node));
  }

  if (!isDefined(state.text) || !Array.isArray(state.lines)) {
    const normalized = buildPanelPresentation(state.value);
    if (!isDefined(state.text)) {
      state.text = normalized.text;
    }
    if (!Array.isArray(state.lines)) {
      state.lines = normalized.lines;
    }
  }

  return {
    value: state.value,
    text: state.text,
    lines: state.lines,
  };
}

export function registerParamsInputComponents({ register }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register params input components.');
  }

  register(PANEL_GUIDS, {
    type: 'panel',
    pinMap: {
      inputs: {
        In: 'data',
        in: 'data',
        Data: 'data',
        data: 'data',
        Value: 'data',
        value: 'data',
        Text: 'data',
        text: 'data',
      },
      outputs: {
        Value: 'value',
        value: 'value',
        Data: 'value',
        data: 'value',
        Out: 'value',
        out: 'value',
        Text: 'text',
        text: 'text',
        Lines: 'lines',
        lines: 'lines',
      },
    },
    createState: createPanelState,
    eval: evaluatePanel,
    describe: (state) => state?.text ?? '',
  });
}

export function registerParamsUtilComponents({ register }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register params util components.');
  }

  register(RELAY_GUIDS, {
    type: 'params',
    pinMap: {
      inputs: {
        D: 'value',
        d: 'value',
        In: 'value',
        in: 'value',
        Data: 'value',
        data: 'value',
        Value: 'value',
        value: 'value',
      },
      outputs: {
        D: 'value',
        d: 'value',
        Out: 'value',
        out: 'value',
        Data: 'value',
        data: 'value',
        Value: 'value',
        value: 'value',
      },
    },
    eval: ({ inputs }) => normalizeRelayValue(inputs?.value),
  });
}

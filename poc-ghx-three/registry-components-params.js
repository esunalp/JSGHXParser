const PANEL_GUIDS = [
  '{59e0b89a-e487-49f8-bab8-b5bab16be14c}',
  'panel',
];

const RELAY_GUIDS = [
  '{b6236720-8d88-4289-93c3-ac4c99f9b97b}',
  'relay',
  'params:relay',
];

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
    eval: ({ inputs }) => ({ value: inputs?.value }),
  });
}

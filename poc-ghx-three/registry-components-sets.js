const GUID_KEYS = (guids = []) => {
  const keys = new Set();
  for (const guid of guids) {
    if (!guid && guid !== 0) continue;
    const text = String(guid).trim();
    if (!text) continue;
    const bare = text.replace(/^\{+/, '').replace(/\}+$/, '');
    if (!bare) continue;
    keys.add(bare);
    keys.add(`{${bare}}`);
  }
  return Array.from(keys);
};

function ensureRegisterFunction(register) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register set list components.');
  }
}

function ensureToNumberFunction(toNumber) {
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register set list components.');
  }
}

function isIterable(value) {
  return value && typeof value === 'object' && typeof value[Symbol.iterator] === 'function';
}

function toList(input) {
  if (input === undefined || input === null) {
    return [];
  }
  if (Array.isArray(input)) {
    return input.slice();
  }
  if (input?.type === 'tree' && Array.isArray(input.branches)) {
    const values = [];
    for (const branch of input.branches) {
      if (!branch) continue;
      const branchValues = branch.values;
      if (Array.isArray(branchValues)) {
        values.push(...branchValues);
      } else if (branchValues !== undefined) {
        values.push(branchValues);
      }
    }
    return values;
  }
  if (typeof input === 'object') {
    if (Object.prototype.hasOwnProperty.call(input, 'values')) {
      return toList(input.values);
    }
    if (Object.prototype.hasOwnProperty.call(input, 'value')) {
      return toList(input.value);
    }
    if (isIterable(input) && typeof input !== 'string') {
      return Array.from(input);
    }
  }
  if (typeof input === 'string') {
    return [input];
  }
  return [input];
}

function toBoolean(value, fallback = false) {
  if (value === undefined || value === null) {
    return fallback;
  }
  if (Array.isArray(value)) {
    if (!value.length) {
      return fallback;
    }
    return toBoolean(value[0], fallback);
  }
  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase();
    if (!normalized) {
      return fallback;
    }
    if (['true', 'yes', '1', 'on'].includes(normalized)) {
      return true;
    }
    if (['false', 'no', '0', 'off'].includes(normalized)) {
      return false;
    }
    return fallback;
  }
  return Boolean(value);
}

function toInteger(value, fallback = 0, toNumber) {
  if (Array.isArray(value)) {
    if (!value.length) return fallback;
    return toInteger(value[0], fallback, toNumber);
  }
  if (typeof value === 'bigint') {
    return Number(value);
  }
  const numeric = toNumber ? toNumber(value, Number.NaN) : Number(value);
  if (!Number.isFinite(numeric)) {
    return fallback;
  }
  if (numeric < 0 && numeric > -1) {
    return 0;
  }
  return Math.trunc(numeric);
}

function wrapIndex(index, length) {
  if (length <= 0) {
    return 0;
  }
  let wrapped = index % length;
  if (wrapped < 0) {
    wrapped += length;
  }
  return wrapped;
}

function gatherIndexedInputs(inputs, prefixRegex) {
  const entries = [];
  for (const [key, value] of Object.entries(inputs || {})) {
    const normalized = String(key);
    const match = normalized.match(prefixRegex);
    if (!match) continue;
    const index = Number.parseInt(match[1], 10);
    if (!Number.isFinite(index)) continue;
    entries[index] = toList(value);
  }
  return entries.map((list) => (Array.isArray(list) ? list : []));
}

function createDataTree(groups) {
  return {
    type: 'tree',
    branches: groups.map((values, index) => ({
      path: [index],
      values: values ?? [],
    })),
  };
}

function mapStructure(value, mapper) {
  if (value?.type === 'tree' && Array.isArray(value.branches)) {
    return {
      type: 'tree',
      branches: value.branches.map((branch) => ({
        path: Array.isArray(branch?.path) ? branch.path.slice() : [],
        values: toList(branch?.values).map((entry, index) => mapper(entry, branch?.path, index)),
      })),
    };
  }
  if (Array.isArray(value)) {
    return value.map((entry, index) => mapper(entry, null, index));
  }
  return mapper(value, null, 0);
}

function isNullLike(value) {
  if (value === undefined || value === null) {
    return true;
  }
  if (Array.isArray(value) && !value.length) {
    return false;
  }
  if (typeof value === 'object') {
    if (value?.isNull === true) {
      return true;
    }
    if (Object.prototype.hasOwnProperty.call(value, 'value')) {
      return isNullLike(value.value);
    }
  }
  return false;
}

function isInvalidValue(value) {
  if (value === undefined || value === null) {
    return false;
  }
  if (typeof value === 'number') {
    return Number.isNaN(value) || !Number.isFinite(value);
  }
  if (value?.isValid === false) {
    return true;
  }
  return false;
}

function describeValueState(value, isNull, isInvalid) {
  if (isNull) return 'Null';
  if (isInvalid) {
    if (typeof value === 'number') {
      if (Number.isNaN(value)) return 'NaN';
      if (!Number.isFinite(value)) return 'Infinite';
    }
    return 'Invalid';
  }
  if (value === undefined) return 'Undefined';
  return 'Valid';
}

function toSortableEntry(value, index, toNumber) {
  const numeric = toNumber(value, Number.NaN);
  if (Number.isFinite(numeric)) {
    return { type: 'number', value: numeric, original: value, index };
  }
  if (value === undefined || value === null) {
    return { type: 'null', value: null, original: value, index };
  }
  const text = typeof value === 'string' ? value : String(value);
  return { type: 'string', value: text.toLowerCase(), original: value, index, text };
}

function compareSortable(a, b) {
  if (a.type === b.type) {
    if (a.type === 'number') {
      if (a.value < b.value) return -1;
      if (a.value > b.value) return 1;
    } else if (a.type === 'string') {
      if (a.value < b.value) return -1;
      if (a.value > b.value) return 1;
    }
  } else {
    const order = { number: 0, string: 1, null: 2 };
    const aOrder = order[a.type] ?? 99;
    const bOrder = order[b.type] ?? 99;
    if (aOrder !== bOrder) {
      return aOrder - bOrder;
    }
  }
  return a.index - b.index;
}

function resolveDomain(domainInput, listLength, toNumber) {
  if (domainInput === undefined || domainInput === null) {
    return { start: 0, end: listLength - 1 };
  }
  const resolveCandidate = (value, fallback) => toInteger(value, fallback, toNumber);
  if (Array.isArray(domainInput)) {
    if (!domainInput.length) {
      return { start: 0, end: listLength - 1 };
    }
    if (domainInput.length === 1) {
      const index = resolveCandidate(domainInput[0], 0);
      return { start: index, end: index };
    }
    const start = resolveCandidate(domainInput[0], 0);
    const end = resolveCandidate(domainInput[1], listLength - 1);
    return { start, end };
  }
  if (typeof domainInput === 'object') {
    const startCandidate = domainInput.start ?? domainInput.min ?? domainInput.t0 ?? domainInput.from ?? domainInput.a ?? domainInput[0];
    const endCandidate = domainInput.end ?? domainInput.max ?? domainInput.t1 ?? domainInput.to ?? domainInput.b ?? domainInput[1];
    const start = resolveCandidate(startCandidate, 0);
    const end = resolveCandidate(endCandidate, listLength - 1);
    return { start, end };
  }
  const single = resolveCandidate(domainInput, 0);
  return { start: 0, end: single };
}

function valuesEqual(a, b) {
  if (Object.is(a, b)) {
    return true;
  }
  if (a === null || b === null || a === undefined || b === undefined) {
    return a === b;
  }
  if (typeof a === 'number' && typeof b === 'number') {
    if (Number.isNaN(a) && Number.isNaN(b)) {
      return true;
    }
  }
  return a === b;
}

function levenshteinDistance(a, b) {
  if (a === b) {
    return 0;
  }
  const stringA = `${a}`;
  const stringB = `${b}`;
  const lengthA = stringA.length;
  const lengthB = stringB.length;
  if (lengthA === 0) {
    return lengthB;
  }
  if (lengthB === 0) {
    return lengthA;
  }

  let previous = new Array(lengthB + 1);
  let current = new Array(lengthB + 1);

  for (let index = 0; index <= lengthB; index += 1) {
    previous[index] = index;
  }

  for (let row = 1; row <= lengthA; row += 1) {
    current[0] = row;
    const charCodeA = stringA.charCodeAt(row - 1);
    for (let column = 1; column <= lengthB; column += 1) {
      const charCodeB = stringB.charCodeAt(column - 1);
      const substitutionCost = charCodeA === charCodeB ? 0 : 1;
      const deletion = previous[column] + 1;
      const insertion = current[column - 1] + 1;
      const substitution = previous[column - 1] + substitutionCost;
      current[column] = Math.min(deletion, insertion, substitution);
    }
    const swap = previous;
    previous = current;
    current = swap;
  }

  return previous[lengthB];
}

function findValueIndex(list, target) {
  if (!Array.isArray(list) || !list.length) {
    return -1;
  }
  for (let index = 0; index < list.length; index += 1) {
    if (valuesEqual(list[index], target)) {
      return index;
    }
  }
  return -1;
}

function includesValue(list, value) {
  return findValueIndex(list, value) !== -1;
}

function uniqueValues(list) {
  const result = [];
  if (!Array.isArray(list)) {
    return result;
  }
  for (const value of list) {
    if (!includesValue(result, value)) {
      result.push(value);
    }
  }
  return result;
}

function createUniqueSetWithMap(list) {
  const unique = [];
  const map = [];
  if (!Array.isArray(list)) {
    return { unique, map };
  }
  for (const value of list) {
    let index = findValueIndex(unique, value);
    if (index === -1) {
      index = unique.length;
      unique.push(value);
    }
    map.push(index);
  }
  return { unique, map };
}

export function registerSetsListComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const registerPickChoose = (guidA, guidB) => {
    register([
      ...GUID_KEYS([guidA, guidB]),
      "Pick'n'Choose",
      'picknchoose',
      'pick choose',
    ], {
      type: 'sets:list',
      pinMap: {
        inputs: {
          P: 'pattern',
          Pattern: 'pattern',
          pattern: 'pattern',
          0: 'stream0',
          'Stream 0': 'stream0',
          1: 'stream1',
          'Stream 1': 'stream1',
          2: 'stream2',
          'Stream 2': 'stream2',
          3: 'stream3',
          'Stream 3': 'stream3',
        },
        outputs: {
          R: 'result',
          Result: 'result',
          W: 'result',
        },
      },
      eval: ({ inputs }) => {
        const streams = gatherIndexedInputs(inputs, /^(?:stream\s*)?(\d+)$/i);
        if (!streams.length) {
          return { result: [] };
        }
        let pattern = toList(inputs.pattern);
        if (!pattern.length) {
          const defaultPattern = [];
          const maxLength = Math.max(...streams.map((stream) => stream.length));
          for (let itemIndex = 0; itemIndex < maxLength; itemIndex += 1) {
            for (let streamIndex = 0; streamIndex < streams.length; streamIndex += 1) {
              if (itemIndex < streams[streamIndex].length) {
                defaultPattern.push(streamIndex);
              }
            }
          }
          pattern = defaultPattern;
        }
        const positions = streams.map(() => 0);
        const result = [];
        for (const entry of pattern) {
          const index = toInteger(entry, NaN, toNumber);
          if (!Number.isFinite(index)) {
            result.push(null);
            continue;
          }
          const stream = streams[index];
          if (!stream || !stream.length) {
            result.push(null);
            continue;
          }
          const pointer = positions[index] ?? 0;
          if (pointer >= stream.length) {
            result.push(null);
            continue;
          }
          result.push(stream[pointer]);
          positions[index] = pointer + 1;
        }
        return { result };
      },
    });
  };

  registerPickChoose('03b801eb-87cd-476a-a591-257fe5d5bf0f', '4356ef8f-0ca1-4632-9c39-9e6dcd2b9496');

  const registerWeave = (guidA, guidB) => {
    register([
      ...GUID_KEYS([guidA, guidB]),
      'Weave',
    ], {
      type: 'sets:list',
      pinMap: {
        inputs: {
          P: 'pattern',
          Pattern: 'pattern',
          pattern: 'pattern',
          0: 'stream0',
          'Stream 0': 'stream0',
          1: 'stream1',
          'Stream 1': 'stream1',
          2: 'stream2',
          'Stream 2': 'stream2',
          3: 'stream3',
          'Stream 3': 'stream3',
        },
        outputs: {
          W: 'result',
          Weave: 'result',
          Result: 'result',
        },
      },
      eval: ({ inputs }) => {
        const streams = gatherIndexedInputs(inputs, /^(?:stream\s*)?(\d+)$/i);
        if (!streams.length) {
          return { result: [] };
        }
        let pattern = toList(inputs.pattern);
        if (!pattern.length) {
          const maxLength = Math.max(...streams.map((stream) => stream.length));
          const defaultPattern = [];
          for (let itemIndex = 0; itemIndex < maxLength; itemIndex += 1) {
            for (let streamIndex = 0; streamIndex < streams.length; streamIndex += 1) {
              if (itemIndex < streams[streamIndex].length) {
                defaultPattern.push(streamIndex);
              }
            }
          }
          pattern = defaultPattern;
        }
        const positions = streams.map(() => 0);
        const result = [];
        for (const entry of pattern) {
          const index = toInteger(entry, NaN, toNumber);
          if (!Number.isFinite(index)) {
            result.push(null);
            continue;
          }
          const stream = streams[index];
          if (!stream || !stream.length) {
            result.push(null);
            continue;
          }
          const pointer = positions[index] ?? 0;
          if (pointer >= stream.length) {
            result.push(null);
            continue;
          }
          result.push(stream[pointer]);
          positions[index] = pointer + 1;
        }
        return { result };
      },
    });
  };

  registerWeave('160c1df2-e2e8-48e5-b538-f2d6981007e3', '50faccbd-9c92-4175-a5fa-d65e36013db6');

  register([
    ...GUID_KEYS(['1817fd29-20ae-4503-b542-f0fb651e67d7']),
    'List Length',
    'list length',
    'list:length',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list' },
      outputs: { L: 'length', Length: 'length' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      return { length: list.length };
    },
  });

  const registerListItem = (guid, outputName) => {
    register([
      ...GUID_KEYS([guid]),
      'List Item',
      'list item',
      'list:item',
    ], {
      type: 'sets:list',
      pinMap: {
        inputs: {
          L: 'list',
          List: 'list',
          list: 'list',
          i: 'index',
          I: 'index',
          Index: 'index',
          W: 'wrap',
          Wrap: 'wrap',
        },
        outputs: {
          E: outputName,
          e: outputName,
          Item: outputName,
          i: outputName,
        },
      },
      eval: ({ inputs }) => {
        const list = toList(inputs.list);
        const wrap = toBoolean(inputs.wrap, false);
        const indexValues = toList(inputs.index);
        const resolvedIndices = indexValues.length ? indexValues.map((value) => toInteger(value, 0, toNumber)) : [0];
        if (!list.length) {
          const fallback = resolvedIndices.length <= 1 ? null : resolvedIndices.map(() => null);
          return { [outputName]: fallback };
        }
        const results = resolvedIndices.map((candidate) => {
          let index = candidate;
          if (wrap) {
            index = wrapIndex(index, list.length);
          }
          if (index < 0 || index >= list.length) {
            return null;
          }
          return list[index];
        });
        return { [outputName]: results.length <= 1 ? results[0] ?? null : results };
      },
    });
  };

  registerListItem('285ddd8a-5398-4a3e-b3c2-361025711a51', 'item');
  registerListItem('59daf374-bc21-4a5e-8282-5504fb7ae9ae', 'item');
  registerListItem('6e2ba21a-2252-42f4-8d3f-f5e0f49cc4ef', 'item');

  const registerSortList = (guid, extraOutputs = []) => {
    register([
      ...GUID_KEYS([guid]),
      'Sort List',
      'sort list',
      'list:sort',
    ], {
      type: 'sets:list',
      pinMap: {
        inputs: {
          K: 'keys',
          Keys: 'keys',
          keys: 'keys',
          A: 'valuesA',
          B: 'valuesB',
          C: 'valuesC',
          a: 'valuesA',
          b: 'valuesB',
          c: 'valuesC',
        },
        outputs: {
          L: 'sortedKeys',
          List: 'sortedKeys',
          K: 'sortedKeys',
          keys: 'sortedKeys',
          A: 'sortedValuesA',
          B: 'sortedValuesB',
          C: 'sortedValuesC',
        },
      },
      eval: ({ inputs }) => {
        const keys = toList(inputs.keys);
        const decorated = keys.map((value, index) => ({
          index,
          key: value,
          sortable: toSortableEntry(value, index, toNumber),
        }));
        decorated.sort((a, b) => compareSortable(a.sortable, b.sortable));
        const sortedKeys = decorated.map((entry) => entry.key);
        const result = { sortedKeys };
        if (inputs.valuesA !== undefined) {
          const values = toList(inputs.valuesA);
          result.sortedValuesA = decorated.map((entry) => values[entry.index]);
        }
        if (inputs.valuesB !== undefined) {
          const values = toList(inputs.valuesB);
          result.sortedValuesB = decorated.map((entry) => values[entry.index]);
        }
        if (inputs.valuesC !== undefined) {
          const values = toList(inputs.valuesC);
          result.sortedValuesC = decorated.map((entry) => values[entry.index]);
        }
        for (const name of extraOutputs) {
          if (inputs[name] !== undefined) {
            const values = toList(inputs[name]);
            const keyName = `sorted${name[0].toUpperCase()}${name.slice(1)}`;
            result[keyName] = decorated.map((entry) => values[entry.index]);
          }
        }
        return result;
      },
    });
  };

  registerSortList('2b2628ea-3f43-4ce9-8435-9a045d54b5c6');
  registerSortList('6f93d366-919f-4dda-a35e-ba03dd62799b');
  registerSortList('cacb2c64-61b5-46db-825d-c61d5d09cc08');

  register([
    ...GUID_KEYS(['3249222f-f536-467a-89f4-f0353fba455a']),
    'Sift Pattern',
    'sift pattern',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', P: 'pattern', Pattern: 'pattern' },
      outputs: { 0: 'trueValues', 1: 'falseValues' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const pattern = toList(inputs.pattern);
      const positives = [];
      const negatives = [];
      if (!pattern.length) {
        return { trueValues: positives, falseValues: list.slice() };
      }
      for (let index = 0; index < list.length; index += 1) {
        const patternValue = pattern[index % pattern.length];
        const target = toBoolean(patternValue, false) ? positives : negatives;
        target.push(list[index]);
      }
      return { trueValues: positives, falseValues: negatives };
    },
  });

  register([
    ...GUID_KEYS(['36947590-f0cb-4807-a8f9-9c90c9b20621']),
    'Cross Reference',
    'cross reference',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { A: 'listA', B: 'listB' },
      outputs: { A: 'resultA', B: 'resultB' },
    },
    eval: ({ inputs }) => {
      const listA = toList(inputs.listA);
      const listB = toList(inputs.listB);
      const resultA = [];
      const resultB = [];
      for (const valueA of listA) {
        for (const valueB of listB) {
          resultA.push(valueA);
          resultB.push(valueB);
        }
      }
      return { resultA, resultB };
    },
  });

  register([
    ...GUID_KEYS(['4fdfe351-6c07-47ce-9fb9-be027fb62186']),
    'Shift List',
    'shift list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', S: 'shift', Shift: 'shift', W: 'wrap', Wrap: 'wrap' },
      outputs: { L: 'result', List: 'result' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      if (!list.length) {
        return { result: [] };
      }
      const shiftAmount = toInteger(inputs.shift, 0, toNumber);
      const wrap = toBoolean(inputs.wrap, false);
      if (!wrap) {
        if (shiftAmount === 0) {
          return { result: list.slice() };
        }
        if (shiftAmount > 0) {
          const padding = Array(Math.min(shiftAmount, list.length)).fill(null);
          const remaining = list.slice(0, Math.max(0, list.length - shiftAmount));
          return { result: padding.concat(remaining) };
        }
        const absShift = Math.abs(shiftAmount);
        const trailing = list.slice(Math.min(absShift, list.length));
        const padding = Array(Math.min(absShift, list.length)).fill(null);
        return { result: trailing.concat(padding) };
      }
      const normalizedShift = wrapIndex(shiftAmount, list.length);
      if (normalizedShift === 0) {
        return { result: list.slice() };
      }
      const result = new Array(list.length);
      for (let index = 0; index < list.length; index += 1) {
        const sourceIndex = wrapIndex(index - normalizedShift, list.length);
        result[index] = list[sourceIndex];
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['5a13ec19-e4e9-43da-bf65-f93025fa87ca']),
    'Shortest List',
    'shortest list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { A: 'listA', B: 'listB' },
      outputs: { A: 'resultA', B: 'resultB' },
    },
    eval: ({ inputs }) => {
      const listA = toList(inputs.listA);
      const listB = toList(inputs.listB);
      const targetLength = Math.min(listA.length, listB.length);
      return {
        resultA: listA.slice(0, targetLength),
        resultB: listB.slice(0, targetLength),
      };
    },
  });

  register([
    ...GUID_KEYS(['5a93246d-2595-4c28-bc2d-90657634f92a']),
    'Partition List',
    'partition list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', S: 'sizes', Size: 'sizes', Sizes: 'sizes' },
      outputs: { C: 'chunks', Chunks: 'chunks' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const sizeEntries = toList(inputs.sizes).map((entry) => Math.max(0, toInteger(entry, 0, toNumber)));
      if (!sizeEntries.length) {
        return { chunks: createDataTree(list.length ? [list.slice()] : []) };
      }
      const groups = [];
      let cursor = 0;
      while (cursor < list.length) {
        for (const size of sizeEntries) {
          if (cursor >= list.length) {
            break;
          }
          const count = size || 0;
          if (count <= 0) {
            groups.push([]);
            continue;
          }
          const next = list.slice(cursor, cursor + count);
          groups.push(next);
          cursor += count;
        }
      }
      return { chunks: createDataTree(groups) };
    },
  });

  register([
    ...GUID_KEYS(['66fbaae1-0fcf-4dbf-bcba-4395d8f6a3e6']),
    'Null Item Tree',
    'null item tree',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { I: 'items', Items: 'items' },
      outputs: { N: 'nullFlags', X: 'invalidFlags' },
    },
    eval: ({ inputs }) => {
      const items = inputs.items;
      const nullFlags = mapStructure(items, (value) => isNullLike(value));
      const invalidFlags = mapStructure(items, (value) => (!isNullLike(value) ? isInvalidValue(value) : false));
      return { nullFlags, invalidFlags };
    },
  });

  register([
    ...GUID_KEYS(['6ec97ea8-c559-47a2-8d0f-ce80c794d1f4']),
    'Reverse List',
    'reverse list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list' },
      outputs: { L: 'result', List: 'result' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      return { result: list.reverse() };
    },
  });

  register([
    ...GUID_KEYS(['7a218bfb-b93d-4c1f-83d3-5a0b909dd60b']),
    'Replace Items',
    'replace items',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        I: 'items',
        Item: 'items',
        i: 'indices',
        Indices: 'indices',
        W: 'wrap',
        Wrap: 'wrap',
      },
      outputs: { L: 'result', List: 'result' },
    },
    eval: ({ inputs }) => {
      const base = toList(inputs.list);
      if (!base.length) {
        return { result: [] };
      }
      const replacements = toList(inputs.items);
      const indices = toList(inputs.indices).map((value) => toInteger(value, 0, toNumber));
      const wrap = toBoolean(inputs.wrap, false);
      const result = base.slice();
      for (let index = 0; index < indices.length; index += 1) {
        let targetIndex = indices[index];
        if (wrap) {
          targetIndex = wrapIndex(targetIndex, result.length);
        }
        if (targetIndex < 0 || targetIndex >= result.length) {
          continue;
        }
        const replacement = replacements.length ? replacements[Math.min(index, replacements.length - 1)] : null;
        result[targetIndex] = replacement;
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['8440fd1b-b6e0-4bdb-aa93-4ec295c213e9']),
    'Longest List',
    'longest list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { A: 'listA', B: 'listB' },
      outputs: { A: 'resultA', B: 'resultB' },
    },
    eval: ({ inputs }) => {
      const listA = toList(inputs.listA);
      const listB = toList(inputs.listB);
      const maxLength = Math.max(listA.length, listB.length);
      const extend = (list) => {
        if (!list.length) {
          return Array(maxLength).fill(null);
        }
        const result = [];
        for (let index = 0; index < maxLength; index += 1) {
          result.push(index < list.length ? list[index] : list[list.length - 1]);
        }
        return result;
      };
      return { resultA: extend(listA), resultB: extend(listB) };
    },
  });

  register([
    ...GUID_KEYS(['9ab93e1a-ebdf-4090-9296-b000cff7b202']),
    'Split List',
    'split list',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', i: 'index', Index: 'index' },
      outputs: { A: 'left', B: 'right' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      if (!list.length) {
        return { left: [], right: [] };
      }
      let splitIndex = toInteger(inputs.index, 0, toNumber);
      if (splitIndex < 0) {
        splitIndex = 0;
      }
      if (splitIndex > list.length) {
        splitIndex = list.length;
      }
      return { left: list.slice(0, splitIndex), right: list.slice(splitIndex) };
    },
  });

  register([
    ...GUID_KEYS(['a759fd55-e6be-4673-8365-c28d5b52c6c0']),
    'Item Index',
    'item index',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', i: 'item', Item: 'item' },
      outputs: { i: 'index', Index: 'index' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const item = inputs.item;
      const index = list.findIndex((entry) => valuesEqual(entry, item));
      return { index: index >= 0 ? index : -1 };
    },
  });

  register([
    ...GUID_KEYS(['b333ff42-93bd-406b-8e17-15780719b6ec']),
    'Sub List',
    'sub list',
    'subset',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        D: 'domain',
        Domain: 'domain',
        W: 'wrap',
        Wrap: 'wrap',
      },
      outputs: { L: 'subset', I: 'indices' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      if (!list.length) {
        return { subset: [], indices: [] };
      }
      const domain = resolveDomain(inputs.domain, list.length, toNumber);
      const wrap = toBoolean(inputs.wrap, false);
      const start = domain.start ?? 0;
      const end = domain.end ?? start;
      const step = start <= end ? 1 : -1;
      const subset = [];
      const indices = [];
      for (let index = start; step > 0 ? index <= end : index >= end; index += step) {
        let actualIndex = index;
        if (wrap) {
          actualIndex = wrapIndex(actualIndex, list.length);
        }
        if (actualIndex < 0 || actualIndex >= list.length) {
          if (!wrap) {
            continue;
          }
        }
        subset.push(list[actualIndex] ?? null);
        indices.push(actualIndex);
      }
      return { subset, indices };
    },
  });

  register([
    ...GUID_KEYS(['d8332545-21b2-4716-96e3-8559a9876e17']),
    'Dispatch',
    'dispatch',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { L: 'list', List: 'list', P: 'pattern', Pattern: 'pattern' },
      outputs: { A: 'trueValues', B: 'falseValues' },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const pattern = toList(inputs.pattern);
      const positives = [];
      const negatives = [];
      if (!pattern.length) {
        return { trueValues: positives, falseValues: list.slice() };
      }
      for (let index = 0; index < list.length; index += 1) {
        const isPositive = toBoolean(pattern[index % pattern.length], false);
        (isPositive ? positives : negatives).push(list[index]);
      }
      return { trueValues: positives, falseValues: negatives };
    },
  });

  register([
    ...GUID_KEYS(['e2039b07-d3f3-40f8-af88-d74fed238727']),
    'Insert Items',
    'insert items',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        I: 'items',
        Item: 'items',
        i: 'indices',
        Indices: 'indices',
        W: 'wrap',
        Wrap: 'wrap',
      },
      outputs: { L: 'result', List: 'result' },
    },
    eval: ({ inputs }) => {
      const base = toList(inputs.list);
      const items = toList(inputs.items);
      const indices = toList(inputs.indices).map((value) => toInteger(value, base.length, toNumber));
      const wrap = toBoolean(inputs.wrap, false);
      const result = base.slice();
      for (let index = 0; index < indices.length; index += 1) {
        const item = items.length ? items[Math.min(index, items.length - 1)] : null;
        let targetIndex = indices[index];
        const currentLength = result.length + index;
        if (wrap) {
          targetIndex = wrapIndex(targetIndex, currentLength + 1);
        } else {
          if (targetIndex < 0) {
            targetIndex = 0;
          }
          if (targetIndex > currentLength) {
            targetIndex = currentLength;
          }
        }
        result.splice(targetIndex, 0, item);
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['e7c80ff6-0299-4303-be36-3080977c14a1']),
    'Combine Data',
    'combine data',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: {
        0: 'input0',
        1: 'input1',
        2: 'input2',
        3: 'input3',
        'Input 0': 'input0',
        'Input 1': 'input1',
        'Input 2': 'input2',
        'Input 3': 'input3',
      },
      outputs: { R: 'result', I: 'indices' },
    },
    eval: ({ inputs }) => {
      const streams = gatherIndexedInputs(inputs, /^(?:input\s*)?(\d+)$/i);
      if (!streams.length) {
        return { result: [], indices: [] };
      }
      const maxLength = Math.max(...streams.map((stream) => stream.length));
      const result = [];
      const indices = [];
      for (let itemIndex = 0; itemIndex < maxLength; itemIndex += 1) {
        let picked = null;
        let pickedIndex = -1;
        for (let streamIndex = 0; streamIndex < streams.length; streamIndex += 1) {
          const stream = streams[streamIndex];
          const value = stream[itemIndex];
          if (!isNullLike(value)) {
            picked = value;
            pickedIndex = streamIndex;
            break;
          }
        }
        result.push(picked);
        indices.push(pickedIndex);
      }
      return { result, indices };
    },
  });

  register([
    ...GUID_KEYS(['f3230ecb-3631-4d6f-86f2-ef4b2ed37f45']),
    'Replace Nulls',
    'replace nulls',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { I: 'items', Items: 'items', R: 'replacements', Replacements: 'replacements' },
      outputs: { I: 'result', Items: 'result', N: 'count', Count: 'count' },
    },
    eval: ({ inputs }) => {
      const items = toList(inputs.items);
      const replacements = toList(inputs.replacements);
      const result = [];
      let replacementCount = 0;
      for (let index = 0; index < items.length; index += 1) {
        const value = items[index];
        if (!isNullLike(value) && !isInvalidValue(value)) {
          result.push(value);
          continue;
        }
        const replacement = replacements[index] ?? replacements[replacements.length - 1] ?? null;
        if (!isNullLike(replacement) || isInvalidValue(value)) {
          replacementCount += 1;
        }
        result.push(replacement);
      }
      return { result, count: replacementCount };
    },
  });

  register([
    ...GUID_KEYS(['c74efd0e-7fe3-4c2d-8c9d-295c5672fb13']),
    'Null Item',
    'null item (single)',
  ], {
    type: 'sets:list',
    pinMap: {
      inputs: { I: 'item', Item: 'item' },
      outputs: { N: 'nullFlag', X: 'invalidFlag', D: 'description' },
    },
    eval: ({ inputs }) => {
      const value = inputs.item;
      const isNull = isNullLike(value);
      const isInvalid = !isNull && isInvalidValue(value);
      return {
        nullFlag: isNull,
        invalidFlag: isInvalid,
        description: describeValueState(value, isNull, isInvalid),
      };
    },
  });
}

export function registerSetsSetsComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  register([
    ...GUID_KEYS(['190d042c-2270-4bc1-81c0-4f90c170c9c9']),
    'Delete Consecutive',
    'delete consecutive',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { S: 'set', Set: 'set', set: 'set', W: 'wrap', Wrap: 'wrap' },
      outputs: { S: 'result', Set: 'result', N: 'count', Count: 'count' },
    },
    eval: ({ inputs }) => {
      const source = toList(inputs.set);
      if (!source.length) {
        return { result: [], count: 0 };
      }
      const wrap = toBoolean(inputs.wrap, false);
      const result = [];
      let removed = 0;
      for (let index = 0; index < source.length; index += 1) {
        const value = source[index];
        if (!result.length) {
          result.push(value);
          continue;
        }
        const previous = source[index - 1];
        if (valuesEqual(previous, value)) {
          removed += 1;
          continue;
        }
        result.push(value);
      }
      if (wrap && result.length > 1 && valuesEqual(result[0], result[result.length - 1])) {
        result.pop();
        removed += 1;
      }
      return { result, count: removed };
    },
  });

  register([
    ...GUID_KEYS(['1edcc3cf-cf84-41d4-8204-561162cfe510']),
    'Key/Value Search',
    'key value search',
    'key search',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { K: 'keys', Keys: 'keys', V: 'values', Values: 'values', S: 'search', Search: 'search' },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const keys = toList(inputs.keys);
      const values = toList(inputs.values);
      const search = inputs.search;
      const index = findValueIndex(keys, search);
      return { result: index >= 0 ? values[index] ?? null : null };
    },
  });

  const registerCreateSet = (guid, includeMap) => {
    register([
      ...GUID_KEYS([guid]),
      'Create Set',
      'create set',
      'set:create',
    ], {
      type: 'sets:sets',
      pinMap: {
        inputs: { L: 'list', List: 'list' },
        outputs: includeMap
          ? { S: 'set', Set: 'set', M: 'map', Map: 'map' }
          : { S: 'set', Set: 'set' },
      },
      eval: ({ inputs }) => {
        const list = toList(inputs.list);
        const { unique, map } = createUniqueSetWithMap(list);
        if (includeMap) {
          return { set: unique, map };
        }
        return { set: unique };
      },
    });
  };

  registerCreateSet('2cb4bf85-a282-464c-b42c-8e735d2a0a74', false);
  registerCreateSet('98c3c63a-e78a-43ea-a111-514fcf312c95', true);

  register([
    ...GUID_KEYS(['3ff27857-b988-417a-b495-b24c733dbd00']),
    'Member Index',
    'member index',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { S: 'set', Set: 'set', set: 'set', M: 'member', Member: 'member' },
      outputs: { I: 'indices', Index: 'indices', indices: 'indices', N: 'count', Count: 'count' },
    },
    eval: ({ inputs }) => {
      const set = toList(inputs.set);
      const member = inputs.member;
      const indices = [];
      for (let index = 0; index < set.length; index += 1) {
        if (valuesEqual(set[index], member)) {
          indices.push(index);
        }
      }
      return { indices, count: indices.length };
    },
  });

  register([
    ...GUID_KEYS(['4cfc0bb0-0745-4772-a520-39f9bf3d99bc']),
    'SubSet',
    'subset',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const setA = uniqueValues(toList(inputs.setA));
      const setB = uniqueValues(toList(inputs.setB));
      const result = setB.every((value) => includesValue(setA, value));
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['81800098-1060-4e2b-80d4-17f835cc825f']),
    'Disjoint',
    'disjoint',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const setA = uniqueValues(toList(inputs.setA));
      const setB = uniqueValues(toList(inputs.setB));
      const result = setA.every((value) => !includesValue(setB, value));
      return { result };
    },
  });

  register([
    ...GUID_KEYS([
      '82f19c48-9e73-43a4-ae6c-3a8368099b08',
      '8a55f680-cf53-4634-a486-b828de92b71d',
    ]),
    'Set Intersection',
    'set intersection',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { U: 'intersection', Union: 'intersection', Intersection: 'intersection' },
    },
    eval: ({ inputs }) => {
      const setA = uniqueValues(toList(inputs.setA));
      const setB = uniqueValues(toList(inputs.setB));
      const intersection = [];
      for (const value of setA) {
        if (includesValue(setB, value)) {
          intersection.push(value);
        }
      }
      return { intersection };
    },
  });

  register([
    ...GUID_KEYS([
      '8eed5d78-7810-4ba1-968e-8a1f1db98e39',
      'ab34845d-4ab9-4ff4-8870-eedd0c5594cb',
    ]),
    'Set Union',
    'set union',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { U: 'union', Union: 'union' },
    },
    eval: ({ inputs }) => {
      const union = [];
      const appendUnique = (values) => {
        for (const value of values) {
          if (!includesValue(union, value)) {
            union.push(value);
          }
        }
      };
      appendUnique(toList(inputs.setA));
      appendUnique(toList(inputs.setB));
      return { union };
    },
  });

  register([
    ...GUID_KEYS(['b4d4235f-14ff-4d4e-a29a-b358dcd2baf4']),
    'Find similar member',
    'find similar member',
    'find similar',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { D: 'data', Data: 'data', S: 'set', Set: 'set' },
      outputs: { H: 'hit', Hit: 'hit', i: 'index', Index: 'index' },
    },
    eval: ({ inputs }) => {
      const data = inputs.data;
      const set = toList(inputs.set);
      if (!set.length) {
        return { hit: null, index: -1 };
      }

      let bestIndex = -1;
      let bestScore = Number.POSITIVE_INFINITY;
      let bestValue = null;

      const numericTarget = toNumber(data, Number.NaN);
      const hasNumericTarget = Number.isFinite(numericTarget);
      const targetString = data === undefined || data === null ? '' : String(data).toLowerCase();

      for (let index = 0; index < set.length; index += 1) {
        const candidate = set[index];
        if (valuesEqual(candidate, data)) {
          bestIndex = index;
          bestValue = candidate;
          bestScore = Number.NEGATIVE_INFINITY;
          break;
        }

        let score = Number.POSITIVE_INFINITY;

        if (hasNumericTarget) {
          const numericCandidate = toNumber(candidate, Number.NaN);
          if (Number.isFinite(numericCandidate)) {
            score = Math.abs(numericCandidate - numericTarget);
          }
        }

        if (!Number.isFinite(score)) {
          const candidateString = candidate === undefined || candidate === null ? '' : String(candidate).toLowerCase();
          if (targetString || candidateString) {
            score = levenshteinDistance(targetString, candidateString) + 1;
          } else {
            score = 1;
          }
        }

        if (!Number.isFinite(score)) {
          score = Number.MAX_VALUE;
        }

        if (score < bestScore) {
          bestScore = score;
          bestIndex = index;
          bestValue = candidate;
        }
      }

      if (bestIndex === -1) {
        bestIndex = 0;
        bestValue = set[0];
      }

      return { hit: bestValue ?? null, index: bestIndex };
    },
  });

  register([
    ...GUID_KEYS(['bafac914-ede4-4a59-a7b2-cc41bc3de961']),
    'Replace Members',
    'replace members',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { S: 'set', Set: 'set', set: 'set', F: 'find', Find: 'find', R: 'replace', Replace: 'replace' },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const base = toList(inputs.set);
      if (!base.length) {
        return { result: [] };
      }
      const find = toList(inputs.find);
      const replace = toList(inputs.replace);
      if (!find.length) {
        return { result: base.slice() };
      }
      const pairs = find.map((value, index) => ({ value, replacement: replace[index] ?? null }));
      const result = base.map((value) => {
        for (const pair of pairs) {
          if (valuesEqual(value, pair.value)) {
            return pair.replacement;
          }
        }
        return value;
      });
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['d2461702-3164-4894-8c10-ed1fc4b52965']),
    'Set Difference (S)',
    'symmetric difference',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { X: 'symmetricDifference', ExDifference: 'symmetricDifference' },
    },
    eval: ({ inputs }) => {
      const setA = uniqueValues(toList(inputs.setA));
      const setB = uniqueValues(toList(inputs.setB));
      const result = [];
      for (const value of setA) {
        if (!includesValue(setB, value)) {
          result.push(value);
        }
      }
      for (const value of setB) {
        if (!includesValue(setA, value) && !includesValue(result, value)) {
          result.push(value);
        }
      }
      return { symmetricDifference: result };
    },
  });

  register([
    ...GUID_KEYS(['d4136a7b-7422-4660-9404-640474bd2725']),
    'Set Majority',
    'set majority',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB', C: 'setC', 'Set C': 'setC' },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const sets = [inputs.setA, inputs.setB, inputs.setC].map((entry) => uniqueValues(toList(entry)));
      const candidates = [];
      for (const unique of sets) {
        for (const value of unique) {
          if (!includesValue(candidates, value)) {
            candidates.push(value);
          }
        }
      }
      const result = [];
      for (const candidate of candidates) {
        let count = 0;
        for (const unique of sets) {
          if (includesValue(unique, candidate)) {
            count += 1;
          }
        }
        if (count >= 2 && !includesValue(result, candidate)) {
          result.push(candidate);
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['deffaf1e-270a-4c15-a693-9216b68afd4a']),
    'Carthesian Product',
    'cartesian product',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { P: 'product', Product: 'product' },
    },
    eval: ({ inputs }) => {
      const setA = toList(inputs.setA);
      const setB = toList(inputs.setB);
      const branches = [];
      for (let indexA = 0; indexA < setA.length; indexA += 1) {
        const branch = [];
        for (let indexB = 0; indexB < setB.length; indexB += 1) {
          branch.push([setA[indexA], setB[indexB]]);
        }
        branches.push(branch);
      }
      return { product: createDataTree(branches) };
    },
  });

  register([
    ...GUID_KEYS(['e3b1a10c-4d49-4140-b8e6-0b5732a26c31']),
    'Set Difference',
    'set difference',
  ], {
    type: 'sets:sets',
    pinMap: {
      inputs: { A: 'setA', 'Set A': 'setA', B: 'setB', 'Set B': 'setB' },
      outputs: { U: 'difference', Union: 'difference', Difference: 'difference' },
    },
    eval: ({ inputs }) => {
      const setA = uniqueValues(toList(inputs.setA));
      const setB = uniqueValues(toList(inputs.setB));
      const difference = [];
      for (const value of setA) {
        if (!includesValue(setB, value)) {
          difference.push(value);
        }
      }
      return { difference };
    },
  });
}

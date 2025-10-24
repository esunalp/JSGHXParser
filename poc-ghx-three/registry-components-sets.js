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

function normalizePathSegment(value, toNumber, fallback = 0) {
  if (typeof value === 'number') {
    return Number.isFinite(value) ? Math.trunc(value) : fallback;
  }
  if (typeof value === 'bigint') {
    return Number(value);
  }
  return toInteger(value, fallback, toNumber);
}

function parsePathInput(value, toNumber) {
  if (value === undefined || value === null) {
    return [];
  }
  if (Array.isArray(value)) {
    return value.map((segment) => normalizePathSegment(segment, toNumber, 0));
  }
  if (typeof value === 'object') {
    if (Array.isArray(value?.path)) {
      return value.path.map((segment) => normalizePathSegment(segment, toNumber, 0));
    }
    if (Array.isArray(value?.indices)) {
      return value.indices.map((segment) => normalizePathSegment(segment, toNumber, 0));
    }
    if (Object.prototype.hasOwnProperty.call(value, 'value')) {
      return parsePathInput(value.value, toNumber);
    }
  }
  const text = String(value).trim();
  if (!text) {
    return [];
  }
  const sanitized = text.replace(/^\{+/, '').replace(/\}+$/, '');
  if (!sanitized) {
    return [];
  }
  const parts = sanitized.split(/[;,\s]+/).filter(Boolean);
  return parts.map((part) => normalizePathSegment(part, toNumber, 0));
}

function formatPathKey(path = []) {
  return Array.isArray(path) ? path.join(';') : '';
}

function comparePaths(pathA = [], pathB = []) {
  const length = Math.min(pathA.length, pathB.length);
  for (let index = 0; index < length; index += 1) {
    const valueA = pathA[index];
    const valueB = pathB[index];
    if (valueA < valueB) return -1;
    if (valueA > valueB) return 1;
  }
  if (pathA.length < pathB.length) return -1;
  if (pathA.length > pathB.length) return 1;
  return 0;
}

function cloneBranch(branch) {
  return {
    path: Array.isArray(branch?.path) ? branch.path.slice() : [],
    values: Array.isArray(branch?.values) ? branch.values.slice() : [],
  };
}

function normalizeTreeBranches(input, toNumber) {
  if (!input) {
    return [];
  }
  if (input.type === 'tree' && Array.isArray(input.branches)) {
    return input.branches.map((branch) => ({
      path: parsePathInput(branch?.path ?? [], toNumber),
      values: Array.isArray(branch?.values)
        ? branch.values.slice()
        : branch?.values === undefined || branch?.values === null
          ? []
          : [branch.values],
    }));
  }
  if (Array.isArray(input)) {
    return [{ path: [], values: input.slice() }];
  }
  return [{ path: [], values: input === undefined || input === null ? [] : [input] }];
}

function createTreeFromBranches(branches = []) {
  return {
    type: 'tree',
    branches: branches.map((branch) => ({
      path: Array.isArray(branch?.path) ? branch.path.slice() : [],
      values: Array.isArray(branch?.values) ? branch.values.slice() : [],
    })),
  };
}

function mergeBranches(branches = []) {
  const map = new Map();
  for (const branch of branches) {
    if (!branch) continue;
    const key = formatPathKey(branch.path);
    if (!map.has(key)) {
      map.set(key, { path: Array.isArray(branch.path) ? branch.path.slice() : [], values: [] });
    }
    const target = map.get(key);
    if (Array.isArray(branch.values)) {
      target.values.push(...branch.values);
    } else if (branch.values !== undefined && branch.values !== null) {
      target.values.push(branch.values);
    }
  }
  const merged = Array.from(map.values());
  merged.sort((a, b) => comparePaths(a.path, b.path));
  return merged;
}

function flattenBranchValues(branches = []) {
  const values = [];
  for (const branch of branches) {
    if (!branch) continue;
    if (Array.isArray(branch.values)) {
      values.push(...branch.values);
    } else if (branch.values !== undefined && branch.values !== null) {
      values.push(branch.values);
    }
  }
  return values;
}

function toPathString(path = []) {
  if (!Array.isArray(path) || !path.length) {
    return '{}';
  }
  return `{${path.join(';')}}`;
}

function gatherIndexedTreeInputs(inputs, prefixRegex, toNumber) {
  const entries = [];
  for (const [key, value] of Object.entries(inputs || {})) {
    const normalized = String(key);
    const match = normalized.match(prefixRegex);
    if (!match) continue;
    const index = Number.parseInt(match[1], 10);
    if (!Number.isFinite(index)) continue;
    entries[index] = normalizeTreeBranches(value, toNumber);
  }
  return entries.map((branches) => (Array.isArray(branches) ? branches : []));
}

function ensureTree(input, toNumber) {
  return createTreeFromBranches(normalizeTreeBranches(input, toNumber));
}

function collectTreeValues(input, toNumber) {
  return flattenBranchValues(normalizeTreeBranches(input, toNumber));
}

function simplifyBranches(branches = [], { frontOnly = false } = {}) {
  if (!Array.isArray(branches) || branches.length <= 1) {
    return branches.map(cloneBranch);
  }
  const maxLength = Math.max(...branches.map((branch) => (Array.isArray(branch?.path) ? branch.path.length : 0)));
  if (maxLength === 0) {
    return branches.map(cloneBranch);
  }
  const removable = new Array(maxLength).fill(true);
  for (let position = 0; position < maxLength; position += 1) {
    let sharedValue = null;
    let comparable = true;
    for (const branch of branches) {
      const path = Array.isArray(branch?.path) ? branch.path : [];
      if (position >= path.length) {
        comparable = false;
        break;
      }
      if (sharedValue === null) {
        sharedValue = path[position];
        continue;
      }
      if (path[position] !== sharedValue) {
        comparable = false;
        break;
      }
    }
    removable[position] = comparable;
  }
  if (frontOnly) {
    for (let index = 0; index < removable.length; index += 1) {
      if (!removable[index]) {
        for (let reset = index; reset < removable.length; reset += 1) {
          removable[reset] = false;
        }
        break;
      }
    }
  }
  return branches.map((branch) => {
    const path = Array.isArray(branch?.path) ? branch.path : [];
    const values = Array.isArray(branch?.values) ? branch.values.slice() : [];
    const simplifiedPath = [];
    for (let index = 0; index < path.length; index += 1) {
      if (!removable[index]) {
        simplifiedPath.push(path[index]);
      }
    }
    return { path: simplifiedPath, values };
  });
}

function trimBranches(branches = [], depth = 0, { fromEnd = true } = {}) {
  if (depth <= 0) {
    return branches.map(cloneBranch);
  }
  return branches.map((branch) => {
    const path = Array.isArray(branch?.path) ? branch.path : [];
    const values = Array.isArray(branch?.values) ? branch.values.slice() : [];
    if (!path.length) {
      return { path: [], values };
    }
    if (fromEnd) {
      return { path: path.slice(0, Math.max(0, path.length - depth)), values };
    }
    return { path: path.slice(Math.min(depth, path.length)), values };
  });
}

function flattenTreeToPath(branches = [], targetPath = []) {
  const values = flattenBranchValues(branches);
  return [{ path: Array.isArray(targetPath) ? targetPath.slice() : [], values }];
}

function sortBranches(branches = []) {
  return branches.slice().sort((a, b) => comparePaths(a?.path, b?.path));
}

function parsePathPattern(mask, toNumber) {
  const tokens = [];
  const segments = parsePathInput(mask, toNumber);
  if (Array.isArray(mask) && mask.some((value) => value === '*' || value === '?')) {
    for (const segment of mask) {
      if (segment === '*') {
        tokens.push({ type: 'wildcard' });
      } else if (segment === '?') {
        tokens.push({ type: 'single' });
      } else {
        tokens.push({ type: 'exact', value: normalizePathSegment(segment, toNumber, 0) });
      }
    }
    return tokens;
  }
  if (typeof mask === 'string') {
    const text = mask.trim();
    if (!text) return segments.map((value) => ({ type: 'exact', value }));
    const sanitized = text.replace(/^\{+/, '').replace(/\}+$/, '');
    if (!sanitized) {
      return [];
    }
    for (const part of sanitized.split(/[;,\s]+/).filter(Boolean)) {
      if (part === '*') {
        tokens.push({ type: 'wildcard' });
      } else if (part === '?') {
        tokens.push({ type: 'single' });
      } else {
        tokens.push({ type: 'exact', value: normalizePathSegment(part, toNumber, 0) });
      }
    }
    return tokens;
  }
  return segments.map((value) => ({ type: 'exact', value }));
}

function matchPathWithPattern(path, pattern) {
  const segments = Array.isArray(path) ? path : [];
  const tokens = Array.isArray(pattern) ? pattern : [];
  function match(pathIndex, tokenIndex) {
    if (tokenIndex >= tokens.length) {
      return pathIndex >= segments.length;
    }
    const token = tokens[tokenIndex];
    if (!token) {
      return pathIndex >= segments.length;
    }
    if (token.type === 'wildcard') {
      for (let skip = pathIndex; skip <= segments.length; skip += 1) {
        if (match(skip, tokenIndex + 1)) {
          return true;
        }
      }
      return false;
    }
    if (pathIndex >= segments.length) {
      return false;
    }
    if (token.type === 'single') {
      return match(pathIndex + 1, tokenIndex + 1);
    }
    if (token.type === 'exact') {
      if (segments[pathIndex] !== token.value) {
        return false;
      }
      return match(pathIndex + 1, tokenIndex + 1);
    }
    return false;
  }
  return match(0, 0);
}

function parseMaskList(maskInput, toNumber) {
  const masks = [];
  if (maskInput === undefined || maskInput === null) {
    return masks;
  }
  if (Array.isArray(maskInput)) {
    for (const entry of maskInput) {
      masks.push(parsePathPattern(entry, toNumber));
    }
    return masks;
  }
  masks.push(parsePathPattern(maskInput, toNumber));
  return masks;
}

function findBranchByPath(branches, targetPath) {
  const key = formatPathKey(targetPath);
  for (const branch of branches ?? []) {
    if (formatPathKey(branch?.path) === key) {
      return branch;
    }
  }
  return null;
}

function ensureArray(value) {
  if (value === undefined || value === null) {
    return [];
  }
  return Array.isArray(value) ? value : [value];
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

function createSeededRandom(seedInput, toNumber) {
  const numericSeed = toInteger(seedInput, Number.NaN, toNumber);
  if (!Number.isFinite(numericSeed)) {
    return () => Math.random();
  }
  let state = numericSeed % 2147483647;
  if (state <= 0) {
    state += 2147483646;
  }
  return () => {
    state = (state * 16807) % 2147483647;
    return state / 2147483647;
  };
}

function createNumericRange(startCandidate, endCandidate, toNumber, fallbackRange) {
  const start = toNumber ? toNumber(startCandidate, Number.NaN) : Number(startCandidate);
  const end = toNumber ? toNumber(endCandidate, Number.NaN) : Number(endCandidate);
  if (Number.isFinite(start) && Number.isFinite(end)) {
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    return {
      start,
      end,
      min,
      max,
      span: end - start,
      length: max - min,
      center: (start + end) / 2,
    };
  }
  return fallbackRange;
}

function ensureNumericRange(rangeInput, toNumber, fallbackStart = 0, fallbackEnd = 1) {
  const fallbackRange = createNumericRange(
    fallbackStart,
    fallbackEnd,
    toNumber,
    {
      start: fallbackStart,
      end: fallbackEnd,
      min: Math.min(fallbackStart, fallbackEnd),
      max: Math.max(fallbackStart, fallbackEnd),
      span: fallbackEnd - fallbackStart,
      length: Math.abs(fallbackEnd - fallbackStart),
      center: (fallbackStart + fallbackEnd) / 2,
    }
  );

  const resolveRange = (startCandidate, endCandidate) =>
    createNumericRange(startCandidate ?? fallbackRange.start, endCandidate ?? fallbackRange.end, toNumber, fallbackRange);

  if (rangeInput === undefined || rangeInput === null) {
    return fallbackRange;
  }

  if (Array.isArray(rangeInput)) {
    if (rangeInput.length >= 2) {
      return resolveRange(rangeInput[0], rangeInput[1]);
    }
    if (rangeInput.length === 1) {
      return resolveRange(fallbackRange.start, rangeInput[0]);
    }
    return fallbackRange;
  }

  if (typeof rangeInput === 'object') {
    if (Array.isArray(rangeInput.values)) {
      const valuesRange = ensureNumericRange(rangeInput.values, toNumber, fallbackRange.start, fallbackRange.end);
      if (valuesRange) {
        return valuesRange;
      }
    }
    if (Object.prototype.hasOwnProperty.call(rangeInput, 'value')) {
      const valueRange = ensureNumericRange(rangeInput.value, toNumber, fallbackRange.start, fallbackRange.end);
      if (valueRange) {
        return valueRange;
      }
    }
    if (Array.isArray(rangeInput.range)) {
      const nested = ensureNumericRange(rangeInput.range, toNumber, fallbackRange.start, fallbackRange.end);
      if (nested) {
        return nested;
      }
    }
    if (Array.isArray(rangeInput.domain)) {
      const nested = ensureNumericRange(rangeInput.domain, toNumber, fallbackRange.start, fallbackRange.end);
      if (nested) {
        return nested;
      }
    }
    if (rangeInput.dimension === 1 && rangeInput.start !== undefined && rangeInput.end !== undefined) {
      return resolveRange(rangeInput.start, rangeInput.end);
    }
    const startCandidate =
      rangeInput.start ??
      rangeInput.Start ??
      rangeInput.s ??
      rangeInput.S ??
      rangeInput.a ??
      rangeInput.A ??
      rangeInput.min ??
      rangeInput.Min ??
      rangeInput.from ??
      rangeInput.From ??
      rangeInput.lower ??
      rangeInput.Lower ??
      rangeInput.t0 ??
      rangeInput.T0 ??
      rangeInput[0];
    const endCandidate =
      rangeInput.end ??
      rangeInput.End ??
      rangeInput.e ??
      rangeInput.E ??
      rangeInput.b ??
      rangeInput.B ??
      rangeInput.max ??
      rangeInput.Max ??
      rangeInput.to ??
      rangeInput.To ??
      rangeInput.upper ??
      rangeInput.Upper ??
      rangeInput.t1 ??
      rangeInput.T1 ??
      rangeInput[1];
    if (startCandidate !== undefined || endCandidate !== undefined) {
      return resolveRange(startCandidate, endCandidate);
    }
    if (rangeInput.min !== undefined && rangeInput.max !== undefined) {
      return resolveRange(rangeInput.min, rangeInput.max);
    }
    if (rangeInput.t0 !== undefined && rangeInput.t1 !== undefined) {
      return resolveRange(rangeInput.t0, rangeInput.t1);
    }
    if (rangeInput.value !== undefined) {
      return resolveRange(rangeInput.value, rangeInput.value);
    }
  }

  const numeric = toNumber(rangeInput, Number.NaN);
  if (Number.isFinite(numeric)) {
    return resolveRange(fallbackRange.start, numeric);
  }

  if (typeof rangeInput === 'string') {
    const matches = rangeInput.match(/-?\d+(?:\.\d+)?/g);
    if (matches && matches.length) {
      if (matches.length === 1) {
        return resolveRange(fallbackRange.start, Number.parseFloat(matches[0]));
      }
      return resolveRange(Number.parseFloat(matches[0]), Number.parseFloat(matches[1]));
    }
  }

  return fallbackRange;
}

const DEFAULT_CHAR_POOL = Array.from('ABCDEFGHIJKLMNOPQRSTUVWXYZ');

function normalizeCharPool(poolInput) {
  const tokens = [];

  const process = (value) => {
    if (value === undefined || value === null) {
      return;
    }
    if (typeof value === 'string') {
      const trimmed = value.trim();
      if (!trimmed) {
        return;
      }
      if (/[;,\s]/.test(trimmed)) {
        const parts = trimmed.split(/[;,\s]+/).filter(Boolean);
        if (parts.length) {
          tokens.push(...parts);
          return;
        }
      }
      for (const char of trimmed) {
        tokens.push(char);
      }
      return;
    }
    if (Array.isArray(value)) {
      for (const entry of value) {
        process(entry);
      }
      return;
    }
    if (typeof value === 'object') {
      if (Array.isArray(value.values)) {
        process(value.values);
        return;
      }
      if (Object.prototype.hasOwnProperty.call(value, 'value')) {
        process(value.value);
        return;
      }
    }
    const text = String(value);
    if (text) {
      tokens.push(text);
    }
  };

  process(poolInput);

  if (!tokens.length) {
    return DEFAULT_CHAR_POOL.slice();
  }

  return tokens;
}

function createCharTokenFromIndex(index, tokens) {
  if (!Array.isArray(tokens) || !tokens.length) {
    return '';
  }
  const base = tokens.length;
  let n = index + 1;
  const digits = [];
  while (n > 0) {
    const remainder = (n - 1) % base;
    digits.push(remainder);
    n = Math.floor((n - 1) / base);
  }
  let result = '';
  for (let digitIndex = digits.length - 1; digitIndex >= 0; digitIndex -= 1) {
    const token = tokens[digits[digitIndex]];
    result += token !== undefined ? String(token) : '';
  }
  return result;
}

function applyFormatMask(value, mask, index) {
  const textValue = value === undefined || value === null ? '' : String(value);
  if (mask === undefined || mask === null) {
    return textValue;
  }
  let formatted = String(mask);
  let replaced = false;
  const valueTokens = ['{0}', '{}', '%s', '{value}', '${value}'];
  for (const token of valueTokens) {
    if (formatted.includes(token)) {
      formatted = formatted.split(token).join(textValue);
      replaced = true;
    }
  }
  const zeroBased = String(index);
  const oneBased = String(index + 1);
  const indexTokens = [
    ['{i}', zeroBased],
    ['{n}', zeroBased],
    ['{index}', zeroBased],
    ['${index}', zeroBased],
    ['{index0}', zeroBased],
    ['{index1}', oneBased],
    ['{1-based}', oneBased],
    ['{n1}', oneBased],
  ];
  for (const [token, replacement] of indexTokens) {
    if (formatted.includes(token)) {
      formatted = formatted.split(token).join(replacement);
    }
  }
  if (!replaced) {
    if (formatted) {
      formatted += textValue;
    } else {
      formatted = textValue;
    }
  }
  return formatted;
}

function compileSequenceExpression(notation) {
  if (notation === undefined || notation === null) {
    return null;
  }
  const text = String(notation).trim();
  if (!text) {
    return null;
  }
  const expression = text.includes('=') ? text.split('=').pop() : text;
  try {
    const body = `'use strict'; return (${expression});`;
    return new Function('n', 'i', 'prev', 'prev1', 'prev2', 'values', 'initial', body);
  } catch (error) {
    return null;
  }
}


export function registerSetsSequenceComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const parseCount = (value, fallback = 0) => Math.max(0, toInteger(value, fallback, toNumber));
  const parseNumber = (value, fallback = 0) => {
    const numeric = toNumber(value, Number.NaN);
    return Number.isFinite(numeric) ? numeric : fallback;
  };

  const generateRandomValues = (rangeInput, countInput, seedInput, { integers = false } = {}) => {
    const range = ensureNumericRange(rangeInput, toNumber, 0, 1);
    const count = parseCount(countInput, 10);
    const rng = createSeededRandom(seedInput, toNumber);
    const values = [];
    if (count <= 0) {
      return values;
    }
    if (integers) {
      const minValue = Math.ceil(range.min);
      const maxValue = Math.floor(range.max);
      if (maxValue < minValue) {
        const fallback = Math.round(range.min);
        for (let index = 0; index < count; index += 1) {
          values.push(fallback);
        }
        return values;
      }
      if (maxValue === minValue) {
        for (let index = 0; index < count; index += 1) {
          values.push(minValue);
        }
        return values;
      }
      const span = maxValue - minValue + 1;
      for (let index = 0; index < count; index += 1) {
        const value = minValue + Math.floor(rng() * span);
        values.push(value);
      }
      return values;
    }
    const delta = range.max - range.min;
    for (let index = 0; index < count; index += 1) {
      const value = range.min + rng() * delta;
      values.push(value);
    }
    return values;
  };

  register([
    ...GUID_KEYS(['008e9a6f-478a-4813-8c8a-546273bc3a6b']),
    'Cull Pattern',
    'cull pattern',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        list: 'list',
        P: 'pattern',
        Pattern: 'pattern',
        'Cull Pattern': 'pattern',
        pattern: 'pattern',
      },
      outputs: {
        L: 'result',
        List: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const pattern = toList(inputs.pattern);
      if (!pattern.length) {
        return { result: list.slice() };
      }
      const normalized = pattern.map((entry) => toBoolean(entry, false));
      if (!normalized.length) {
        return { result: list.slice() };
      }
      const result = [];
      for (let index = 0; index < list.length; index += 1) {
        const shouldCull = normalized[index % normalized.length];
        if (!shouldCull) {
          result.push(list[index]);
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['01640871-69ea-40ac-9380-4660d6d28bd2']),
    'Char Sequence',
    'char sequence',
    'charseq',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        C: 'count',
        Count: 'count',
        count: 'count',
        P: 'pool',
        'Char Pool': 'pool',
        pool: 'pool',
        F: 'format',
        Format: 'format',
        format: 'format',
      },
      outputs: {
        S: 'sequence',
        Sequence: 'sequence',
      },
    },
    eval: ({ inputs }) => {
      const count = parseCount(inputs.count, 0);
      if (count <= 0) {
        return { sequence: [] };
      }
      const tokens = normalizeCharPool(inputs.pool);
      const sequence = [];
      for (let index = 0; index < count; index += 1) {
        const token = createCharTokenFromIndex(index, tokens);
        sequence.push(applyFormatMask(token, inputs.format, index));
      }
      return { sequence };
    },
  });

  register([
    ...GUID_KEYS([
      '2ab17f9a-d852-4405-80e1-938c5e57e78d',
      'b7e4e0ef-a01d-48c4-93be-2a12d4417e22',
    ]),
    'Random',
    'random',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        R: 'range',
        Range: 'range',
        range: 'range',
        D: 'range',
        Domain: 'range',
        N: 'count',
        Number: 'count',
        count: 'count',
        S: 'seed',
        Seed: 'seed',
        seed: 'seed',
        I: 'integers',
        Integers: 'integers',
        integers: 'integers',
      },
      outputs: {
        R: 'values',
        Random: 'values',
        Range: 'values',
        Values: 'values',
      },
    },
    eval: ({ inputs }) => {
      const values = generateRandomValues(inputs.range ?? inputs.domain, inputs.count ?? inputs.number, inputs.seed, {
        integers: toBoolean(inputs.integers, false),
      });
      return { values };
    },
  });

  register([
    ...GUID_KEYS(['455925fd-23ff-4e57-a0e7-913a4165e659']),
    'Random Reduce',
    'random reduce',
    'reduce',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        list: 'list',
        R: 'reduction',
        Reduction: 'reduction',
        reduction: 'reduction',
        S: 'seed',
        Seed: 'seed',
        seed: 'seed',
      },
      outputs: {
        L: 'result',
        List: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const reduction = parseCount(inputs.reduction, 0);
      if (!list.length || reduction <= 0) {
        return { result: list.slice() };
      }
      const removeCount = Math.min(reduction, list.length);
      if (removeCount === list.length) {
        return { result: [] };
      }
      const rng = createSeededRandom(inputs.seed, toNumber);
      const indices = list.map((_, index) => index);
      for (let index = indices.length - 1; index > 0; index -= 1) {
        const swapIndex = Math.floor(rng() * (index + 1));
        const temp = indices[index];
        indices[index] = indices[swapIndex];
        indices[swapIndex] = temp;
      }
      const removal = new Set(indices.slice(0, removeCount));
      const result = [];
      for (let index = 0; index < list.length; index += 1) {
        if (!removal.has(index)) {
          result.push(list[index]);
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS([
      '501aecbb-c191-4d13-83d6-7ee32445ac50',
      '6568e019-f59c-4984-84d6-96bd5bfbe9e7',
    ]),
    'Cull Index',
    'cull index',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        list: 'list',
        I: 'indices',
        Indices: 'indices',
        indices: 'indices',
        W: 'wrap',
        Wrap: 'wrap',
        wrap: 'wrap',
      },
      outputs: {
        L: 'result',
        List: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const length = list.length;
      if (!length) {
        return { result: [] };
      }
      const indices = toList(inputs.indices);
      if (!indices.length) {
        return { result: list.slice() };
      }
      const wrap = toBoolean(inputs.wrap, false);
      const removal = new Set();
      for (const entry of indices) {
        const rawIndex = toInteger(entry, Number.NaN, toNumber);
        if (!Number.isFinite(rawIndex)) {
          continue;
        }
        let resolved = rawIndex;
        if (wrap) {
          resolved = wrapIndex(rawIndex, length);
        }
        if (resolved < 0 || resolved >= length) {
          continue;
        }
        removal.add(resolved);
      }
      if (!removal.size) {
        return { result: list.slice() };
      }
      const result = list.filter((_, index) => !removal.has(index));
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['5fa4e736-0d82-4af0-97fb-30a79f4cbf41']),
    'Stack Data',
    'stack data',
    'stack',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'data',
        Data: 'data',
        data: 'data',
        S: 'stack',
        Stack: 'stack',
        stack: 'stack',
      },
      outputs: {
        D: 'result',
        Data: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const data = toList(inputs.data);
      if (!data.length) {
        return { result: [] };
      }
      const patternValues = toList(inputs.stack);
      const counts = patternValues.length
        ? patternValues.map((entry) => Math.max(0, toInteger(entry, 0, toNumber)))
        : [1];
      const result = [];
      for (let index = 0; index < data.length; index += 1) {
        const repeat = counts[index % counts.length] ?? 0;
        for (let iteration = 0; iteration < repeat; iteration += 1) {
          result.push(data[index]);
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['932b9817-fcc6-4ac3-b5fd-c0e8eeadc53f']),
    'Cull Nth',
    'cull nth',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        list: 'list',
        N: 'frequency',
        'Cull frequency': 'frequency',
        frequency: 'frequency',
      },
      outputs: {
        L: 'result',
        List: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const list = toList(inputs.list);
      const frequency = parseCount(inputs.frequency, 0);
      if (!list.length || frequency <= 0) {
        return { result: list.slice() };
      }
      if (frequency === 1) {
        return { result: [] };
      }
      const result = list.filter((_, index) => ((index + 1) % frequency) !== 0);
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['9445ca40-cc73-4861-a455-146308676855']),
    'Range',
    'range',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'domain',
        Domain: 'domain',
        domain: 'domain',
        R: 'domain',
        Range: 'domain',
        N: 'steps',
        Steps: 'steps',
        steps: 'steps',
      },
      outputs: {
        R: 'range',
        Range: 'range',
        Values: 'range',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureNumericRange(inputs.domain ?? inputs.range, toNumber, 0, 1);
      const steps = parseCount(inputs.steps, 10);
      if (steps <= 0) {
        return { range: [domain.start] };
      }
      const values = [];
      const increment = (domain.end - domain.start) / steps;
      for (let index = 0; index <= steps; index += 1) {
        values.push(domain.start + increment * index);
      }
      return { range: values };
    },
  });

  register([
    ...GUID_KEYS(['a12dddbf-bb49-4ef4-aeb8-5653bc882cbd']),
    'RandomEx',
    'randomex',
    'random ex',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L0: 'min',
        Min: 'min',
        min: 'min',
        Lower: 'min',
        L1: 'max',
        Max: 'max',
        max: 'max',
        Upper: 'max',
        N: 'count',
        Count: 'count',
        count: 'count',
        S: 'seed',
        Seed: 'seed',
        seed: 'seed',
      },
      outputs: {
        V: 'values',
        Values: 'values',
        Result: 'values',
      },
    },
    eval: ({ inputs }) => {
      const min = parseNumber(inputs.min ?? inputs.lower ?? inputs.l0, 0);
      const max = parseNumber(inputs.max ?? inputs.upper ?? inputs.l1, 1);
      const lower = Math.min(min, max);
      const upper = Math.max(min, max);
      const count = parseCount(inputs.count, 10);
      const rng = createSeededRandom(inputs.seed, toNumber);
      const values = [];
      if (count <= 0) {
        return { values };
      }
      if (upper === lower) {
        for (let index = 0; index < count; index += 1) {
          values.push(lower);
        }
        return { values };
      }
      const delta = upper - lower;
      for (let index = 0; index < count; index += 1) {
        values.push(lower + rng() * delta);
      }
      return { values };
    },
  });

  register([
    ...GUID_KEYS(['c40dc145-9e36-4a69-ac1a-6d825c654993']),
    'Repeat Data',
    'repeat data',
    'repeat',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'data',
        Data: 'data',
        data: 'data',
        L: 'length',
        Length: 'length',
        length: 'length',
      },
      outputs: {
        D: 'result',
        Data: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const data = toList(inputs.data);
      const targetLength = parseCount(inputs.length, data.length || 0);
      if (!targetLength) {
        return { result: [] };
      }
      if (!data.length) {
        return { result: [] };
      }
      const result = [];
      for (let index = 0; index < targetLength; index += 1) {
        result.push(data[index % data.length]);
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['dd8134c0-109b-4012-92be-51d843edfff7']),
    'Duplicate Data',
    'duplicate data',
    'dup data',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'data',
        Data: 'data',
        data: 'data',
        N: 'count',
        Number: 'count',
        count: 'count',
        O: 'order',
        Order: 'order',
        order: 'order',
      },
      outputs: {
        D: 'result',
        Data: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const data = toList(inputs.data);
      const duplicates = parseCount(inputs.count, 1);
      if (!data.length || duplicates <= 0) {
        return { result: [] };
      }
      const keepOrder = toBoolean(inputs.order, true);
      const result = [];
      if (keepOrder) {
        for (let iteration = 0; iteration < duplicates; iteration += 1) {
          result.push(...data);
        }
      } else {
        for (const item of data) {
          for (let iteration = 0; iteration < duplicates; iteration += 1) {
            result.push(item);
          }
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['e64c5fb1-845c-4ab1-8911-5f338516ba67']),
    'Series',
    'series',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        S: 'start',
        Start: 'start',
        start: 'start',
        N: 'step',
        Step: 'step',
        step: 'step',
        C: 'count',
        Count: 'count',
        count: 'count',
      },
      outputs: {
        S: 'series',
        Series: 'series',
        Result: 'series',
      },
    },
    eval: ({ inputs }) => {
      const start = parseNumber(inputs.start ?? inputs.s, 0);
      const step = parseNumber(inputs.step ?? inputs.n, 1);
      const count = parseCount(inputs.count, 10);
      const series = [];
      for (let index = 0; index < count; index += 1) {
        series.push(start + step * index);
      }
      return { series };
    },
  });

  register([
    ...GUID_KEYS(['e6e344aa-f45b-43d5-a2d9-9cf8e8e608dc']),
    'Duplicate data [OBSOLETE]',
    'duplicate data obsolete',
    'dup obsolete',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'data',
        Data: 'data',
        data: 'data',
        N: 'count',
        Number: 'count',
        count: 'count',
      },
      outputs: {
        D: 'result',
        Data: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const data = toList(inputs.data);
      const duplicates = parseCount(inputs.count, 1);
      if (!data.length || duplicates <= 0) {
        return { result: [] };
      }
      const result = [];
      for (const item of data) {
        for (let iteration = 0; iteration < duplicates; iteration += 1) {
          result.push(item);
        }
      }
      return { result };
    },
  });

  register([
    ...GUID_KEYS(['e9b2d2a6-0377-4c1c-a89e-b3f219a95b4d']),
    'Sequence',
    'sequence',
    'seq',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        N: 'notation',
        Notation: 'notation',
        notation: 'notation',
        L: 'length',
        Length: 'length',
        length: 'length',
        I: 'initial',
        Initial: 'initial',
        initial: 'initial',
      },
      outputs: {
        S: 'sequence',
        Sequence: 'sequence',
      },
    },
    eval: ({ inputs }) => {
      const initialValues = toList(inputs.initial);
      const normalizedSeeds = initialValues.length
        ? initialValues.map((value) => {
            const numeric = toNumber(value, Number.NaN);
            return Number.isFinite(numeric) ? numeric : value;
          })
        : [0];
      const length = parseCount(inputs.length, normalizedSeeds.length || 1);
      if (length <= 0) {
        return { sequence: [] };
      }
      const evaluator = compileSequenceExpression(inputs.notation);
      const sequence = [];
      for (let index = 0; index < normalizedSeeds.length && sequence.length < length; index += 1) {
        sequence.push(normalizedSeeds[index]);
      }
      const seeds = sequence.length ? sequence.slice() : [0];
      if (evaluator) {
        while (sequence.length < length) {
          const index = sequence.length;
          const prev = index > 0 ? sequence[index - 1] : seeds[seeds.length - 1] ?? 0;
          const prev1 = index > 1 ? sequence[index - 2] : prev;
          const prev2 = index > 2 ? sequence[index - 3] : prev1;
          let next;
          try {
            next = evaluator(index + 1, index, prev, prev1, prev2, sequence, seeds);
          } catch (error) {
            next = prev;
          }
          const numeric = toNumber(next, Number.NaN);
          sequence.push(Number.isFinite(numeric) ? numeric : next);
        }
      } else {
        const pattern = seeds.length ? seeds : [0];
        while (sequence.length < length) {
          sequence.push(pattern[sequence.length % pattern.length]);
        }
      }
      if (sequence.length > length) {
        sequence.length = length;
      }
      return { sequence };
    },
  });

  register([
    ...GUID_KEYS(['f02a20f6-bb49-4e3d-b155-8ed5d3c6b000']),
    'Jitter',
    'jitter',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        L: 'list',
        List: 'list',
        list: 'list',
        J: 'strength',
        Jitter: 'strength',
        jitter: 'strength',
        S: 'seed',
        Seed: 'seed',
        seed: 'seed',
      },
      outputs: {
        V: 'values',
        Values: 'values',
        I: 'indices',
        Indices: 'indices',
      },
    },
    eval: ({ inputs }) => {
      const values = toList(inputs.list);
      const indices = values.map((_, index) => index);
      const strength = Math.min(1, Math.max(0, parseNumber(inputs.strength, 1)));
      if (values.length <= 1 || strength <= 0) {
        return { values, indices };
      }
      const rng = createSeededRandom(inputs.seed, toNumber);
      if (strength >= 1) {
        for (let index = values.length - 1; index > 0; index -= 1) {
          const swapIndex = Math.floor(rng() * (index + 1));
          const tempValue = values[index];
          values[index] = values[swapIndex];
          values[swapIndex] = tempValue;
          const tempIndex = indices[index];
          indices[index] = indices[swapIndex];
          indices[swapIndex] = tempIndex;
        }
        return { values, indices };
      }
      const swapCount = Math.max(1, Math.round(strength * values.length));
      for (let iteration = 0; iteration < swapCount; iteration += 1) {
        const first = Math.floor(rng() * values.length);
        let second = Math.floor(rng() * values.length);
        if (values.length > 1) {
          let attempts = 0;
          while (second === first && attempts < 5) {
            second = Math.floor(rng() * values.length);
            attempts += 1;
          }
          if (second === first) {
            continue;
          }
        }
        const tempValue = values[first];
        values[first] = values[second];
        values[second] = tempValue;
        const tempIndex = indices[first];
        indices[first] = indices[second];
        indices[second] = tempIndex;
      }
      return { values, indices };
    },
  });

  register([
    ...GUID_KEYS(['fbcf0d42-c9a5-4ca5-8d5b-567fb54abc43']),
    'Split [OBSOLETE]',
    'split obsolete',
    'split',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        D: 'data',
        Data: 'data',
        data: 'data',
        B: 'flag',
        Boolean: 'flag',
        flag: 'flag',
      },
      outputs: {
        F: 'falseResult',
        False: 'falseResult',
        T: 'trueResult',
        True: 'trueResult',
      },
    },
    eval: ({ inputs }) => {
      const data = toList(inputs.data);
      const flag = toBoolean(inputs.flag, false);
      if (flag) {
        return { trueResult: data, falseResult: [] };
      }
      return { trueResult: [], falseResult: data };
    },
  });

  register([
    ...GUID_KEYS(['fe99f302-3d0d-4389-8494-bd53f7935a02']),
    'Fibonacci',
    'fibonacci',
    'fib',
  ], {
    type: 'sets:sequence',
    pinMap: {
      inputs: {
        A: 'seedA',
        'Seed A': 'seedA',
        seedA: 'seedA',
        B: 'seedB',
        'Seed B': 'seedB',
        seedB: 'seedB',
        N: 'count',
        Number: 'count',
        count: 'count',
      },
      outputs: {
        S: 'series',
        Series: 'series',
      },
    },
    eval: ({ inputs }) => {
      const seedA = parseNumber(inputs.seedA ?? inputs.a, 0);
      const seedB = parseNumber(inputs.seedB ?? inputs.b, 1);
      const count = parseCount(inputs.count, 10);
      const series = [];
      if (count <= 0) {
        return { series };
      }
      series.push(seedA);
      if (count > 1) {
        series.push(seedB);
      }
      for (let index = 2; index < count; index += 1) {
        series.push(parseNumber(series[index - 1], 0) + parseNumber(series[index - 2], 0));
      }
      return { series };
    },
  });
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

export function registerSetsTreeComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const mergeTrees = (trees = []) => {
    const branches = [];
    for (const tree of trees) {
      const normalized = normalizeTreeBranches(tree, toNumber);
      for (const branch of normalized) {
        branches.push(cloneBranch(branch));
      }
    }
    return createTreeFromBranches(mergeBranches(branches));
  };

  const registerSimplifyTree = (guid, frontOnly) => {
    register([
      ...GUID_KEYS([guid]),
      'Simplify Tree',
      'simplify tree',
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: { T: 'tree', Tree: 'tree', tree: 'tree', F: 'front', Front: 'front' },
        outputs: { T: 'tree', Tree: 'tree' },
      },
      eval: ({ inputs }) => {
        const branches = normalizeTreeBranches(inputs.tree, toNumber);
        const simplified = mergeBranches(
          simplifyBranches(branches, { frontOnly: frontOnly || toBoolean(inputs.front, false) })
        );
        return { tree: createTreeFromBranches(simplified) };
      },
    });
  };

  registerSimplifyTree('06b3086c-1e9d-41c2-bcfc-bb843156196e', false);
  registerSimplifyTree('1303da7b-e339-4e65-a051-82c4dce8224d', true);

  register([
    ...GUID_KEYS(['071c3940-a12d-4b77-bb23-42b5d3314a0d']),
    'Clean Tree',
    'clean tree',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        N: 'removeNulls',
        'Remove Nulls': 'removeNulls',
        X: 'removeInvalid',
        'Remove Invalid': 'removeInvalid',
        E: 'removeEmpty',
        'Remove Empty': 'removeEmpty',
        T: 'tree',
        Tree: 'tree',
      },
      outputs: { T: 'tree', Tree: 'tree' },
    },
    eval: ({ inputs }) => {
      const removeNulls = toBoolean(inputs.removeNulls, true);
      const removeInvalid = toBoolean(inputs.removeInvalid, true);
      const removeEmpty = toBoolean(inputs.removeEmpty, true);
      const branches = [];
      for (const branch of normalizeTreeBranches(inputs.tree, toNumber)) {
        const values = [];
        for (const value of ensureArray(branch.values)) {
          const nullLike = removeNulls && isNullLike(value);
          const invalid = removeInvalid && isInvalidValue(value);
          if (nullLike || invalid) {
            continue;
          }
          values.push(value);
        }
        if (!values.length && removeEmpty) {
          continue;
        }
        branches.push({ path: branch.path.slice(), values });
      }
      return { tree: createTreeFromBranches(mergeBranches(branches)) };
    },
  });

  register([
    ...GUID_KEYS(['7991bc5f-8a01-4768-bfb0-a39357ac6b84']),
    'Clean Tree',
    'clean tree',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        T: 'tree',
        Tree: 'tree',
        X: 'removeInvalid',
        'Clean Invalid': 'removeInvalid',
        E: 'removeEmpty',
        'Clean Empty': 'removeEmpty',
      },
      outputs: { T: 'tree', Tree: 'tree' },
    },
    eval: ({ inputs }) => {
      const removeInvalid = toBoolean(inputs.removeInvalid, true);
      const removeEmpty = toBoolean(inputs.removeEmpty, true);
      const cleaned = [];
      for (const branch of normalizeTreeBranches(inputs.tree, toNumber)) {
        const values = [];
        for (const value of ensureArray(branch.values)) {
          if (removeInvalid && isInvalidValue(value)) {
            continue;
          }
          values.push(value);
        }
        if (!values.length && removeEmpty) {
          continue;
        }
        cleaned.push({ path: branch.path.slice(), values });
      }
      return { tree: createTreeFromBranches(mergeBranches(cleaned)) };
    },
  });

  register([
    ...GUID_KEYS(['70ce4230-da08-4fce-b29d-63dc42a88585']),
    'Clean Tree',
    'clean tree',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: { T: 'tree', Tree: 'tree', X: 'removeInvalid', Invalid: 'removeInvalid' },
      outputs: { D: 'data', Data: 'data' },
    },
    eval: ({ inputs }) => {
      const removeInvalid = toBoolean(inputs.removeInvalid, true);
      const values = [];
      for (const branch of normalizeTreeBranches(inputs.tree, toNumber)) {
        for (const value of ensureArray(branch.values)) {
          if (removeInvalid && isInvalidValue(value)) {
            continue;
          }
          values.push(value);
        }
      }
      return { data: values };
    },
  });

  register([
    ...GUID_KEYS(['0b6c5dac-6c93-4158-b8d1-ca3187d45f25']),
    'Merge Multiple',
    'merge multiple',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        0: 'stream0',
        'Stream 0': 'stream0',
        1: 'stream1',
        'Stream 1': 'stream1',
        2: 'stream2',
        'Stream 2': 'stream2',
        3: 'stream3',
        'Stream 3': 'stream3',
        4: 'stream4',
        'Stream 4': 'stream4',
        5: 'stream5',
        'Stream 5': 'stream5',
        6: 'stream6',
        'Stream 6': 'stream6',
        7: 'stream7',
        'Stream 7': 'stream7',
        8: 'stream8',
        'Stream 8': 'stream8',
        9: 'stream9',
        'Stream 9': 'stream9',
      },
      outputs: { S: 'stream', Stream: 'stream' },
    },
    eval: ({ inputs }) => {
      const trees = gatherIndexedTreeInputs(inputs, /^(?:stream\s*)?(\d+)$/i, toNumber);
      return { stream: mergeTrees(trees.map((branches) => createTreeFromBranches(branches))) };
    },
  });

  const graftBranches = (branches, stripNulls) => {
    const result = [];
    for (const branch of branches) {
      const path = Array.isArray(branch?.path) ? branch.path : [];
      const values = ensureArray(branch?.values);
      for (let index = 0; index < values.length; index += 1) {
        const value = values[index];
        if (stripNulls && isNullLike(value)) {
          continue;
        }
        result.push({ path: [...path, index], values: [value] });
      }
    }
    return result;
  };

  register([
    ...GUID_KEYS(['10a8674b-f4bb-4fdf-a56e-94dc606ecf33']),
    'Graft Tree',
    'graft tree',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: { D: 'tree', Data: 'tree', T: 'tree', S: 'strip', Strip: 'strip' },
      outputs: { T: 'tree', Tree: 'tree' },
    },
    eval: ({ inputs }) => {
      const stripNulls = toBoolean(inputs.strip, false);
      const branches = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber);
      return { tree: createTreeFromBranches(graftBranches(branches, stripNulls)) };
    },
  });

  register([
    ...GUID_KEYS(['87e1d9ef-088b-4d30-9dda-8a7448a17329']),
    'Graft Tree',
    'graft tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree' }, outputs: { T: 'tree', Tree: 'tree' } },
    eval: ({ inputs }) => {
      const branches = normalizeTreeBranches(inputs.tree, toNumber);
      return { tree: createTreeFromBranches(graftBranches(branches, false)) };
    },
  });

  register([
    ...GUID_KEYS(['1177d6ee-3993-4226-9558-52b7fd63e1e3']),
    'Trim Tree',
    'trim tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', D: 'depth', Depth: 'depth' }, outputs: { T: 'tree', Tree: 'tree' } },
    eval: ({ inputs }) => {
      const depth = Math.max(0, toInteger(inputs.depth, 1, toNumber));
      const trimmed = trimBranches(normalizeTreeBranches(inputs.tree, toNumber), depth, { fromEnd: true });
      return { tree: createTreeFromBranches(mergeBranches(trimmed)) };
    },
  });

  register([
    ...GUID_KEYS(['1d8b0e2c-e772-4fa9-b7f7-b158251b34b8']),
    'Path Compare',
    'path compare',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { P: 'path', Path: 'path', M: 'mask', Mask: 'mask' }, outputs: { C: 'comparison', Comparison: 'comparison' } },
    eval: ({ inputs }) => {
      const path = parsePathInput(inputs.path, toNumber);
      const masks = parseMaskList(inputs.mask, toNumber);
      const match = !masks.length || masks.some((mask) => matchPathWithPattern(path, mask));
      return { comparison: match ? 'Match' : 'Mismatch' };
    },
  });

  const registerMergeByLetters = (guid, label, count, outputName = 'stream') => {
    const letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
    const pinInputs = {};
    for (let index = 0; index < count; index += 1) {
      const letter = letters[index];
      pinInputs[letter] = `stream${letter}`;
      pinInputs[`Stream ${letter}`] = `stream${letter}`;
    }
    register([
      ...GUID_KEYS([guid]),
      label,
      label.toLowerCase(),
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: pinInputs,
        outputs: { S: outputName, Stream: outputName, R: outputName, Result: outputName },
      },
      eval: ({ inputs }) => {
        const trees = [];
        for (let index = 0; index < count; index += 1) {
          const letter = letters[index];
          const value = inputs[`stream${letter}`];
          if (value !== undefined) {
            trees.push(value);
          }
        }
        return { [outputName]: mergeTrees(trees) };
      },
    });
  };

  registerMergeByLetters('22f66ff6-d281-453c-bd8c-36ed24026783', 'Merge 10', 10);
  registerMergeByLetters('481f0339-1299-43ba-b15c-c07891a8f822', 'Merge 03', 3);
  registerMergeByLetters('86866576-6cc0-485a-9cd2-6f7d493f57f7', 'Merge', 2);
  registerMergeByLetters('a70aa477-0109-4e75-ba73-78725dca0274', 'Merge 08', 8);
  registerMergeByLetters('ac9b4faf-c9d5-4f6a-a5e9-58c0c2cac116', 'Merge 06', 6);
  registerMergeByLetters('b5be5d1f-717f-493c-b958-816957f271fd', 'Merge 04', 4);
  registerMergeByLetters('f4b0f7b4-5a10-46c4-8191-58d7d66ffdff', 'Merge 05', 5);

  register([
    ...GUID_KEYS(['2d61f4e0-47c5-41d6-a41d-6afa96ee63af']),
    'Shift Paths',
    'shift paths',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { D: 'tree', Data: 'tree', T: 'tree', O: 'offset', Offset: 'offset' }, outputs: { D: 'tree', Data: 'tree' } },
    eval: ({ inputs }) => {
      const offset = toInteger(inputs.offset, 0, toNumber);
      const shifted = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber).map((branch) => ({
        path: Array.isArray(branch?.path) ? branch.path.map((segment) => segment + offset) : [],
        values: ensureArray(branch?.values),
      }));
      return { tree: createTreeFromBranches(mergeBranches(shifted)) };
    },
  });

  register([
    ...GUID_KEYS(['3a710c1e-1809-4e19-8c15-82adce31cd62']),
    'Tree Branch',
    'tree branch',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', P: 'path', Path: 'path' }, outputs: { B: 'branch', Branch: 'branch' } },
    eval: ({ inputs }) => {
      const treeBranches = normalizeTreeBranches(inputs.tree, toNumber);
      const targetPath = parsePathInput(inputs.path, toNumber);
      const branch = findBranchByPath(treeBranches, targetPath);
      if (!branch) {
        return { branch: createTreeFromBranches([]) };
      }
      return { branch: createTreeFromBranches([{ path: branch.path.slice(), values: ensureArray(branch.values) }]) };
    },
  });

  register([
    ...GUID_KEYS(['3cadddef-1e2b-4c09-9390-0e8f78f7609f']),
    'Merge',
    'merge tree',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: { D1: 'streamA', 'Data 1': 'streamA', D2: 'streamB', 'Data 2': 'streamB' },
      outputs: { R: 'result', Result: 'result', S: 'result', Stream: 'result' },
    },
    eval: ({ inputs }) => {
      return { result: mergeTrees([inputs.streamA, inputs.streamB]) };
    },
  });

  const registerStreamFilter = (guid) => {
    register([
      ...GUID_KEYS([guid]),
      'Stream Filter',
      'stream filter',
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: { G: 'gate', Gate: 'gate', 0: 'stream0', 'Stream 0': 'stream0', 1: 'stream1', 'Stream 1': 'stream1' },
        outputs: { S: 'stream', Stream: 'stream' },
      },
      eval: ({ inputs }) => {
        const gateIndex = toInteger(inputs.gate, 0, toNumber);
        const streams = gatherIndexedTreeInputs(inputs, /^(?:stream\s*)?(\d+)$/i, toNumber);
        const index = wrapIndex(gateIndex, streams.length || 1);
        const selected = streams.length ? streams[index] : [];
        return { stream: createTreeFromBranches(selected) };
      },
    });
  };

  registerStreamFilter('3e5582a1-901a-4f7c-b58d-f5d7e3166124');
  registerStreamFilter('eeafc956-268e-461d-8e73-ee05c6f72c01');

  register([
    ...GUID_KEYS(['41aa4112-9c9b-42f4-847e-503b9d90e4c7']),
    'Flip Matrix',
    'flip matrix',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { D: 'tree', Data: 'tree', T: 'tree' }, outputs: { D: 'tree', Data: 'tree' } },
    eval: ({ inputs }) => {
      const branches = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber);
      const maxColumns = Math.max(0, ...branches.map((branch) => ensureArray(branch.values).length));
      const columns = [];
      for (let columnIndex = 0; columnIndex < maxColumns; columnIndex += 1) {
        const values = [];
        for (const branch of branches) {
          const branchValues = ensureArray(branch.values);
          values.push(columnIndex < branchValues.length ? branchValues[columnIndex] : null);
        }
        columns.push({ path: [columnIndex], values });
      }
      return { tree: createTreeFromBranches(columns) };
    },
  });

  register([
    ...GUID_KEYS(['46372d0d-82dc-4acb-adc3-25d1fde04c4e']),
    'Match Tree',
    'match tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', G: 'guide', Guide: 'guide' }, outputs: { T: 'tree', Tree: 'tree' } },
    eval: ({ inputs }) => {
      const values = collectTreeValues(inputs.tree, toNumber);
      const guideBranches = sortBranches(normalizeTreeBranches(inputs.guide, toNumber));
      const result = [];
      let cursor = 0;
      for (const branch of guideBranches) {
        const count = ensureArray(branch.values).length || 1;
        const branchValues = [];
        for (let index = 0; index < count; index += 1) {
          branchValues.push(cursor < values.length ? values[cursor] : null);
          cursor += 1;
        }
        result.push({ path: branch.path.slice(), values: branchValues });
      }
      return { tree: createTreeFromBranches(result) };
    },
  });

  const resolveRelativeItems = (treeAInput, treeBInput, offsetValue, wrapPathsValue, wrapItemsValue) => {
    const branchesA = sortBranches(normalizeTreeBranches(treeAInput, toNumber));
    const branchesB = sortBranches(normalizeTreeBranches(treeBInput, toNumber));
    const wrapPaths = toBoolean(wrapPathsValue, false);
    const wrapItems = toBoolean(wrapItemsValue, false);
    const offset = toInteger(offsetValue, 0, toNumber);
    const resultA = [];
    const resultB = [];
    const branchKeys = branchesB.map((branch) => formatPathKey(branch.path));
    for (let index = 0; index < branchesA.length; index += 1) {
      const branchA = branchesA[index];
      const valuesA = ensureArray(branchA.values);
      let branchB = findBranchByPath(branchesB, branchA.path);
      if (!branchB && wrapPaths && branchesB.length) {
        const wrappedIndex = wrapIndex(index, branchesB.length);
        branchB = branchesB[wrappedIndex];
      }
      const valuesB = ensureArray(branchB?.values);
      const pairedA = [];
      const pairedB = [];
      for (let itemIndex = 0; itemIndex < valuesA.length; itemIndex += 1) {
        const valueA = valuesA[itemIndex];
        let targetIndex = itemIndex + offset;
        if (wrapItems && valuesB.length) {
          targetIndex = wrapIndex(targetIndex, valuesB.length);
        }
        const valueB = targetIndex >= 0 && targetIndex < valuesB.length ? valuesB[targetIndex] : null;
        pairedA.push(valueA);
        pairedB.push(valueB);
      }
      resultA.push({ path: branchA.path.slice(), values: pairedA });
      resultB.push({ path: branchA.path.slice(), values: pairedB });
    }
    return { resultA, resultB, branchKeys };
  };

  register([
    ...GUID_KEYS(['2653b135-4df1-4a6b-820c-55e2ad3bc1e0']),
    'Relative Items',
    'relative items',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        A: 'treeA',
        'Tree A': 'treeA',
        B: 'treeB',
        'Tree B': 'treeB',
        O: 'offset',
        Offset: 'offset',
        Wp: 'wrapPaths',
        'Wrap Paths': 'wrapPaths',
        Wi: 'wrapItems',
        'Wrap Items': 'wrapItems',
      },
      outputs: { A: 'resultA', 'Item A': 'resultA', B: 'resultB', 'Item B': 'resultB' },
    },
    eval: ({ inputs }) => {
      const { resultA, resultB } = resolveRelativeItems(
        inputs.treeA,
        inputs.treeB,
        inputs.offset,
        inputs.wrapPaths,
        inputs.wrapItems
      );
      return {
        resultA: createTreeFromBranches(resultA),
        resultB: createTreeFromBranches(resultB),
      };
    },
  });

  register([
    ...GUID_KEYS(['fac0d5be-e3ff-4bbb-9742-ec9a54900d41']),
    'Relative Item',
    'relative item',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        T: 'tree',
        Tree: 'tree',
        O: 'offset',
        Offset: 'offset',
        Wp: 'wrapPaths',
        'Wrap Paths': 'wrapPaths',
        Wi: 'wrapItems',
        'Wrap Items': 'wrapItems',
      },
      outputs: { A: 'resultA', 'Item A': 'resultA', B: 'resultB', 'Item B': 'resultB' },
    },
    eval: ({ inputs }) => {
      const { resultA, resultB } = resolveRelativeItems(
        inputs.tree,
        inputs.tree,
        inputs.offset,
        inputs.wrapPaths,
        inputs.wrapItems
      );
      return {
        resultA: createTreeFromBranches(resultA),
        resultB: createTreeFromBranches(resultB),
      };
    },
  });

  const registerStreamGate = (guid, outputAsTree) => {
    register([
      ...GUID_KEYS([guid]),
      'Stream Gate',
      'stream gate',
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: { S: 'stream', Stream: 'stream', G: 'gate', Gate: 'gate' },
        outputs: outputAsTree
          ? { 0: 'target0', 'Target 0': 'target0', 1: 'target1', 'Target 1': 'target1' }
          : { 0: 'target0', 'Target 0': 'target0', 1: 'target1', 'Target 1': 'target1' },
      },
      eval: ({ inputs }) => {
        const gate = toBoolean(inputs.gate, false);
        const branches = normalizeTreeBranches(inputs.stream, toNumber);
        if (outputAsTree) {
          return {
            target0: gate ? createTreeFromBranches([]) : createTreeFromBranches(branches),
            target1: gate ? createTreeFromBranches(branches) : createTreeFromBranches([]),
          };
        }
        const values = flattenBranchValues(branches);
        return {
          target0: gate ? [] : values,
          target1: gate ? values : [],
        };
      },
    });
  };

  registerStreamGate('71fcc052-6add-4d70-8d97-cfb37ea9d169', true);
  registerStreamGate('d6313940-216b-487f-b511-6c8a5b87eae7', false);

  const registerExplodeTree = (guid, outputNames) => {
    register([
      ...GUID_KEYS([guid]),
      'Explode Tree',
      'explode tree',
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: { D: 'tree', Data: 'tree', T: 'tree' },
        outputs: outputNames.reduce((map, name, index) => {
          map[name.gh] = name.key;
          return map;
        }, {}),
      },
      eval: ({ inputs }) => {
        const branches = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber);
        const result = {};
        for (let index = 0; index < outputNames.length; index += 1) {
          const branch = branches[index];
          result[outputNames[index].key] = branch
            ? createTreeFromBranches([{ path: branch.path.slice(), values: ensureArray(branch.values) }])
            : createTreeFromBranches([]);
        }
        return result;
      },
    });
  };

  registerExplodeTree('74cad441-2264-45fe-a57d-85034751208a', [
    { gh: '-', key: 'branch0' },
    { gh: '-', key: 'branch1' },
  ]);

  registerExplodeTree('8a470a35-d673-4779-a65e-ba95765e59e4', [
    { gh: '0', key: 'branch0' },
    { gh: '1', key: 'branch1' },
  ]);

  register([
    ...GUID_KEYS(['946cb61e-18d2-45e3-8840-67b0efa26528']),
    'Construct Path',
    'construct path',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { I: 'indices', Indices: 'indices' }, outputs: { B: 'branch', Branch: 'branch' } },
    eval: ({ inputs }) => {
      const path = parsePathInput(inputs.indices, toNumber);
      return { branch: { path, text: toPathString(path) } };
    },
  });

  register([
    ...GUID_KEYS(['df6d9197-9a6e-41a2-9c9d-d2221accb49e']),
    'Deconstruct Path',
    'deconstruct path',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { B: 'branch', Branch: 'branch' }, outputs: { I: 'indices', Indices: 'indices' } },
    eval: ({ inputs }) => {
      const indices = parsePathInput(inputs.branch, toNumber);
      return { indices };
    },
  });

  register([
    ...GUID_KEYS(['99bee19d-588c-41a0-b9b9-1d00fb03ea1a']),
    'Tree Statistics',
    'tree statistics',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree' }, outputs: { P: 'paths', Paths: 'paths', L: 'lengths', Length: 'lengths', C: 'count', Count: 'count' } },
    eval: ({ inputs }) => {
      const branches = sortBranches(normalizeTreeBranches(inputs.tree, toNumber));
      const paths = [];
      const lengths = [];
      let count = 0;
      for (const branch of branches) {
        const values = ensureArray(branch.values);
        paths.push(toPathString(branch.path));
        lengths.push(values.length);
        count += values.length;
      }
      return { paths, lengths, count };
    },
  });

  const registerFlattenTree = (guid, includePathInput) => {
    register([
      ...GUID_KEYS([guid]),
      'Flatten Tree',
      'flatten tree',
    ], {
      type: 'sets:tree',
      pinMap: {
        inputs: includePathInput
          ? { T: 'tree', Tree: 'tree', P: 'path', Path: 'path' }
          : { D: 'tree', Data: 'tree', T: 'tree' },
        outputs: { T: 'tree', Tree: 'tree', D: 'tree', Data: 'tree' },
      },
      eval: ({ inputs }) => {
        const treeValue = includePathInput ? inputs.tree : inputs.tree ?? inputs.data;
        const path = includePathInput ? parsePathInput(inputs.path, toNumber) : [];
        const flattened = flattenTreeToPath(normalizeTreeBranches(treeValue, toNumber), path);
        return { tree: createTreeFromBranches(flattened) };
      },
    });
  };

  registerFlattenTree('a13fcd5d-81af-4337-a32e-28dd7e23ae4c', false);
  registerFlattenTree('f80cfe18-9510-4b89-8301-8e58faf423bb', true);

  register([
    ...GUID_KEYS(['b8e2aa8f-8830-4ee1-bb59-613ea279c281']),
    'Unflatten Tree',
    'unflatten tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', G: 'guide', Guide: 'guide' }, outputs: { T: 'tree', Tree: 'tree' } },
    eval: ({ inputs }) => {
      const values = collectTreeValues(inputs.tree, toNumber);
      const guideBranches = sortBranches(normalizeTreeBranches(inputs.guide, toNumber));
      const result = [];
      let cursor = 0;
      for (const branch of guideBranches) {
        const templateValues = ensureArray(branch.values);
        const count = templateValues.length || 1;
        const branchValues = [];
        for (let index = 0; index < count; index += 1) {
          branchValues.push(cursor < values.length ? values[cursor] : null);
          cursor += 1;
        }
        result.push({ path: branch.path.slice(), values: branchValues });
      }
      return { tree: createTreeFromBranches(result) };
    },
  });

  register([
    ...GUID_KEYS(['bfaaf799-77dc-4f31-9ad8-2f7d1a80aeb0']),
    'Replace Paths',
    'replace paths',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { D: 'tree', Data: 'tree', S: 'search', Search: 'search', R: 'replace', Replace: 'replace' }, outputs: { D: 'tree', Data: 'tree' } },
    eval: ({ inputs }) => {
      const searchList = ensureArray(inputs.search).map((entry) => parsePathInput(entry, toNumber));
      const replaceList = ensureArray(inputs.replace).map((entry) => parsePathInput(entry, toNumber));
      const replacements = new Map();
      for (let index = 0; index < searchList.length; index += 1) {
        const key = formatPathKey(searchList[index]);
        const replacement = replaceList[index] ?? replaceList[replaceList.length - 1] ?? [];
        replacements.set(key, replacement);
      }
      const branches = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber).map((branch) => {
        const key = formatPathKey(branch.path);
        if (replacements.has(key)) {
          return { path: replacements.get(key).slice(), values: ensureArray(branch.values) };
        }
        return { path: branch.path.slice(), values: ensureArray(branch.values) };
      });
      return { tree: createTreeFromBranches(mergeBranches(branches)) };
    },
  });

  register([
    ...GUID_KEYS(['c1ec65a3-bda4-4fad-87d0-edf86ed9d81c']),
    'Tree Item',
    'tree item',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', P: 'path', Path: 'path', i: 'index', I: 'index', Index: 'index', W: 'wrap', Wrap: 'wrap' }, outputs: { E: 'element', Element: 'element' } },
    eval: ({ inputs }) => {
      const branches = normalizeTreeBranches(inputs.tree, toNumber);
      const path = parsePathInput(inputs.path, toNumber);
      const branch = findBranchByPath(branches, path);
      if (!branch) {
        return { element: null };
      }
      const values = ensureArray(branch.values);
      if (!values.length) {
        return { element: null };
      }
      const wrap = toBoolean(inputs.wrap, false);
      const indices = ensureArray(inputs.index).map((value) => toInteger(value, 0, toNumber));
      const resolved = indices.length ? indices : [0];
      const results = resolved.map((candidate) => {
        let index = candidate;
        if (wrap) {
          index = wrapIndex(index, values.length);
        }
        if (index < 0 || index >= values.length) {
          return null;
        }
        return values[index];
      });
      return { element: results.length <= 1 ? results[0] ?? null : results };
    },
  });

  register([
    ...GUID_KEYS(['c9785b8e-2f30-4f90-8ee3-cca710f82402']),
    'Entwine',
    'entwine',
  ], {
    type: 'sets:tree',
    pinMap: {
      inputs: {
        '{0;0}': 'branch_0_0',
        'Branch {0;0}': 'branch_0_0',
        '{0;1}': 'branch_0_1',
        'Branch {0;1}': 'branch_0_1',
        '{0;2}': 'branch_0_2',
        'Branch {0;2}': 'branch_0_2',
        '{0;3}': 'branch_0_3',
        'Branch {0;3}': 'branch_0_3',
        '{0;4}': 'branch_0_4',
        'Branch {0;4}': 'branch_0_4',
        '{0;5}': 'branch_0_5',
        'Branch {0;5}': 'branch_0_5',
      },
      outputs: { R: 'result', Result: 'result' },
    },
    eval: ({ inputs }) => {
      const branches = [];
      for (const [key, value] of Object.entries(inputs)) {
        if (!key.startsWith('branch_')) continue;
        const suffix = key.slice('branch_'.length);
        const segments = suffix.split('_').map((segment) => Number.parseInt(segment, 10)).filter(Number.isFinite);
        const prefixPath = segments;
        for (const branch of normalizeTreeBranches(value, toNumber)) {
          branches.push({ path: [...prefixPath, ...branch.path], values: ensureArray(branch.values) });
        }
      }
      return { result: createTreeFromBranches(mergeBranches(branches)) };
    },
  });

  register([
    ...GUID_KEYS(['d8b1e7ac-cd31-4748-b262-e07e53068afc']),
    'Split Tree',
    'split tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { D: 'tree', Data: 'tree', T: 'tree', M: 'masks', Masks: 'masks' }, outputs: { P: 'positive', Positive: 'positive', N: 'negative', Negative: 'negative' } },
    eval: ({ inputs }) => {
      const branches = normalizeTreeBranches(inputs.tree ?? inputs.data, toNumber);
      const masks = parseMaskList(inputs.masks, toNumber);
      if (!masks.length) {
        return {
          positive: createTreeFromBranches(branches),
          negative: createTreeFromBranches([]),
        };
      }
      const positives = [];
      const negatives = [];
      for (const branch of branches) {
        const target = masks.some((mask) => matchPathWithPattern(branch.path, mask)) ? positives : negatives;
        target.push({ path: branch.path.slice(), values: ensureArray(branch.values) });
      }
      return {
        positive: createTreeFromBranches(positives),
        negative: createTreeFromBranches(negatives),
      };
    },
  });

  register([
    ...GUID_KEYS(['e6859d1e-2b3d-4704-93ea-32714acae176']),
    'Null Check',
    'null check',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { D: 'tree', Data: 'tree', T: 'tree' }, outputs: { N: 'isNull', Null: 'isNull' } },
    eval: ({ inputs }) => {
      const values = collectTreeValues(inputs.tree ?? inputs.data, toNumber);
      const hasNull = values.some((value) => isNullLike(value));
      return { isNull: hasNull };
    },
  });

  register([
    ...GUID_KEYS(['fe769f85-8900-45dd-ba11-ec9cd6c778c6']),
    'Prune Tree',
    'prune tree',
  ], {
    type: 'sets:tree',
    pinMap: { inputs: { T: 'tree', Tree: 'tree', N0: 'min', Minimum: 'min', N1: 'max', Maximum: 'max' }, outputs: { T: 'tree', Tree: 'tree' } },
    eval: ({ inputs }) => {
      const min = Math.max(0, toInteger(inputs.min, 0, toNumber));
      const maxCandidate = toInteger(inputs.max, 0, toNumber);
      const hasMax = maxCandidate > 0;
      const max = hasMax ? Math.max(min, maxCandidate) : 0;
      const branches = [];
      for (const branch of normalizeTreeBranches(inputs.tree, toNumber)) {
        const values = ensureArray(branch.values);
        if (values.length < min) {
          continue;
        }
        if (hasMax && values.length > max) {
          continue;
        }
        branches.push({ path: branch.path.slice(), values });
      }
      return { tree: createTreeFromBranches(mergeBranches(branches)) };
    },
  });
}

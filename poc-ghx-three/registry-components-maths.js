import * as THREE from 'three';

export function registerMathDomainComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math domain components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register math domain components.');
  }

  function toBoolean(value, fallback = false) {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (Array.isArray(value)) {
      if (!value.length) return fallback;
      return toBoolean(value[0], fallback);
    }
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      if (!normalized) return fallback;
      if (['true', 'yes', '1', 'on'].includes(normalized)) return true;
      if (['false', 'no', '0', 'off'].includes(normalized)) return false;
      return fallback;
    }
    return Boolean(value);
  }

  function createDomain(startValue, endValue) {
    const start = Number(startValue);
    const end = Number(endValue);
    if (!Number.isFinite(start) || !Number.isFinite(end)) {
      return null;
    }
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    const span = end - start;
    const length = max - min;
    const center = (start + end) / 2;
    return { start, end, min, max, span, length, center, dimension: 1 };
  }

  function extractNumericProperty(source, keys) {
    if (!source || typeof source !== 'object') {
      return undefined;
    }
    for (const key of keys) {
      if (Object.prototype.hasOwnProperty.call(source, key)) {
        const numeric = toNumber(source[key], Number.NaN);
        if (Number.isFinite(numeric)) {
          return numeric;
        }
      }
    }
    return undefined;
  }

  function ensureDomain(input) {
    if (input === undefined || input === null) {
      return null;
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        const domain = createDomain(input[0], input[1]);
        if (domain) return domain;
      }
      if (input.length === 1) {
        return ensureDomain(input[0]);
      }
      return null;
    }
    if (typeof input === 'object') {
      if (input.dimension === 2) {
        return null;
      }
      if (input.dimension === 1 && input.start !== undefined && input.end !== undefined) {
        const domain = createDomain(input.start, input.end);
        if (domain) {
          return domain;
        }
      }
      const start = extractNumericProperty(input, ['start', 'Start', 's', 'S', 'a', 'A', 'min', 'Min', 'from', 'From', 't0', 'T0', 'lower', 'Lower']);
      const end = extractNumericProperty(input, ['end', 'End', 'e', 'E', 'b', 'B', 'max', 'Max', 'to', 'To', 't1', 'T1', 'upper', 'Upper']);
      if (start !== undefined && end !== undefined) {
        const domain = createDomain(start, end);
        if (domain) {
          return domain;
        }
      }
      if (typeof input.min !== 'undefined' && typeof input.max !== 'undefined') {
        const domain = createDomain(input.min, input.max);
        if (domain) {
          return domain;
        }
      }
      if (typeof input.t0 !== 'undefined' && typeof input.t1 !== 'undefined') {
        const domain = createDomain(input.t0, input.t1);
        if (domain) {
          return domain;
        }
      }
      if (typeof input.value === 'number') {
        return createDomain(input.value, input.value);
      }
      return null;
    }
    const numeric = toNumber(input, Number.NaN);
    if (Number.isFinite(numeric)) {
      return createDomain(numeric, numeric);
    }
    return null;
  }

  function createDomain2(u, v) {
    const uDomain = ensureDomain(u);
    const vDomain = ensureDomain(v);
    if (!uDomain || !vDomain) {
      return null;
    }
    return { dimension: 2, u: uDomain, v: vDomain };
  }

  function ensureDomain2(input) {
    if (input === undefined || input === null) {
      return null;
    }
    if (Array.isArray(input)) {
      if (input.length >= 4) {
        const domain = createDomain2([input[0], input[1]], [input[2], input[3]]);
        if (domain) return domain;
      }
      if (input.length >= 2) {
        const domain = createDomain2(input[0], input[1]);
        if (domain) return domain;
      }
      if (input.length === 1) {
        return ensureDomain2(input[0]);
      }
      return null;
    }
    if (typeof input === 'object') {
      if (input.dimension === 2 && input.u && input.v) {
        const domain = createDomain2(input.u, input.v);
        if (domain) {
          return domain;
        }
      }
      const u = input.u ?? input.U ?? input.uDomain ?? input.UDomain ?? input['u domain'];
      const v = input.v ?? input.V ?? input.vDomain ?? input.VDomain ?? input['v domain'];
      const composed = createDomain2(u, v);
      if (composed) {
        return composed;
      }
      const u0 = extractNumericProperty(input, ['u0', 'U0', 'umin', 'Umin', 'minu', 'u_min']);
      const u1 = extractNumericProperty(input, ['u1', 'U1', 'umax', 'Umax', 'maxu', 'u_max']);
      const v0 = extractNumericProperty(input, ['v0', 'V0', 'vmin', 'Vmin', 'minv', 'v_min']);
      const v1 = extractNumericProperty(input, ['v1', 'V1', 'vmax', 'Vmax', 'maxv', 'v_max']);
      if (u0 !== undefined && u1 !== undefined && v0 !== undefined && v1 !== undefined) {
        const domain = createDomain2([u0, u1], [v0, v1]);
        if (domain) {
          return domain;
        }
      }
      return null;
    }
    return null;
  }

  function domainDistance(value, domain) {
    if (!domain) return Number.POSITIVE_INFINITY;
    const numeric = toNumber(value, Number.NaN);
    if (!Number.isFinite(numeric)) return Number.POSITIVE_INFINITY;
    if (numeric < domain.min) {
      return domain.min - numeric;
    }
    if (numeric > domain.max) {
      return numeric - domain.max;
    }
    return 0;
  }

  function isValueInDomain(value, domain, { strict = false } = {}) {
    if (!domain) return false;
    const numeric = toNumber(value, Number.NaN);
    if (!Number.isFinite(numeric)) return false;
    if (strict) {
      if (domain.length === 0) return false;
      return numeric > domain.min && numeric < domain.max;
    }
    return numeric >= domain.min && numeric <= domain.max;
  }

  function clampValueToDomain(value, domain) {
    if (!domain) return Number.NaN;
    const numeric = toNumber(value, Number.NaN);
    if (!Number.isFinite(numeric)) {
      return domain.min;
    }
    if (numeric < domain.min) return domain.min;
    if (numeric > domain.max) return domain.max;
    return numeric;
  }

  function remapValue(value, sourceDomain, targetDomain) {
    if (!sourceDomain || !targetDomain) {
      return null;
    }
    const numeric = toNumber(value, Number.NaN);
    if (!Number.isFinite(numeric)) {
      return targetDomain.start;
    }
    const sourceSpan = sourceDomain.end - sourceDomain.start;
    if (sourceSpan === 0) {
      return targetDomain.start;
    }
    const targetSpan = targetDomain.end - targetDomain.start;
    const ratio = (numeric - sourceDomain.start) / sourceSpan;
    return targetDomain.start + ratio * targetSpan;
  }

  function collectNumbers(input) {
    const numbers = [];
    const stack = [input];
    while (stack.length) {
      const current = stack.pop();
      if (current === undefined || current === null) continue;
      if (Array.isArray(current)) {
        for (let i = current.length - 1; i >= 0; i -= 1) {
          stack.push(current[i]);
        }
        continue;
      }
      if (typeof current === 'object') {
        if (typeof current.value !== 'undefined') {
          stack.push(current.value);
        }
        continue;
      }
      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        numbers.push(numeric);
      }
    }
    return numbers;
  }

  function collectCoordinatePairs(input) {
    const pairs = [];
    const stack = [input];
    while (stack.length) {
      const current = stack.pop();
      if (current === undefined || current === null) continue;
      if (Array.isArray(current)) {
        if (current.length >= 2) {
          const x = toNumber(current[0], Number.NaN);
          const y = toNumber(current[1], Number.NaN);
          if (Number.isFinite(x) && Number.isFinite(y)) {
            pairs.push({ x, y });
            continue;
          }
        }
        for (let i = current.length - 1; i >= 0; i -= 1) {
          stack.push(current[i]);
        }
        continue;
      }
      if (typeof current === 'object') {
        if (current.isVector2 || current.isVector3) {
          const x = toNumber(current.x, Number.NaN);
          const y = toNumber(current.y, Number.NaN);
          if (Number.isFinite(x) && Number.isFinite(y)) {
            pairs.push({ x, y });
          }
          continue;
        }
        if (current.point) {
          stack.push(current.point);
          continue;
        }
        if (current.position) {
          stack.push(current.position);
          continue;
        }
        if (typeof current.x !== 'undefined' && typeof current.y !== 'undefined') {
          const x = toNumber(current.x, Number.NaN);
          const y = toNumber(current.y, Number.NaN);
          if (Number.isFinite(x) && Number.isFinite(y)) {
            pairs.push({ x, y });
          }
          continue;
        }
        continue;
      }
    }
    return pairs;
  }

  function computeDomainFromNumbers(numbers) {
    if (!numbers || numbers.length === 0) {
      return null;
    }
    let min = Number.POSITIVE_INFINITY;
    let max = Number.NEGATIVE_INFINITY;
    for (const value of numbers) {
      if (value < min) min = value;
      if (value > max) max = value;
    }
    if (!Number.isFinite(min) || !Number.isFinite(max)) {
      return null;
    }
    return createDomain(min, max);
  }

  function subdivideDomain(domain, count) {
    if (!domain) return [];
    const segments = Math.max(0, Math.floor(toNumber(count, 0)));
    if (!Number.isFinite(segments) || segments <= 0) {
      return [];
    }
    const step = (domain.end - domain.start) / segments;
    const result = [];
    for (let i = 0; i < segments; i += 1) {
      const start = domain.start + step * i;
      const end = i === segments - 1 ? domain.end : domain.start + step * (i + 1);
      result.push(createDomain(start, end));
    }
    return result;
  }

  function subdivideDomain2(domain, uCount, vCount) {
    if (!domain) return [];
    const uSegments = subdivideDomain(domain.u, uCount);
    const vSegments = subdivideDomain(domain.v, vCount);
    if (!uSegments.length || !vSegments.length) {
      return [];
    }
    const result = [];
    for (const u of uSegments) {
      for (const v of vSegments) {
        result.push({ dimension: 2, u, v });
      }
    }
    return result;
  }

  register(['{0b5c7fad-0473-41aa-bf52-d7a861dcaa29}', 'find domain', 'fdom'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        D: 'domains',
        Domains: 'domains',
        domains: 'domains',
        N: 'value',
        Number: 'value',
        value: 'value',
        S: 'strict',
        Strict: 'strict',
      },
      outputs: {
        I: 'index',
        Index: 'index',
        N: 'neighbour',
        Neighbor: 'neighbour',
        Neighbour: 'neighbour',
      },
    },
    eval: ({ inputs }) => {
      const domainsInput = inputs.domains;
      const list = Array.isArray(domainsInput) ? domainsInput : domainsInput !== undefined ? [domainsInput] : [];
      const value = toNumber(inputs.value, Number.NaN);
      const strict = toBoolean(inputs.strict, false);
      if (!Number.isFinite(value) || !list.length) {
        return { index: -1, neighbour: -1 };
      }
      let firstMatch = -1;
      let closestIndex = -1;
      let closestDistance = Number.POSITIVE_INFINITY;
      list.forEach((entry, idx) => {
        const domain = ensureDomain(entry);
        if (!domain) return;
        if (firstMatch === -1 && isValueInDomain(value, domain, { strict })) {
          firstMatch = idx;
        }
        const distance = domainDistance(value, domain);
        if (distance < closestDistance) {
          closestDistance = distance;
          closestIndex = idx;
        }
      });
      return { index: firstMatch, neighbour: closestIndex };
    }
  });

  register(['{2fcc2743-8339-4cdf-a046-a1f17439191d}', 'remap numbers'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        V: 'value',
        Value: 'value',
        value: 'value',
        S: 'source',
        Source: 'source',
        source: 'source',
        T: 'target',
        Target: 'target',
        target: 'target',
      },
      outputs: {
        R: 'mapped',
        Result: 'mapped',
        Mapped: 'mapped',
        C: 'clipped',
        Clipped: 'clipped',
      },
    },
    eval: ({ inputs }) => {
      const sourceDomain = ensureDomain(inputs.source);
      const targetDomain = ensureDomain(inputs.target);
      if (!sourceDomain || !targetDomain) {
        return {};
      }
      const value = toNumber(inputs.value, Number.NaN);
      if (!Number.isFinite(value)) {
        return { mapped: targetDomain.start, clipped: targetDomain.start };
      }
      const mapped = remapValue(value, sourceDomain, targetDomain);
      const clippedSourceValue = clampValueToDomain(value, sourceDomain);
      const clipped = remapValue(clippedSourceValue, sourceDomain, targetDomain);
      return { mapped, clipped };
    }
  });

  register(['{47c30f9d-b685-4d4d-9b20-5b60e48d5af8}', 'dedom2num', 'deconstruct domain²'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
      outputs: {
        U0: 'u0',
        'U min': 'u0',
        U1: 'u1',
        'U max': 'u1',
        V0: 'v0',
        'V min': 'v0',
        V1: 'v1',
        'V max': 'v1',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain2(inputs.domain);
      if (!domain) {
        return {};
      }
      return {
        u0: domain.u.start,
        u1: domain.u.end,
        v0: domain.v.start,
        v1: domain.v.end,
      };
    }
  });

  register(['{75ac008b-1bc2-4edd-b967-667d628b9d24}', 'divide domain²', 'divide'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
        U: 'uCount',
        'U Count': 'uCount',
        V: 'vCount',
        'V Count': 'vCount',
      },
      outputs: {
        S: 'segments',
        Segments: 'segments',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain2(inputs.domain);
      if (!domain) {
        return { segments: [] };
      }
      const uCount = inputs.uCount ?? inputs.U;
      const vCount = inputs.vCount ?? inputs.V;
      const segments = subdivideDomain2(domain, uCount, vCount);
      return { segments };
    }
  });

  register(['{75ef4190-91a2-42d9-a245-32a7162b0384}', 'divide domain', 'div'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
        C: 'count',
        Count: 'count',
        count: 'count',
      },
      outputs: {
        S: 'segments',
        Segments: 'segments',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain(inputs.domain);
      if (!domain) {
        return { segments: [] };
      }
      const segments = subdivideDomain(domain, inputs.count);
      return { segments };
    }
  });

  register(['{825ea536-aebb-41e9-af32-8baeb2ecb590}', 'dedomain', 'deconstruct domain'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
      outputs: {
        S: 'start',
        Start: 'start',
        start: 'start',
        E: 'end',
        End: 'end',
        end: 'end',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain(inputs.domain);
      if (!domain) {
        return {};
      }
      return { start: domain.start, end: domain.end };
    }
  });

  register(['{8555a743-36c1-42b8-abcc-06d9cb94519f}', 'dom²', 'construct domain²'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        U: 'uDomain',
        'Domain U': 'uDomain',
        V: 'vDomain',
        'Domain V': 'vDomain',
      },
      outputs: {
        'I²': 'domain',
        '2D Domain': 'domain',
        domain: 'domain',
      },
    },
    eval: ({ inputs }) => {
      const domain = createDomain2(inputs.uDomain, inputs.vDomain);
      if (!domain) {
        return {};
      }
      return { domain };
    }
  });

  register(['{9083b87f-a98c-4e41-9591-077ae4220b19}', 'dom²num', 'construct domain² numbers'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        U0: 'u0',
        'U min': 'u0',
        U1: 'u1',
        'U max': 'u1',
        V0: 'v0',
        'V min': 'v0',
        V1: 'v1',
        'V max': 'v1',
      },
      outputs: {
        'I²': 'domain',
        '2D Domain': 'domain',
        domain: 'domain',
      },
    },
    eval: ({ inputs }) => {
      const domain = createDomain2([inputs.u0, inputs.u1], [inputs.v0, inputs.v1]);
      if (!domain) {
        return {};
      }
      return { domain };
    }
  });

  register(['{95992b33-89e1-4d36-bd35-2754a11af21e}', 'consec', 'consecutive domains'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        N: 'numbers',
        Numbers: 'numbers',
        numbers: 'numbers',
        A: 'additive',
        Additive: 'additive',
      },
      outputs: {
        D: 'domains',
        Domains: 'domains',
      },
    },
    eval: ({ inputs }) => {
      const values = collectNumbers(inputs.numbers).sort((a, b) => a - b);
      const additive = toBoolean(inputs.additive, false);
      if (!values.length) {
        return { domains: [] };
      }
      if (additive) {
        const domains = [];
        let start = 0;
        for (const length of values) {
          if (!Number.isFinite(length)) continue;
          const end = start + length;
          const domain = createDomain(start, end);
          if (domain) {
            domains.push(domain);
          }
          start = end;
        }
        return { domains };
      }
      const uniqueValues = [...new Set(values)];
      if (uniqueValues.length < 2) {
        return { domains: [] };
      }
      const domains = [];
      for (let i = 0; i < uniqueValues.length - 1; i += 1) {
        const domain = createDomain(uniqueValues[i], uniqueValues[i + 1]);
        if (domain) {
          domains.push(domain);
        }
      }
      return { domains };
    }
  });

  register(['{9624aeeb-f2a1-49da-b1c7-8789db217177}', 'remap numbers list'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        V: 'values',
        Values: 'values',
        values: 'values',
        S: 'source',
        Source: 'source',
        source: 'source',
        T: 'target',
        Target: 'target',
        target: 'target',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        Resulted: 'result',
      },
    },
    eval: ({ inputs }) => {
      const values = collectNumbers(inputs.values);
      if (!values.length) {
        return { result: [] };
      }
      const targetDomain = ensureDomain(inputs.target);
      if (!targetDomain) {
        return { result: [...values] };
      }
      const sourceDomain = ensureDomain(inputs.source) ?? computeDomainFromNumbers(values);
      if (!sourceDomain) {
        return { result: values.map(() => targetDomain.start) };
      }
      const result = values.map((value) => remapValue(value, sourceDomain, targetDomain));
      return { result };
    }
  });

  register(['{d1a28e95-cf96-4936-bf34-8bf142d731bf}', 'dom', 'construct domain'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        A: 'start',
        'Domain start': 'start',
        B: 'end',
        'Domain end': 'end',
      },
      outputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
    },
    eval: ({ inputs }) => {
      const domain = createDomain(inputs.start, inputs.end);
      if (!domain) {
        return {};
      }
      return { domain };
    }
  });

  register(['{dd53b24c-003a-4a04-b185-a44d91633cbe}', 'bounds 2d', 'bnd2d'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        C: 'coordinates',
        Coordinates: 'coordinates',
        coordinates: 'coordinates',
      },
      outputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
    },
    eval: ({ inputs }) => {
      const pairs = collectCoordinatePairs(inputs.coordinates);
      if (!pairs.length) {
        return {};
      }
      let minX = Number.POSITIVE_INFINITY;
      let maxX = Number.NEGATIVE_INFINITY;
      let minY = Number.POSITIVE_INFINITY;
      let maxY = Number.NEGATIVE_INFINITY;
      for (const pair of pairs) {
        if (pair.x < minX) minX = pair.x;
        if (pair.x > maxX) maxX = pair.x;
        if (pair.y < minY) minY = pair.y;
        if (pair.y > maxY) maxY = pair.y;
      }
      if (!Number.isFinite(minX) || !Number.isFinite(maxX) || !Number.isFinite(minY) || !Number.isFinite(maxY)) {
        return {};
      }
      const domain = createDomain2([minX, maxX], [minY, maxY]);
      if (!domain) {
        return {};
      }
      return { domain };
    }
  });

  register(['{f0adfc96-b175-46a6-80c7-2b0ee17395c4}', 'dedom2', 'deconstruct domain² components'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
      outputs: {
        U: 'u',
        'U component': 'u',
        V: 'v',
        'V component': 'v',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain2(inputs.domain);
      if (!domain) {
        return {};
      }
      return { u: domain.u, v: domain.v };
    }
  });

  register(['{f217f873-92f1-47ae-ad71-ca3c5a45c3f8}', 'includes', 'inc'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        V: 'value',
        Value: 'value',
        value: 'value',
        D: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
      outputs: {
        I: 'includes',
        Includes: 'includes',
        included: 'includes',
        D: 'deviation',
        Deviation: 'deviation',
      },
    },
    eval: ({ inputs }) => {
      const domain = ensureDomain(inputs.domain);
      const value = toNumber(inputs.value, Number.NaN);
      if (!domain || !Number.isFinite(value)) {
        return { includes: false, deviation: null };
      }
      const includes = isValueInDomain(value, domain);
      const deviation = includes ? 0 : domainDistance(value, domain);
      return { includes, deviation };
    }
  });

  register(['{f44b92b0-3b5b-493a-86f4-fd7408c3daf3}', 'bounds', 'bnd'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        N: 'numbers',
        Numbers: 'numbers',
        numbers: 'numbers',
      },
      outputs: {
        I: 'domain',
        Domain: 'domain',
        domain: 'domain',
      },
    },
    eval: ({ inputs }) => {
      const numbers = collectNumbers(inputs.numbers);
      const domain = computeDomainFromNumbers(numbers);
      if (!domain) {
        return {};
      }
      return { domain };
    }
  });

  register(['{fa314286-867b-41fa-a7f6-3f474197bb81}', 'remap numbers single'], {
    type: 'math-domain',
    pinMap: {
      inputs: {
        V: 'value',
        Value: 'value',
        value: 'value',
        S: 'source',
        Source: 'source',
        source: 'source',
        T: 'target',
        Target: 'target',
        target: 'target',
      },
      outputs: {
        R: 'result',
        Result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const sourceDomain = ensureDomain(inputs.source);
      const targetDomain = ensureDomain(inputs.target);
      if (!sourceDomain || !targetDomain) {
        return {};
      }
      const value = toNumber(inputs.value, Number.NaN);
      if (!Number.isFinite(value)) {
        return { result: targetDomain.start };
      }
      const result = remapValue(value, sourceDomain, targetDomain);
      return { result };
    }
  });
}

export function registerMathOperatorComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math operator components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register math operator components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register math operator components.');
  }

  const EPSILON = 1e-9;

  function unwrapSingle(value) {
    let current = value;
    let depth = 0;
    const maxDepth = 32;
    while (depth < maxDepth) {
      if (current === undefined || current === null) {
        return current;
      }
      if (Array.isArray(current)) {
        if (!current.length) {
          return undefined;
        }
        current = current[0];
        depth += 1;
        continue;
      }
      if (typeof current === 'object' && !current.isVector3) {
        if ('value' in current) {
          current = current.value;
          depth += 1;
          continue;
        }
        if ('item' in current) {
          current = current.item;
          depth += 1;
          continue;
        }
        if ('point' in current) {
          current = current.point;
          depth += 1;
          continue;
        }
        if ('position' in current) {
          current = current.position;
          depth += 1;
          continue;
        }
      }
      return current;
    }
    return current;
  }

  function toBoolean(value, fallback = false) {
    const resolved = unwrapSingle(value);
    if (typeof resolved === 'boolean') {
      return resolved;
    }
    if (typeof resolved === 'number') {
      return resolved !== 0;
    }
    if (typeof resolved === 'string') {
      const normalized = resolved.trim().toLowerCase();
      if (!normalized) return fallback;
      if (['true', 'yes', '1', 'on'].includes(normalized)) return true;
      if (['false', 'no', '0', 'off'].includes(normalized)) return false;
      return fallback;
    }
    if (Array.isArray(resolved)) {
      if (!resolved.length) return fallback;
      return toBoolean(resolved[0], fallback);
    }
    return fallback;
  }

  function isVectorLike(value) {
    if (!value) return false;
    if (value.isVector3) return true;
    if (typeof value === 'object') {
      const x = toNumber(value.x, Number.NaN);
      const y = toNumber(value.y, Number.NaN);
      const z = toNumber(value.z, Number.NaN);
      return Number.isFinite(x) || Number.isFinite(y) || Number.isFinite(z);
    }
    return false;
  }

  function convertValueForMath(value) {
    const resolved = unwrapSingle(value);
    if (resolved === undefined || resolved === null) {
      return null;
    }
    if (isVectorLike(resolved)) {
      return toVector3(resolved, new THREE.Vector3());
    }
    const numeric = toNumber(resolved, Number.NaN);
    if (Number.isFinite(numeric)) {
      return numeric;
    }
    return null;
  }

  function extractOrderedValues(input, result = [], seen = new Set()) {
    if (input === undefined || input === null) {
      return result;
    }
    if (Array.isArray(input)) {
      for (const item of input) {
        extractOrderedValues(item, result, seen);
      }
      return result;
    }
    if (typeof input === 'object') {
      if (input.isVector3) {
        result.push(input.clone());
        return result;
      }
      if (seen.has(input)) {
        return result;
      }
      seen.add(input);
      if ('value' in input) {
        extractOrderedValues(input.value, result, seen);
        return result;
      }
      let hadChildren = false;
      if ('values' in input) {
        hadChildren = true;
        extractOrderedValues(input.values, result, seen);
      }
      if ('items' in input) {
        hadChildren = true;
        extractOrderedValues(input.items, result, seen);
      }
      if ('data' in input) {
        hadChildren = true;
        extractOrderedValues(input.data, result, seen);
      }
      if ('point' in input) {
        hadChildren = true;
        extractOrderedValues(input.point, result, seen);
      }
      if ('position' in input) {
        hadChildren = true;
        extractOrderedValues(input.position, result, seen);
      }
      if (!hadChildren) {
        result.push(input);
      }
      return result;
    }
    result.push(input);
    return result;
  }

  function cloneValue(value) {
    return value?.isVector3 ? value.clone() : value;
  }

  function sequentialCombine(input, combiner) {
    const values = extractOrderedValues(input);
    const converted = [];
    for (const entry of values) {
      const convertedEntry = convertValueForMath(entry);
      if (convertedEntry !== null) {
        converted.push(convertedEntry);
      }
    }
    if (!converted.length) {
      return { result: null, partial: [] };
    }
    let accumulator = converted[0]?.isVector3 ? converted[0].clone() : converted[0];
    const partial = [cloneValue(accumulator)];
    for (let index = 1; index < converted.length; index += 1) {
      const nextValue = converted[index];
      const left = accumulator?.isVector3 ? accumulator.clone() : accumulator;
      const right = nextValue?.isVector3 ? nextValue.clone() : nextValue;
      accumulator = combiner(left, right);
      partial.push(cloneValue(accumulator));
    }
    return { result: accumulator, partial };
  }

  function addScalarsOrVectors(a, b) {
    const aIsVector = isVectorLike(a);
    const bIsVector = isVectorLike(b);
    if (!aIsVector && !bIsVector) {
      return toNumber(a, 0) + toNumber(b, 0);
    }
    const va = toVector3(a, new THREE.Vector3());
    const vb = toVector3(b, new THREE.Vector3());
    return va.add(vb);
  }

  function subtractScalarsOrVectors(a, b) {
    const aIsVector = isVectorLike(a);
    const bIsVector = isVectorLike(b);
    if (!aIsVector && !bIsVector) {
      return toNumber(a, 0) - toNumber(b, 0);
    }
    const va = toVector3(a, new THREE.Vector3());
    const vb = toVector3(b, new THREE.Vector3());
    return va.sub(vb);
  }

  function multiplyScalarsOrVectors(a, b) {
    const aIsVector = isVectorLike(a);
    const bIsVector = isVectorLike(b);
    if (!aIsVector && !bIsVector) {
      return toNumber(a, 0) * toNumber(b, 0);
    }
    const va = toVector3(a, new THREE.Vector3());
    const vb = toVector3(b, new THREE.Vector3());
    if (!bIsVector) {
      return va.multiplyScalar(toNumber(b, 1));
    }
    if (!aIsVector) {
      return vb.multiplyScalar(toNumber(a, 1));
    }
    return new THREE.Vector3(va.x * vb.x, va.y * vb.y, va.z * vb.z);
  }

  function divideScalarsOrVectors(a, b) {
    const aIsVector = isVectorLike(a);
    const bIsVector = isVectorLike(b);
    if (!aIsVector && !bIsVector) {
      return toNumber(a, 0) / toNumber(b, 1);
    }
    if (aIsVector && !bIsVector) {
      const divisor = toNumber(b, 1);
      const vector = toVector3(a, new THREE.Vector3());
      if (divisor === 0) {
        return new THREE.Vector3();
      }
      return vector.divideScalar(divisor);
    }
    const va = toVector3(a, new THREE.Vector3());
    const vb = toVector3(b, new THREE.Vector3(1, 1, 1));
    const safeComponent = (value, divisor) => (divisor === 0 ? 0 : value / divisor);
    return new THREE.Vector3(
      safeComponent(va.x, vb.x),
      safeComponent(va.y, vb.y),
      safeComponent(va.z, vb.z)
    );
  }

  function collectNumberList(input) {
    const values = extractOrderedValues(input);
    const numbers = [];
    for (const value of values) {
      const numeric = toNumber(unwrapSingle(value), Number.NaN);
      if (Number.isFinite(numeric)) {
        numbers.push(numeric);
      }
    }
    return numbers;
  }

  function ensureVector(value) {
    if (value?.isVector3) {
      return value.clone();
    }
    return toVector3(value, new THREE.Vector3());
  }

  function computeFactorial(input) {
    const numeric = Math.floor(toNumber(unwrapSingle(input), Number.NaN));
    if (!Number.isFinite(numeric) || numeric < 0) {
      return null;
    }
    let result = 1;
    for (let i = 2; i <= numeric; i += 1) {
      result *= i;
      if (!Number.isFinite(result)) {
        return Number.POSITIVE_INFINITY;
      }
    }
    return result;
  }

  register(['{040f195d-0b4e-4fe0-901f-fedb2fd3db15}', 'gate and', 'and'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: toBoolean(inputs.a) && toBoolean(inputs.b) })
  });

  register(['{5cad70f9-5a53-4c5c-a782-54a479b4abe3}', 'gate or', 'or'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: toBoolean(inputs.a) || toBoolean(inputs.b) })
  });

  register(['{5ca5de6b-bc71-46c4-a8f7-7f30d7040acb}', 'gate nand', 'nand'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: !(toBoolean(inputs.a) && toBoolean(inputs.b)) })
  });

  register(['{548177c2-d1db-4172-b667-bec979e2d38b}', 'gate nor', 'nor'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: !(toBoolean(inputs.a) || toBoolean(inputs.b)) })
  });

  register(['{de4a0d86-2709-4564-935a-88bf4d40af89}', 'gate xor', 'xor'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const a = toBoolean(inputs.a);
      const b = toBoolean(inputs.b);
      return { result: (a || b) && !(a && b) };
    }
  });

  register(['{b6aedcac-bf43-42d4-899e-d763612f834d}', 'gate xnor', 'xnor'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: toBoolean(inputs.a) === toBoolean(inputs.b) })
  });

  register(['{cb2c7d3c-41b4-4c6d-a6bd-9235bd2851bb}', 'gate not', 'not'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: !toBoolean(inputs.a) })
  });

  register(['{78669f9c-4fea-44fd-ab12-2a69eeec58de}', 'gate majority', 'majority', 'vote'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b', C: 'c', c: 'c' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const votes = [inputs.a, inputs.b, inputs.c].filter((value) => toBoolean(value)).length;
      return { result: votes >= 2 };
    }
  });

  register(['{28124995-cf99-4298-b6f4-c75a8e379f18}', 'absolute', 'abs'], {
    type: 'math',
    pinMap: {
      inputs: { Value: 'value', value: 'value', x: 'value', X: 'value' },
      outputs: { Result: 'result', R: 'result', result: 'result', y: 'result', Y: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumber(unwrapSingle(inputs.value), Number.NaN);
      return { result: Number.isFinite(numeric) ? Math.abs(numeric) : 0 };
    }
  });

  register(['{a3371040-e552-4bc8-b0ff-10a840258e88}', 'negative', 'neg'], {
    type: 'math',
    pinMap: {
      inputs: { Value: 'value', value: 'value', x: 'value', X: 'value' },
      outputs: { Result: 'result', R: 'result', result: 'result', y: 'result', Y: 'result' },
    },
    eval: ({ inputs }) => ({ result: -toNumber(unwrapSingle(inputs.value), 0) })
  });

  register([
    '{a0d62394-a118-422d-abb3-6af115c75b25}',
    '{d18db32b-7099-4eea-85c4-8ba675ee8ec3}',
    'addition',
    'add',
    'a+b',
  ], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: addScalarsOrVectors(inputs.a, inputs.b) })
  });

  register([
    '{2c56ab33-c7cc-4129-886c-d5856b714010}',
    '{9c007a04-d0d9-48e4-9da3-9ba142bc4d46}',
    'subtraction',
    'a-b',
    'minus',
  ], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: subtractScalarsOrVectors(inputs.a, inputs.b) })
  });

  register([
    '{b8963bb1-aa57-476e-a20e-ed6cf635a49c}',
    '{ce46b74e-00c9-43c4-805a-193b69ea4a11}',
    'multiplication',
    'multiply',
    'a×b',
    'a*b',
  ], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: multiplyScalarsOrVectors(inputs.a, inputs.b) })
  });

  register(['{9c85271f-89fa-4e9f-9f4a-d75802120ccc}', 'division', 'divide', 'a/b'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => ({ result: divideScalarsOrVectors(inputs.a, inputs.b) })
  });

  register(['{431bc610-8ae1-4090-b217-1a9d9c519fe2}', 'modulus', 'mod'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const divisor = toNumber(unwrapSingle(inputs.b), Number.NaN);
      const dividend = toNumber(unwrapSingle(inputs.a), Number.NaN);
      if (!Number.isFinite(divisor) || divisor === 0 || !Number.isFinite(dividend)) {
        return { result: null };
      }
      const remainder = ((dividend % divisor) + divisor) % divisor;
      return { result: remainder };
    }
  });

  register(['{54db2568-3441-4ae2-bcef-92c4cc608e11}', 'integer division', 'a\\b', 'int division'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const divisor = toNumber(unwrapSingle(inputs.b), Number.NaN);
      const dividend = toNumber(unwrapSingle(inputs.a), Number.NaN);
      if (!Number.isFinite(divisor) || divisor === 0 || !Number.isFinite(dividend)) {
        return { result: null };
      }
      return { result: Math.trunc(dividend / divisor) };
    }
  });

  register(['{78fed580-851b-46fe-af2f-6519a9d378e0}', 'power', 'pow'], {
    type: 'math',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { R: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const base = toNumber(unwrapSingle(inputs.a), Number.NaN);
      const exponent = toNumber(unwrapSingle(inputs.b), Number.NaN);
      if (!Number.isFinite(base) || !Number.isFinite(exponent)) {
        return { result: null };
      }
      return { result: Math.pow(base, exponent) };
    }
  });

  register([
    '{80da90e3-3ea9-4cfe-b7cc-2b6019f850e3}',
    '{a0a38131-c5fc-4984-b05d-34cf57f0c018}',
    'factorial',
    'fac',
  ], {
    type: 'math',
    pinMap: {
      inputs: { Number: 'value', number: 'value', N: 'value', n: 'value' },
      outputs: { Factorial: 'factorial', F: 'factorial', factorial: 'factorial' },
    },
    eval: ({ inputs }) => ({ factorial: computeFactorial(inputs.value) })
  });

  register(['{40177d8a-a35c-4622-bca7-d150031fe427}', 'similarity', 'similar'], {
    type: 'math',
    pinMap: {
      inputs: {
        'First Number': 'a',
        A: 'a',
        a: 'a',
        'Second Number': 'b',
        B: 'b',
        b: 'b',
        Threshold: 'threshold',
        'T%': 'threshold',
        threshold: 'threshold',
      },
      outputs: {
        Similarity: 'match',
        '=': 'match',
        'Absolute difference': 'difference',
        dt: 'difference',
      },
    },
    eval: ({ inputs }) => {
      const first = toNumber(unwrapSingle(inputs.a), Number.NaN);
      const second = toNumber(unwrapSingle(inputs.b), Number.NaN);
      if (!Number.isFinite(first) || !Number.isFinite(second)) {
        return { match: false, difference: Number.NaN };
      }
      const threshold = Math.abs(toNumber(unwrapSingle(inputs.threshold), 0));
      const difference = Math.abs(first - second);
      return { match: difference <= threshold, difference };
    }
  });

  register(['{5db0fb89-4f22-4f09-a777-fa5e55aed7ec}', 'equality', 'equals'], {
    type: 'math',
    pinMap: {
      inputs: {
        'First Number': 'a',
        A: 'a',
        a: 'a',
        'Second Number': 'b',
        B: 'b',
        b: 'b',
      },
      outputs: {
        Equality: 'equal',
        '=': 'equal',
        Inequality: 'notEqual',
        '≠': 'notEqual',
      },
    },
    eval: ({ inputs }) => {
      const leftRaw = unwrapSingle(inputs.a);
      const rightRaw = unwrapSingle(inputs.b);
      let equal = false;
      if (isVectorLike(leftRaw) || isVectorLike(rightRaw)) {
        const leftVector = ensureVector(leftRaw ?? 0);
        const rightVector = ensureVector(rightRaw ?? 0);
        equal = leftVector.distanceTo(rightVector) <= EPSILON;
      } else {
        const leftNumeric = toNumber(leftRaw, Number.NaN);
        const rightNumeric = toNumber(rightRaw, Number.NaN);
        if (Number.isFinite(leftNumeric) && Number.isFinite(rightNumeric)) {
          equal = Math.abs(leftNumeric - rightNumeric) <= EPSILON;
        } else {
          equal = leftRaw === rightRaw;
        }
      }
      return { equal, notEqual: !equal };
    }
  });

  register(['{30d58600-1aab-42db-80a3-f1ea6c4269a0}', 'larger than', 'greater than', '>'], {
    type: 'math',
    pinMap: {
      inputs: {
        'First Number': 'a',
        A: 'a',
        a: 'a',
        'Second Number': 'b',
        B: 'b',
        b: 'b',
      },
      outputs: {
        'Larger than': 'greater',
        '>': 'greater',
        '… or Equal to': 'greaterOrEqual',
        '... or Equal to': 'greaterOrEqual',
        '>=': 'greaterOrEqual',
      },
    },
    eval: ({ inputs }) => {
      const first = toNumber(unwrapSingle(inputs.a), Number.NaN);
      const second = toNumber(unwrapSingle(inputs.b), Number.NaN);
      if (!Number.isFinite(first) || !Number.isFinite(second)) {
        return { greater: false, greaterOrEqual: false };
      }
      return {
        greater: first > second,
        greaterOrEqual: first >= second,
      };
    }
  });

  register(['{ae840986-cade-4e5a-96b0-570f007d4fc0}', 'smaller than', 'less than', '<'], {
    type: 'math',
    pinMap: {
      inputs: {
        'First Number': 'a',
        A: 'a',
        a: 'a',
        'Second Number': 'b',
        B: 'b',
        b: 'b',
      },
      outputs: {
        'Smaller than': 'smaller',
        '<': 'smaller',
        '… or Equal to': 'smallerOrEqual',
        '... or Equal to': 'smallerOrEqual',
        '<=': 'smallerOrEqual',
      },
    },
    eval: ({ inputs }) => {
      const first = toNumber(unwrapSingle(inputs.a), Number.NaN);
      const second = toNumber(unwrapSingle(inputs.b), Number.NaN);
      if (!Number.isFinite(first) || !Number.isFinite(second)) {
        return { smaller: false, smallerOrEqual: false };
      }
      return {
        smaller: first < second,
        smallerOrEqual: first <= second,
      };
    }
  });

  register(['{586706a8-109b-43ec-b581-743e920c951a}', 'series addition', 'sa'], {
    type: 'math',
    pinMap: {
      inputs: {
        Numbers: 'numbers',
        N: 'numbers',
        numbers: 'numbers',
        Goal: 'goal',
        G: 'goal',
        goal: 'goal',
        Start: 'start',
        S: 'start',
        start: 'start',
      },
      outputs: {
        Series: 'series',
        S: 'series',
        Remainder: 'remainder',
        R: 'remainder',
      },
    },
    eval: ({ inputs }) => {
      const pool = collectNumberList(inputs.numbers);
      const goal = toNumber(unwrapSingle(inputs.goal), Number.NaN);
      const startValue = toNumber(unwrapSingle(inputs.start), 0);
      const series = [];
      let total = startValue;
      if (!pool.length) {
        return {
          series,
          remainder: Number.isFinite(goal) ? startValue - goal : 0,
        };
      }
      const hasGoal = Number.isFinite(goal);
      const direction = hasGoal ? Math.sign(goal - startValue) : 0;
      for (const value of pool) {
        total += value;
        series.push(total);
        if (hasGoal) {
          if ((direction >= 0 && total >= goal) || (direction < 0 && total <= goal)) {
            break;
          }
        }
      }
      return {
        series,
        remainder: hasGoal ? total - goal : 0,
      };
    }
  });

  register(['{5b850221-b527-4bd6-8c62-e94168cd6efa}', 'mass addition', 'ma'], {
    type: 'math',
    pinMap: {
      inputs: { Input: 'values', input: 'values', I: 'values', values: 'values' },
      outputs: {
        Result: 'result',
        R: 'result',
        result: 'result',
        'Partial Results': 'partialResults',
        PR: 'partialResults',
        Pr: 'partialResults',
      },
    },
    eval: ({ inputs }) => {
      const { result, partial } = sequentialCombine(inputs.values, addScalarsOrVectors);
      if (result === null || result === undefined) {
        return { result: 0, partialResults: [] };
      }
      return {
        result: cloneValue(result),
        partialResults: partial.map(cloneValue),
      };
    }
  });

  register([
    '{921775f7-bf22-4cfc-a4db-c415a56069c4}',
    '{e44c1bd7-72cc-4697-80c9-02787baf7bb4}',
    'mass multiplication',
    'mm',
  ], {
    type: 'math',
    pinMap: {
      inputs: { Input: 'values', input: 'values', I: 'values', values: 'values' },
      outputs: {
        Result: 'result',
        R: 'result',
        result: 'result',
        'Partial Results': 'partialResults',
        PR: 'partialResults',
        Pr: 'partialResults',
      },
    },
    eval: ({ inputs }) => {
      const { result, partial } = sequentialCombine(inputs.values, multiplyScalarsOrVectors);
      if (result === null || result === undefined) {
        return { result: 1, partialResults: [] };
      }
      return {
        result: cloneValue(result),
        partialResults: partial.map(cloneValue),
      };
    }
  });

  register(['{dd17d442-3776-40b3-ad5b-5e188b56bd4c}', 'relative differences', 'reldif'], {
    type: 'math',
    pinMap: {
      inputs: { Values: 'values', values: 'values', V: 'values' },
      outputs: { Differenced: 'differences', D: 'differences' },
    },
    eval: ({ inputs }) => {
      const ordered = extractOrderedValues(inputs.values);
      const converted = [];
      for (const entry of ordered) {
        const convertedEntry = convertValueForMath(entry);
        if (convertedEntry !== null) {
          converted.push(convertedEntry);
        }
      }
      if (converted.length < 2) {
        return { differences: [] };
      }
      const differences = [];
      for (let index = 1; index < converted.length; index += 1) {
        const current = converted[index];
        const previous = converted[index - 1];
        if ((current && current.isVector3) || (previous && previous.isVector3)) {
          const currentVector = ensureVector(current ?? 0);
          const previousVector = ensureVector(previous ?? 0);
          differences.push(currentVector.sub(previousVector));
        } else {
          differences.push(current - previous);
        }
      }
      return { differences: differences.map(cloneValue) };
    }
  });
}

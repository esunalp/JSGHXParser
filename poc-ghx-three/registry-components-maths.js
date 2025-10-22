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

export function registerMathTrigComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math trigonometry components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register math trigonometry components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register math trigonometry components.');
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
      if (typeof current === 'object' && !current?.isVector3) {
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

  function toNumeric(value) {
    return toNumber(unwrapSingle(value), Number.NaN);
  }

  function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
  }

  function assignIfMissing(target, key, value) {
    if (!Number.isFinite(value)) {
      return false;
    }
    if (target[key] === undefined) {
      target[key] = value;
      return true;
    }
    return false;
  }

  function assignAngle(target, key, value) {
    if (!Number.isFinite(value) || value <= EPSILON) {
      return false;
    }
    return assignIfMissing(target, key, value);
  }

  function assignLength(target, key, value) {
    if (!Number.isFinite(value) || value <= EPSILON) {
      return false;
    }
    return assignIfMissing(target, key, value);
  }

  function isAngle(value) {
    return Number.isFinite(value) && value > EPSILON;
  }

  function isLength(value) {
    return Number.isFinite(value) && value > EPSILON;
  }

  function resolvePoint(value, depth = 0) {
    if (depth > 8) {
      return null;
    }
    const resolved = unwrapSingle(value);
    if (resolved === undefined || resolved === null) {
      return null;
    }
    if (resolved?.isVector3) {
      return resolved.clone();
    }
    if (Array.isArray(resolved)) {
      if (!resolved.length) {
        return null;
      }
      if (resolved.length >= 3) {
        const x = toNumber(resolved[0], Number.NaN);
        const y = toNumber(resolved[1], Number.NaN);
        const z = toNumber(resolved[2], Number.NaN);
        if (!Number.isFinite(x) && !Number.isFinite(y) && !Number.isFinite(z)) {
          return null;
        }
        return new THREE.Vector3(
          Number.isFinite(x) ? x : 0,
          Number.isFinite(y) ? y : 0,
          Number.isFinite(z) ? z : 0,
        );
      }
      return resolvePoint(resolved[0], depth + 1);
    }
    if (typeof resolved === 'object') {
      if ('point' in resolved) {
        return resolvePoint(resolved.point, depth + 1);
      }
      if ('position' in resolved) {
        return resolvePoint(resolved.position, depth + 1);
      }
      if ('value' in resolved) {
        return resolvePoint(resolved.value, depth + 1);
      }
      const x = toNumber(resolved.x, Number.NaN);
      const y = toNumber(resolved.y, Number.NaN);
      const z = toNumber(resolved.z, Number.NaN);
      if (!Number.isFinite(x) && !Number.isFinite(y) && !Number.isFinite(z)) {
        return null;
      }
      return new THREE.Vector3(
        Number.isFinite(x) ? x : 0,
        Number.isFinite(y) ? y : 0,
        Number.isFinite(z) ? z : 0,
      );
    }
    return null;
  }

  function createLine(startPoint, endPoint) {
    const start = startPoint.clone();
    const end = endPoint.clone();
    const direction = end.clone().sub(start);
    const length = direction.length();
    const safeDirection = length > EPSILON ? direction.clone().divideScalar(length) : new THREE.Vector3(1, 0, 0);
    return {
      type: 'line',
      start,
      end,
      length,
      direction: safeDirection,
    };
  }

  function createTriangleData(pointA, pointB, pointC) {
    const a = resolvePoint(pointA);
    const b = resolvePoint(pointB);
    const c = resolvePoint(pointC);
    if (!a || !b || !c) {
      return null;
    }
    const ab = b.clone().sub(a);
    const ac = c.clone().sub(a);
    const normal = ab.clone().cross(ac);
    const areaSq = normal.lengthSq();
    if (areaSq < EPSILON) {
      return null;
    }
    const abLength = ab.length();
    if (abLength < EPSILON) {
      return null;
    }
    const xAxis = ab.clone().divideScalar(abLength);
    const zAxis = normal.clone().normalize();
    const yAxis = zAxis.clone().cross(xAxis).normalize();
    const cRelative = c.clone().sub(a);
    const cX = cRelative.dot(xAxis);
    const cY = cRelative.dot(yAxis);
    return {
      a,
      b,
      c,
      frame: {
        origin: a.clone(),
        xAxis,
        yAxis,
        zAxis,
        abLength,
      },
      coords: {
        a: { x: 0, y: 0 },
        b: { x: abLength, y: 0 },
        c: { x: cX, y: cY },
      },
    };
  }

  function from2D(frame, point) {
    const result = frame.origin.clone();
    result.add(frame.xAxis.clone().multiplyScalar(point.x));
    result.add(frame.yAxis.clone().multiplyScalar(point.y));
    return result;
  }

  function midpoint2D(p, q) {
    return {
      x: (p.x + q.x) / 2,
      y: (p.y + q.y) / 2,
    };
  }

  function projectPointOntoLine(point, start, end) {
    const dx = end.x - start.x;
    const dy = end.y - start.y;
    const denom = dx * dx + dy * dy;
    if (denom < EPSILON) {
      return { x: start.x, y: start.y };
    }
    const t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / denom;
    return {
      x: start.x + dx * t,
      y: start.y + dy * t,
    };
  }

  function computeCircumcentreData(triangle) {
    const { frame, coords } = triangle;
    const ax = coords.a.x;
    const ay = coords.a.y;
    const bx = coords.b.x;
    const by = coords.b.y;
    const cx = coords.c.x;
    const cy = coords.c.y;
    const d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if (Math.abs(d) < EPSILON) {
      return null;
    }
    const ax2ay2 = ax * ax + ay * ay;
    const bx2by2 = bx * bx + by * by;
    const cx2cy2 = cx * cx + cy * cy;
    const ux = (ax2ay2 * (by - cy) + bx2by2 * (cy - ay) + cx2cy2 * (ay - by)) / d;
    const uy = (ax2ay2 * (cx - bx) + bx2by2 * (ax - cx) + cx2cy2 * (bx - ax)) / d;
    const centre2D = { x: ux, y: uy };
    const centre = from2D(frame, centre2D);
    const midAB = from2D(frame, midpoint2D(coords.a, coords.b));
    const midBC = from2D(frame, midpoint2D(coords.b, coords.c));
    const midCA = from2D(frame, midpoint2D(coords.c, coords.a));
    return {
      centre,
      bisectorAB: createLine(midAB, centre.clone()),
      bisectorBC: createLine(midBC, centre.clone()),
      bisectorCA: createLine(midCA, centre.clone()),
    };
  }

  function computeOrthocentreData(triangle) {
    const { frame, coords, a, b, c } = triangle;
    const bx = coords.b.x;
    const cx = coords.c.x;
    const cy = coords.c.y;
    if (Math.abs(cy) < EPSILON) {
      return null;
    }
    const ortho2D = { x: cx, y: (cx * (bx - cx)) / cy };
    const ortho = from2D(frame, ortho2D);
    const footAB = from2D(frame, { x: cx, y: 0 });
    const footBC = from2D(frame, projectPointOntoLine(coords.a, coords.b, coords.c));
    const footCA = from2D(frame, projectPointOntoLine(coords.b, coords.a, coords.c));
    return {
      orthocentre: ortho,
      altitudeAB: createLine(c.clone(), footAB),
      altitudeBC: createLine(a.clone(), footBC),
      altitudeCA: createLine(b.clone(), footCA),
    };
  }

  function computeCentroidData(triangle) {
    const { frame, coords, a, b, c } = triangle;
    const centroid = a.clone().add(b).add(c).multiplyScalar(1 / 3);
    const midAB = from2D(frame, midpoint2D(coords.a, coords.b));
    const midBC = from2D(frame, midpoint2D(coords.b, coords.c));
    const midCA = from2D(frame, midpoint2D(coords.c, coords.a));
    return {
      centroid,
      medianAB: createLine(c.clone(), midAB),
      medianBC: createLine(a.clone(), midBC),
      medianCA: createLine(b.clone(), midCA),
    };
  }

  function computeIncentreData(triangle) {
    const { a, b, c } = triangle;
    const sideA = b.clone().sub(c).length();
    const sideB = a.clone().sub(c).length();
    const sideC = a.clone().sub(b).length();
    const perimeter = sideA + sideB + sideC;
    if (!Number.isFinite(perimeter) || perimeter < EPSILON) {
      return null;
    }
    const incenter = new THREE.Vector3();
    incenter.add(a.clone().multiplyScalar(sideA));
    incenter.add(b.clone().multiplyScalar(sideB));
    incenter.add(c.clone().multiplyScalar(sideC));
    incenter.divideScalar(perimeter);
    return {
      incenter,
      bisectorA: createLine(a.clone(), incenter.clone()),
      bisectorB: createLine(b.clone(), incenter.clone()),
      bisectorC: createLine(c.clone(), incenter.clone()),
    };
  }

  function detectAngleUnit(values, sumTarget) {
    const valid = values.filter((value) => Number.isFinite(value));
    if (!valid.length) {
      return 'radians';
    }
    const maxAbs = Math.max(...valid.map((value) => Math.abs(value)));
    if (maxAbs > sumTarget + 0.1) {
      return 'degrees';
    }
    const sum = valid.reduce((acc, value) => acc + Math.abs(value), 0);
    if (sum > sumTarget + 0.1) {
      return 'degrees';
    }
    return 'radians';
  }

  function computeSineRatio(state) {
    const ratios = [];
    if (isAngle(state.alpha) && isLength(state.a)) {
      const sinAlpha = Math.sin(state.alpha);
      if (Math.abs(sinAlpha) > EPSILON) {
        ratios.push(state.a / sinAlpha);
      }
    }
    if (isAngle(state.beta) && isLength(state.b)) {
      const sinBeta = Math.sin(state.beta);
      if (Math.abs(sinBeta) > EPSILON) {
        ratios.push(state.b / sinBeta);
      }
    }
    if (isAngle(state.gamma) && isLength(state.c)) {
      const sinGamma = Math.sin(state.gamma);
      if (Math.abs(sinGamma) > EPSILON) {
        ratios.push(state.c / sinGamma);
      }
    }
    if (!ratios.length) {
      return null;
    }
    const valid = ratios.filter((value) => Number.isFinite(value) && Math.abs(value) > EPSILON);
    if (!valid.length) {
      return null;
    }
    return valid.reduce((acc, value) => acc + value, 0) / valid.length;
  }

  function solveTriangle(initial) {
    const state = { ...initial };
    for (let iteration = 0; iteration < 32; iteration += 1) {
      let changed = false;
      if (isAngle(state.alpha) && isAngle(state.beta) && state.gamma === undefined) {
        changed = assignAngle(state, 'gamma', Math.PI - state.alpha - state.beta) || changed;
      }
      if (isAngle(state.alpha) && isAngle(state.gamma) && state.beta === undefined) {
        changed = assignAngle(state, 'beta', Math.PI - state.alpha - state.gamma) || changed;
      }
      if (isAngle(state.beta) && isAngle(state.gamma) && state.alpha === undefined) {
        changed = assignAngle(state, 'alpha', Math.PI - state.beta - state.gamma) || changed;
      }

      const sineRatio = computeSineRatio(state);
      if (sineRatio !== null) {
        if (isAngle(state.alpha) && state.a === undefined) {
          changed = assignLength(state, 'a', Math.sin(state.alpha) * sineRatio) || changed;
        }
        if (isAngle(state.beta) && state.b === undefined) {
          changed = assignLength(state, 'b', Math.sin(state.beta) * sineRatio) || changed;
        }
        if (isAngle(state.gamma) && state.c === undefined) {
          changed = assignLength(state, 'c', Math.sin(state.gamma) * sineRatio) || changed;
        }
        if (state.a !== undefined && state.alpha === undefined) {
          const sinAlpha = clamp(state.a / sineRatio, -1, 1);
          const candidate = Math.asin(sinAlpha);
          changed = assignAngle(state, 'alpha', candidate) || changed;
        }
        if (state.b !== undefined && state.beta === undefined) {
          const sinBeta = clamp(state.b / sineRatio, -1, 1);
          const candidate = Math.asin(sinBeta);
          changed = assignAngle(state, 'beta', candidate) || changed;
        }
        if (state.c !== undefined && state.gamma === undefined) {
          const sinGamma = clamp(state.c / sineRatio, -1, 1);
          const candidate = Math.asin(sinGamma);
          changed = assignAngle(state, 'gamma', candidate) || changed;
        }
      }

      if (state.a === undefined && isLength(state.b) && isLength(state.c) && isAngle(state.alpha)) {
        const value = Math.sqrt(Math.max(0, state.b * state.b + state.c * state.c - 2 * state.b * state.c * Math.cos(state.alpha)));
        changed = assignLength(state, 'a', value) || changed;
      }
      if (state.b === undefined && isLength(state.a) && isLength(state.c) && isAngle(state.beta)) {
        const value = Math.sqrt(Math.max(0, state.a * state.a + state.c * state.c - 2 * state.a * state.c * Math.cos(state.beta)));
        changed = assignLength(state, 'b', value) || changed;
      }
      if (state.c === undefined && isLength(state.a) && isLength(state.b) && isAngle(state.gamma)) {
        const value = Math.sqrt(Math.max(0, state.a * state.a + state.b * state.b - 2 * state.a * state.b * Math.cos(state.gamma)));
        changed = assignLength(state, 'c', value) || changed;
      }

      if (isLength(state.a) && isLength(state.b) && isLength(state.c)) {
        if (state.alpha === undefined) {
          const cosAlpha = clamp((state.b * state.b + state.c * state.c - state.a * state.a) / (2 * state.b * state.c), -1, 1);
          changed = assignAngle(state, 'alpha', Math.acos(cosAlpha)) || changed;
        }
        if (state.beta === undefined) {
          const cosBeta = clamp((state.a * state.a + state.c * state.c - state.b * state.b) / (2 * state.a * state.c), -1, 1);
          changed = assignAngle(state, 'beta', Math.acos(cosBeta)) || changed;
        }
        if (state.gamma === undefined) {
          const cosGamma = clamp((state.a * state.a + state.b * state.b - state.c * state.c) / (2 * state.a * state.b), -1, 1);
          changed = assignAngle(state, 'gamma', Math.acos(cosGamma)) || changed;
        }
      }

      if (!changed) {
        break;
      }
    }
    return state;
  }

  function solveRightTriangle(initial) {
    const state = { ...initial };
    for (let iteration = 0; iteration < 32; iteration += 1) {
      let changed = false;
      if (isAngle(state.alpha) && state.beta === undefined) {
        changed = assignAngle(state, 'beta', Math.PI / 2 - state.alpha) || changed;
      }
      if (isAngle(state.beta) && state.alpha === undefined) {
        changed = assignAngle(state, 'alpha', Math.PI / 2 - state.beta) || changed;
      }

      if (isLength(state.p) && isLength(state.q) && state.r === undefined) {
        changed = assignLength(state, 'r', Math.hypot(state.p, state.q)) || changed;
      }
      if (isLength(state.p) && isLength(state.r) && state.q === undefined && state.r > state.p) {
        const value = Math.sqrt(Math.max(0, state.r * state.r - state.p * state.p));
        changed = assignLength(state, 'q', value) || changed;
      }
      if (isLength(state.q) && isLength(state.r) && state.p === undefined && state.r > state.q) {
        const value = Math.sqrt(Math.max(0, state.r * state.r - state.q * state.q));
        changed = assignLength(state, 'p', value) || changed;
      }

      if (isAngle(state.alpha) && state.r !== undefined) {
        if (state.p === undefined) {
          changed = assignLength(state, 'p', state.r * Math.sin(state.alpha)) || changed;
        }
        if (state.q === undefined) {
          changed = assignLength(state, 'q', state.r * Math.cos(state.alpha)) || changed;
        }
      }
      if (isAngle(state.beta) && state.r !== undefined) {
        if (state.q === undefined) {
          changed = assignLength(state, 'q', state.r * Math.sin(state.beta)) || changed;
        }
        if (state.p === undefined) {
          changed = assignLength(state, 'p', state.r * Math.cos(state.beta)) || changed;
        }
      }

      if (isAngle(state.alpha) && isLength(state.p) && state.r === undefined) {
        changed = assignLength(state, 'r', state.p / Math.sin(state.alpha)) || changed;
      }
      if (isAngle(state.alpha) && isLength(state.q) && state.r === undefined) {
        changed = assignLength(state, 'r', state.q / Math.cos(state.alpha)) || changed;
      }
      if (isAngle(state.beta) && isLength(state.p) && state.r === undefined) {
        changed = assignLength(state, 'r', state.p / Math.cos(state.beta)) || changed;
      }
      if (isAngle(state.beta) && isLength(state.q) && state.r === undefined) {
        changed = assignLength(state, 'r', state.q / Math.sin(state.beta)) || changed;
      }

      if (isLength(state.p) && isLength(state.q)) {
        if (state.alpha === undefined) {
          changed = assignAngle(state, 'alpha', Math.atan2(state.p, state.q)) || changed;
        }
        if (state.beta === undefined) {
          changed = assignAngle(state, 'beta', Math.atan2(state.q, state.p)) || changed;
        }
      }
      if (isLength(state.p) && isLength(state.r) && state.alpha === undefined && state.r > EPSILON) {
        changed = assignAngle(state, 'alpha', Math.asin(clamp(state.p / state.r, -1, 1))) || changed;
      }
      if (isLength(state.q) && isLength(state.r) && state.beta === undefined && state.r > EPSILON) {
        changed = assignAngle(state, 'beta', Math.asin(clamp(state.q / state.r, -1, 1))) || changed;
      }

      if (!changed) {
        break;
      }
    }
    if (isAngle(state.alpha) && !isAngle(state.beta)) {
      assignAngle(state, 'beta', Math.PI / 2 - state.alpha);
    }
    if (isAngle(state.beta) && !isAngle(state.alpha)) {
      assignAngle(state, 'alpha', Math.PI / 2 - state.beta);
    }
    return state;
  }

  function simpleTrigEval(inputs, fn, { handleSingularity = false } = {}) {
    const numeric = toNumeric(inputs.value);
    if (!Number.isFinite(numeric)) {
      return { result: 0 };
    }
    if (handleSingularity && Math.abs(numeric) < EPSILON) {
      return { result: 1 };
    }
    return { result: fn(numeric) };
  }

  register(['{0d77c51e-584f-44e8-aed2-c2ddf4803888}', 'degrees', 'deg'], {
    type: 'math',
    pinMap: {
      inputs: { R: 'radians', r: 'radians', Radians: 'radians', radians: 'radians' },
      outputs: { D: 'degrees', Degrees: 'degrees', degrees: 'degrees' },
    },
    eval: ({ inputs }) => {
      const radians = toNumeric(inputs.radians);
      if (!Number.isFinite(radians)) {
        return { degrees: 0 };
      }
      return { degrees: radians * (180 / Math.PI) };
    }
  });

  register(['{a4cd2751-414d-42ec-8916-476ebf62d7fe}', 'radians', 'rad'], {
    type: 'math',
    pinMap: {
      inputs: { D: 'degrees', d: 'degrees', Degrees: 'degrees', degrees: 'degrees' },
      outputs: { R: 'radians', Radians: 'radians', radians: 'radians' },
    },
    eval: ({ inputs }) => {
      const degrees = toNumeric(inputs.degrees);
      if (!Number.isFinite(degrees)) {
        return { radians: 0 };
      }
      return { radians: degrees * (Math.PI / 180) };
    }
  });

  register(['{7663efbb-d9b8-4c6a-a0da-c3750a7bbe77}', 'sine', 'sin'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => simpleTrigEval(inputs, Math.sin),
  });

  register(['{d2d2a900-780c-4d58-9a35-1f9d8d35df6f}', 'cosine', 'cos'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => simpleTrigEval(inputs, Math.cos),
  });

  register(['{0f31784f-7177-4104-8500-1f4f4a306df4}', 'tangent', 'tan'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      const cosValue = Math.cos(numeric);
      if (Math.abs(cosValue) < EPSILON) {
        return { result: null };
      }
      return { result: Math.tan(numeric) };
    }
  });

  register(['{1f602c33-f38e-4f47-898b-359f0a4de3c2}', 'cotangent', 'cot'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      const tanValue = Math.tan(numeric);
      if (Math.abs(tanValue) < EPSILON) {
        return { result: null };
      }
      return { result: 1 / tanValue };
    }
  });

  register(['{60103def-1bb7-4700-b294-3a89100525c4}', 'secant', 'sec'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      const cosValue = Math.cos(numeric);
      if (Math.abs(cosValue) < EPSILON) {
        return { result: null };
      }
      return { result: 1 / cosValue };
    }
  });

  register(['{d222500b-dfd5-45e0-933e-eabefd07cbfa}', 'cosecant', 'csc'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      const sinValue = Math.sin(numeric);
      if (Math.abs(sinValue) < EPSILON) {
        return { result: null };
      }
      return { result: 1 / sinValue };
    }
  });

  register(['{cc15ba56-fae7-4f05-b599-cb7c43b60e11}', 'arcsine', 'asin'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      if (numeric < -1 || numeric > 1) {
        return { result: null };
      }
      return { result: Math.asin(clamp(numeric, -1, 1)) };
    }
  });

  register(['{49584390-d541-41f7-b5f6-1f9515ac0f73}', 'arccosine', 'acos'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const numeric = toNumeric(inputs.value);
      if (!Number.isFinite(numeric)) {
        return { result: 0 };
      }
      if (numeric < -1 || numeric > 1) {
        return { result: null };
      }
      return { result: Math.acos(clamp(numeric, -1, 1)) };
    }
  });

  register(['{b4647919-d041-419e-99f5-fa0dc0ddb8b6}', 'arctangent', 'atan'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => simpleTrigEval(inputs, Math.atan),
  });

  register(['{a2d9503d-a83c-4d71-81e0-02af8d09cd0c}', 'sinc'], {
    type: 'math',
    pinMap: {
      inputs: { x: 'value', X: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => simpleTrigEval(inputs, (x) => Math.sin(x) / x, { handleSingularity: true }),
  });

  register(['{21d0767c-5340-4087-aa09-398d0e706908}', 'circumcentre', 'ccentre', 'circumcenter'], {
    type: 'math',
    pinMap: {
      inputs: {
        A: 'pointA', 'Point A': 'pointA', pointA: 'pointA',
        B: 'pointB', 'Point B': 'pointB', pointB: 'pointB',
        C: 'pointC', 'Point C': 'pointC', pointC: 'pointC',
      },
      outputs: {
        C: 'circumcentre', Circumcentre: 'circumcentre', Circumcenter: 'circumcentre',
        AB: 'bisectorAB', 'Bisector AB': 'bisectorAB',
        BC: 'bisectorBC', 'Bisector BC': 'bisectorBC',
        CA: 'bisectorCA', 'Bisector CA': 'bisectorCA',
      },
    },
    eval: ({ inputs }) => {
      const triangle = createTriangleData(inputs.pointA, inputs.pointB, inputs.pointC);
      if (!triangle) {
        return { circumcentre: null, bisectorAB: null, bisectorBC: null, bisectorCA: null };
      }
      const data = computeCircumcentreData(triangle);
      if (!data) {
        return { circumcentre: null, bisectorAB: null, bisectorBC: null, bisectorCA: null };
      }
      return {
        circumcentre: data.centre,
        bisectorAB: data.bisectorAB,
        bisectorBC: data.bisectorBC,
        bisectorCA: data.bisectorCA,
      };
    }
  });

  register(['{36dd5551-b6bd-4246-bd2f-1fd91eb2f02d}', 'orthocentre', 'ocentre', 'orthocenter'], {
    type: 'math',
    pinMap: {
      inputs: {
        A: 'pointA', 'Point A': 'pointA',
        B: 'pointB', 'Point B': 'pointB',
        C: 'pointC', 'Point C': 'pointC',
      },
      outputs: {
        C: 'orthocentre', Orthocentre: 'orthocentre', Orthocenter: 'orthocentre',
        AB: 'altitudeAB', 'Altitude AB': 'altitudeAB',
        BC: 'altitudeBC', 'Altitude BC': 'altitudeBC',
        CA: 'altitudeCA', 'Altitude CA': 'altitudeCA',
      },
    },
    eval: ({ inputs }) => {
      const triangle = createTriangleData(inputs.pointA, inputs.pointB, inputs.pointC);
      if (!triangle) {
        return { orthocentre: null, altitudeAB: null, altitudeBC: null, altitudeCA: null };
      }
      const data = computeOrthocentreData(triangle);
      if (!data) {
        return { orthocentre: null, altitudeAB: null, altitudeBC: null, altitudeCA: null };
      }
      return {
        orthocentre: data.orthocentre,
        altitudeAB: data.altitudeAB,
        altitudeBC: data.altitudeBC,
        altitudeCA: data.altitudeCA,
      };
    }
  });

  register(['{afbcbad4-2a2a-4954-8040-d999e316d2bd}', 'centroid'], {
    type: 'math',
    pinMap: {
      inputs: {
        A: 'pointA', 'Point A': 'pointA',
        B: 'pointB', 'Point B': 'pointB',
        C: 'pointC', 'Point C': 'pointC',
      },
      outputs: {
        C: 'centroid', Centroid: 'centroid',
        AB: 'medianAB', 'Median AB': 'medianAB',
        BC: 'medianBC', 'Median BC': 'medianBC',
        CA: 'medianCA', 'Median CA': 'medianCA',
      },
    },
    eval: ({ inputs }) => {
      const triangle = createTriangleData(inputs.pointA, inputs.pointB, inputs.pointC);
      if (!triangle) {
        return { centroid: null, medianAB: null, medianBC: null, medianCA: null };
      }
      const data = computeCentroidData(triangle);
      return {
        centroid: data.centroid,
        medianAB: data.medianAB,
        medianBC: data.medianBC,
        medianCA: data.medianCA,
      };
    }
  });

  register(['{c3342ea2-e181-46aa-a9b9-e438ccbfb831}', 'incentre', 'icentre', 'incenter'], {
    type: 'math',
    pinMap: {
      inputs: {
        A: 'pointA', 'Point A': 'pointA',
        B: 'pointB', 'Point B': 'pointB',
        C: 'pointC', 'Point C': 'pointC',
      },
      outputs: {
        I: 'incentre', Incentre: 'incentre', Incenter: 'incentre',
        A: 'bisectorA', 'Bisector A': 'bisectorA',
        B: 'bisectorB', 'Bisector B': 'bisectorB',
        C: 'bisectorC', 'Bisector C': 'bisectorC',
      },
    },
    eval: ({ inputs }) => {
      const triangle = createTriangleData(inputs.pointA, inputs.pointB, inputs.pointC);
      if (!triangle) {
        return { incentre: null, bisectorA: null, bisectorB: null, bisectorC: null };
      }
      const data = computeIncentreData(triangle);
      if (!data) {
        return { incentre: null, bisectorA: null, bisectorB: null, bisectorC: null };
      }
      return {
        incentre: data.incenter,
        bisectorA: data.bisectorA,
        bisectorB: data.bisectorB,
        bisectorC: data.bisectorC,
      };
    }
  });

  register(['{92af1a02-9b87-43a0-8c45-0ce1b81555ec}', 'triangle trigonometry', 'trig'], {
    type: 'math',
    pinMap: {
      inputs: {
        α: 'alpha', Alpha: 'alpha', alpha: 'alpha',
        β: 'beta', Beta: 'beta', beta: 'beta',
        γ: 'gamma', Gamma: 'gamma', gamma: 'gamma',
        A: 'aLength', 'A length': 'aLength',
        B: 'bLength', 'B length': 'bLength',
        C: 'cLength', 'C length': 'cLength',
      },
      outputs: {
        α: 'alpha', Alpha: 'alpha', alpha: 'alpha',
        β: 'beta', Beta: 'beta', beta: 'beta',
        γ: 'gamma', Gamma: 'gamma', gamma: 'gamma',
        A: 'aLength', 'A length': 'aLength',
        B: 'bLength', 'B length': 'bLength',
        C: 'cLength', 'C length': 'cLength',
      },
    },
    eval: ({ inputs }) => {
      const alphaInput = toNumeric(inputs.alpha);
      const betaInput = toNumeric(inputs.beta);
      const gammaInput = toNumeric(inputs.gamma);
      const aInput = toNumeric(inputs.aLength);
      const bInput = toNumeric(inputs.bLength);
      const cInput = toNumeric(inputs.cLength);
      const unit = detectAngleUnit([alphaInput, betaInput, gammaInput], Math.PI);
      const toRadiansFactor = unit === 'degrees' ? Math.PI / 180 : 1;
      const fromRadiansFactor = unit === 'degrees' ? 180 / Math.PI : 1;
      const solution = solveTriangle({
        alpha: Number.isFinite(alphaInput) ? alphaInput * toRadiansFactor : undefined,
        beta: Number.isFinite(betaInput) ? betaInput * toRadiansFactor : undefined,
        gamma: Number.isFinite(gammaInput) ? gammaInput * toRadiansFactor : undefined,
        a: isLength(aInput) ? aInput : undefined,
        b: isLength(bInput) ? bInput : undefined,
        c: isLength(cInput) ? cInput : undefined,
      });
      return {
        alpha: isAngle(solution.alpha) ? solution.alpha * fromRadiansFactor : null,
        beta: isAngle(solution.beta) ? solution.beta * fromRadiansFactor : null,
        gamma: isAngle(solution.gamma) ? solution.gamma * fromRadiansFactor : null,
        aLength: isLength(solution.a) ? solution.a : null,
        bLength: isLength(solution.b) ? solution.b : null,
        cLength: isLength(solution.c) ? solution.c : null,
      };
    }
  });

  register(['{e75d4624-8ee2-4067-ac8d-c56bdc901d83}', 'right trigonometry', 'rtrig'], {
    type: 'math',
    pinMap: {
      inputs: {
        α: 'alpha', Alpha: 'alpha', alpha: 'alpha',
        β: 'beta', Beta: 'beta', beta: 'beta',
        P: 'p', p: 'p', 'P length': 'p',
        Q: 'q', q: 'q', 'Q length': 'q',
        R: 'r', r: 'r', 'R length': 'r',
      },
      outputs: {
        α: 'alpha', Alpha: 'alpha', alpha: 'alpha',
        β: 'beta', Beta: 'beta', beta: 'beta',
        P: 'p', 'P length': 'p',
        Q: 'q', 'Q length': 'q',
        R: 'r', 'R length': 'r',
      },
    },
    eval: ({ inputs }) => {
      const alphaInput = toNumeric(inputs.alpha);
      const betaInput = toNumeric(inputs.beta);
      const pInput = toNumeric(inputs.p);
      const qInput = toNumeric(inputs.q);
      const rInput = toNumeric(inputs.r);
      const unit = detectAngleUnit([alphaInput, betaInput], Math.PI / 2);
      const toRadiansFactor = unit === 'degrees' ? Math.PI / 180 : 1;
      const fromRadiansFactor = unit === 'degrees' ? 180 / Math.PI : 1;
      const solution = solveRightTriangle({
        alpha: Number.isFinite(alphaInput) ? alphaInput * toRadiansFactor : undefined,
        beta: Number.isFinite(betaInput) ? betaInput * toRadiansFactor : undefined,
        p: isLength(pInput) ? pInput : undefined,
        q: isLength(qInput) ? qInput : undefined,
        r: isLength(rInput) ? rInput : undefined,
      });
      return {
        alpha: isAngle(solution.alpha) ? solution.alpha * fromRadiansFactor : null,
        beta: isAngle(solution.beta) ? solution.beta * fromRadiansFactor : null,
        p: isLength(solution.p) ? solution.p : null,
        q: isLength(solution.q) ? solution.q : null,
        r: isLength(solution.r) ? solution.r : null,
      };
    }
  });
}

export function registerMathBooleanComponents({ register }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math boolean components.');
  }

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
      if (typeof current === 'object' && !current?.isVector3) {
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
      if (!normalized) {
        return fallback;
      }
      if (['true', 'yes', '1', 'on'].includes(normalized)) {
        return true;
      }
      if (['false', 'no', '0', 'off'].includes(normalized)) {
        return false;
      }
      const numeric = Number(normalized);
      if (Number.isFinite(numeric)) {
        return numeric !== 0;
      }
      return fallback;
    }
    if (Array.isArray(resolved)) {
      if (!resolved.length) {
        return fallback;
      }
      return toBoolean(resolved[0], fallback);
    }
    if (resolved === undefined || resolved === null) {
      return fallback;
    }
    if (typeof resolved === 'object') {
      if ('value' in resolved) {
        return toBoolean(resolved.value, fallback);
      }
      if ('values' in resolved) {
        return toBoolean(resolved.values, fallback);
      }
    }
    return fallback;
  }

  const OUTPUT_PINS = { R: 'result', Result: 'result', result: 'result' };
  const BINARY_INPUT_PINS = { A: 'a', a: 'a', B: 'b', b: 'b' };
  const TERNARY_INPUT_PINS = { A: 'a', a: 'a', B: 'b', b: 'b', C: 'c', c: 'c' };
  const UNARY_INPUT_PINS = { A: 'a', a: 'a' };

  function registerBinaryGate(keys, operation) {
    register(keys, {
      type: 'math',
      pinMap: {
        inputs: BINARY_INPUT_PINS,
        outputs: OUTPUT_PINS,
      },
      eval: ({ inputs }) => {
        const a = toBoolean(inputs.a, false);
        const b = toBoolean(inputs.b, false);
        return { result: operation(a, b) };
      }
    });
  }

  function registerUnaryGate(keys, operation) {
    register(keys, {
      type: 'math',
      pinMap: {
        inputs: UNARY_INPUT_PINS,
        outputs: OUTPUT_PINS,
      },
      eval: ({ inputs }) => {
        const value = toBoolean(inputs.a, false);
        return { result: operation(value) };
      }
    });
  }

  function registerTernaryGate(keys, operation) {
    register(keys, {
      type: 'math',
      pinMap: {
        inputs: TERNARY_INPUT_PINS,
        outputs: OUTPUT_PINS,
      },
      eval: ({ inputs }) => {
        const a = toBoolean(inputs.a, false);
        const b = toBoolean(inputs.b, false);
        const c = toBoolean(inputs.c, false);
        return { result: operation(a, b, c) };
      }
    });
  }

  registerBinaryGate([
    '{28f35e12-cd50-4bce-b036-695c2a3d04da}',
    '{040f195d-0b4e-4fe0-901f-fedb2fd3db15}',
    'gate and',
    'and',
  ], (a, b) => a && b);

  registerBinaryGate([
    '{eb3c8610-85b9-4593-a366-52550e8305b7}',
    '{5cad70f9-5a53-4c5c-a782-54a479b4abe3}',
    'gate or',
    'or',
  ], (a, b) => a || b);

  registerBinaryGate([
    '{5ca5de6b-bc71-46c4-a8f7-7f30d7040acb}',
    'gate nand',
    'nand',
  ], (a, b) => !(a && b));

  registerBinaryGate([
    '{548177c2-d1db-4172-b667-bec979e2d38b}',
    'gate nor',
    'nor',
  ], (a, b) => !(a || b));

  registerBinaryGate([
    '{de4a0d86-2709-4564-935a-88bf4d40af89}',
    'gate xor',
    'xor',
  ], (a, b) => (a || b) && !(a && b));

  registerBinaryGate([
    '{b6aedcac-bf43-42d4-899e-d763612f834d}',
    'gate xnor',
    'xnor',
  ], (a, b) => a === b);

  registerUnaryGate([
    '{cb2c7d3c-41b4-4c6d-a6bd-9235bd2851bb}',
    'gate not',
    'not',
  ], (value) => !value);

  registerTernaryGate([
    '{78669f9c-4fea-44fd-ab12-2a69eeec58de}',
    'gate majority',
    'majority',
    'vote',
  ], (a, b, c) => {
    const votes = [a, b, c].filter(Boolean).length;
    return votes >= 2;
  });

  registerTernaryGate([
    '{c1364962-87dd-4a6d-901a-e5b170e5ef9e}',
    'gate and ternary',
    'and ternary',
    'ternary and',
  ], (a, b, c) => a && b && c);

  registerTernaryGate([
    '{55104772-8096-4ffc-a78a-30e36191ace2}',
    'gate or ternary',
    'or ternary',
    'ternary or',
  ], (a, b, c) => a || b || c);
}

export function registerMathPolynomialComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math polynomial components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register math polynomial components.');
  }

  const unaryInputPins = {
    x: 'value',
    X: 'value',
    Value: 'value',
    value: 'value',
    Number: 'value',
    number: 'value',
  };

  const unaryOutputPins = {
    y: 'result',
    Y: 'result',
    Result: 'result',
    result: 'result',
  };

  function registerUnaryPolynomial(keys, compute, { fallback = 0 } = {}) {
    register(keys, {
      type: 'math',
      pinMap: {
        inputs: unaryInputPins,
        outputs: unaryOutputPins,
      },
      eval: ({ inputs }) => {
        const value = toNumber(inputs.value, fallback);
        const result = compute(value);
        return { result: typeof result === 'number' ? result : Number.NaN };
      },
    });
  }

  const safeLog10 = Math.log10 || ((value) => Math.log(value) / Math.LN10);
  const safeCbrt = Math.cbrt || ((value) => {
    if (value === 0) return 0;
    const abs = Math.abs(value);
    const root = Math.pow(abs, 1 / 3);
    return value < 0 ? -root : root;
  });

  registerUnaryPolynomial([
    '{2280dde4-9fa2-4b4a-ae2f-37d554861367}',
    'square',
    'sqr',
  ], (value) => value * value);

  registerUnaryPolynomial([
    '{23afc7aa-2d2f-4ae7-b876-bf366246b826}',
    'natural logarithm',
    'ln',
  ], (value) => {
    if (value < 0) return Number.NaN;
    if (value === 0) return Number.NEGATIVE_INFINITY;
    return Math.log(value);
  }, { fallback: 1 });

  registerUnaryPolynomial([
    '{27d6f724-a701-4585-992f-3897488abf08}',
    'logarithm',
    'log',
    'log10',
  ], (value) => {
    if (value < 0) return Number.NaN;
    if (value === 0) return Number.NEGATIVE_INFINITY;
    return safeLog10(value);
  }, { fallback: 1 });

  registerUnaryPolynomial([
    '{2ebb82ef-1f90-4ac9-9a71-1fe0f4ef7044}',
    'power of 10',
    '10º',
    '10^',
  ], (value) => Math.pow(10, value));

  registerUnaryPolynomial([
    '{5b0be57a-31f5-4446-a11a-ae0d348bca90}',
    'cube root',
    'cbrt',
  ], (value) => safeCbrt(value));

  registerUnaryPolynomial([
    '{797d922f-3a1d-46fe-9155-358b009b5997}',
    'one over x',
    '1/x',
  ], (value) => 1 / value, { fallback: 1 });

  registerUnaryPolynomial([
    '{7a1e5fd7-b7da-4244-a261-f1da66614992}',
    'power of 2',
    '2º',
    '2^',
  ], (value) => Math.pow(2, value));

  register(['{7ab8d289-26a2-4dd4-b4ad-df5b477999d8}', 'log n', 'logn'], {
    type: 'math',
    pinMap: {
      inputs: {
        ...unaryInputPins,
        V: 'value',
        v: 'value',
        Base: 'base',
        base: 'base',
        B: 'base',
      },
      outputs: unaryOutputPins,
    },
    eval: ({ inputs }) => {
      const value = toNumber(inputs.value, Number.NaN);
      const base = toNumber(inputs.base, Number.NaN);
      if (!Number.isFinite(value) || !Number.isFinite(base)) {
        return { result: Number.NaN };
      }
      if (value <= 0 || base <= 0 || base === 1) {
        return { result: Number.NaN };
      }
      return { result: Math.log(value) / Math.log(base) };
    },
  });

  registerUnaryPolynomial([
    '{7e3185eb-a38c-4949-bcf2-0e80dee3a344}',
    'cube',
  ], (value) => value * value * value);

  registerUnaryPolynomial([
    '{ad476cb7-b6d1-41c8-986b-0df243a64146}',
    'square root',
    'sqrt',
  ], (value) => (value < 0 ? Number.NaN : Math.sqrt(value)));

  registerUnaryPolynomial([
    '{c717f26f-e4a0-475c-8e1c-b8f77af1bc99}',
    'power of e',
    'eº',
  ], (value) => Math.exp(value));
}

export function registerMathScriptComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register math script components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register math script components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register math script components.');
  }

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
      if (typeof current === 'object' && !current?.isVector3) {
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

  function ensureVector(value) {
    if (value?.isVector3) {
      return value.clone();
    }
    return toVector3(value, new THREE.Vector3());
  }

  const expressionCache = new Map();
  const RESERVED_IDENTIFIERS = new Set(['if', 'for', 'while', 'switch', 'case', 'return', 'function', 'var', 'let', 'const', 'class']);

  function isValidIdentifier(name) {
    if (typeof name !== 'string' || !name) {
      return false;
    }
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) {
      return false;
    }
    if (RESERVED_IDENTIFIERS.has(name)) {
      return false;
    }
    return true;
  }

  function computeNameVariants(name) {
    if (typeof name !== 'string' || !name) {
      return [];
    }
    const variants = new Set([name, name.toLowerCase(), name.toUpperCase()]);
    const capitalized = name.charAt(0).toUpperCase() + name.slice(1);
    variants.add(capitalized);
    const result = [];
    for (const variant of variants) {
      if (isValidIdentifier(variant)) {
        result.push(variant);
      }
    }
    return result;
  }

  function toExpressionString(value, depth = 0) {
    if (depth > 8) {
      return '';
    }
    if (value === undefined || value === null) {
      return '';
    }
    if (typeof value === 'string') {
      return value;
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value);
    }
    if (Array.isArray(value)) {
      for (let index = 0; index < value.length; index += 1) {
        const candidate = toExpressionString(value[index], depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      return '';
    }
    if (typeof value === 'object') {
      if ('expression' in value) {
        const candidate = toExpressionString(value.expression, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('code' in value) {
        const candidate = toExpressionString(value.code, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('text' in value) {
        const candidate = toExpressionString(value.text, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('value' in value) {
        const candidate = toExpressionString(value.value, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('values' in value) {
        const candidate = toExpressionString(value.values, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if (typeof value.toString === 'function' && value.toString !== Object.prototype.toString) {
        const text = `${value}`;
        if (text && text !== '[object Object]') {
          return text;
        }
      }
      return '';
    }
    return '';
  }

  function normalizeExpressionSource(source) {
    if (typeof source !== 'string') {
      return '';
    }
    const trimmed = source.trim();
    if (!trimmed) {
      return '';
    }
    return trimmed
      .replace(/<>/g, '!=')
      .replace(/\^/g, '**')
      .replace(/;+\s*$/g, '');
  }

  const expressionContext = new Map();

  function addContextEntry(name, value) {
    if (!isValidIdentifier(name)) {
      return;
    }
    if (!expressionContext.has(name)) {
      expressionContext.set(name, value);
    }
  }

  function registerContextVariants(baseName, value) {
    for (const variant of computeNameVariants(baseName)) {
      addContextEntry(variant, value);
    }
  }

  const safeSinh = Math.sinh ?? ((v) => (Math.exp(v) - Math.exp(-v)) / 2);
  const safeCosh = Math.cosh ?? ((v) => (Math.exp(v) + Math.exp(-v)) / 2);
  const safeTanh = Math.tanh ?? ((v) => {
    const ePos = Math.exp(v);
    const eNeg = Math.exp(-v);
    return (ePos - eNeg) / (ePos + eNeg);
  });
  const safeAsinh = Math.asinh ?? ((v) => Math.log(v + Math.sqrt(v * v + 1)));
  const safeAcosh = Math.acosh ?? ((v) => Math.log(v + Math.sqrt(v * v - 1)));
  const safeAtanh = Math.atanh ?? ((v) => 0.5 * Math.log((1 + v) / (1 - v)));
  const safeHypot = Math.hypot ?? ((...values) => Math.sqrt(values.reduce((sum, entry) => sum + entry * entry, 0)));

  const signFunction = Math.sign ?? ((v) => {
    const numeric = Number(v);
    if (Number.isNaN(numeric)) {
      return 0;
    }
    if (numeric > 0) return 1;
    if (numeric < 0) return -1;
    return 0;
  });

  const moduloFunction = (a, b) => {
    const dividend = Number(a);
    const divisor = Number(b);
    if (!Number.isFinite(dividend) || !Number.isFinite(divisor) || divisor === 0) {
      return Number.NaN;
    }
    const remainder = dividend % divisor;
    if (remainder === 0) {
      return 0;
    }
    return remainder < 0 ? remainder + (divisor < 0 ? -divisor : divisor) : remainder;
  };

  const clampFunction = (value, min = 0, max = 1) => {
    const numericValue = Number(value);
    const numericMin = Number(min);
    const numericMax = Number(max);
    if (!Number.isFinite(numericValue) || !Number.isFinite(numericMin) || !Number.isFinite(numericMax)) {
      return Number.NaN;
    }
    const lower = Math.min(numericMin, numericMax);
    const upper = Math.max(numericMin, numericMax);
    if (numericValue <= lower) return lower;
    if (numericValue >= upper) return upper;
    return numericValue;
  };

  const lerpFunction = (a = 0, b = 0, t = 0) => {
    const start = Number(a);
    const end = Number(b);
    const parameter = Number(t);
    if (!Number.isFinite(start) || !Number.isFinite(end) || !Number.isFinite(parameter)) {
      return Number.NaN;
    }
    return start + (end - start) * parameter;
  };

  const degFunction = (value) => Number(value) * (180 / Math.PI);
  const radFunction = (value) => Number(value) * (Math.PI / 180);
  const fracFunction = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) {
      return Number.NaN;
    }
    return numeric - Math.trunc(numeric);
  };
  const randomFunction = (min = 0, max = 1) => {
    const numericMin = Number(min);
    const numericMax = Number(max);
    if (!Number.isFinite(numericMin) || !Number.isFinite(numericMax)) {
      return Math.random();
    }
    if (numericMin === numericMax) {
      return numericMin;
    }
    const lower = Math.min(numericMin, numericMax);
    const upper = Math.max(numericMin, numericMax);
    return lower + Math.random() * (upper - lower);
  };

  registerContextVariants('abs', Math.abs);
  registerContextVariants('sign', signFunction);
  registerContextVariants('sgn', signFunction);
  registerContextVariants('floor', Math.floor);
  registerContextVariants('ceil', Math.ceil);
  registerContextVariants('ceiling', Math.ceil);
  registerContextVariants('round', Math.round);
  registerContextVariants('trunc', Math.trunc ?? ((v) => (v < 0 ? Math.ceil(v) : Math.floor(v))));
  registerContextVariants('frac', fracFunction);
  registerContextVariants('sqrt', Math.sqrt);
  registerContextVariants('power', Math.pow);
  registerContextVariants('pow', Math.pow);
  registerContextVariants('exp', Math.exp);
  registerContextVariants('ln', Math.log);
  registerContextVariants('log', Math.log);
  registerContextVariants('log10', Math.log10 ?? ((v) => Math.log(v) / Math.LN10));
  registerContextVariants('log2', Math.log2 ?? ((v) => Math.log(v) / Math.LN2));
  registerContextVariants('sin', Math.sin);
  registerContextVariants('cos', Math.cos);
  registerContextVariants('tan', Math.tan);
  registerContextVariants('asin', Math.asin);
  registerContextVariants('acos', Math.acos);
  registerContextVariants('atan', Math.atan);
  registerContextVariants('atan2', Math.atan2);
  registerContextVariants('sinh', safeSinh);
  registerContextVariants('cosh', safeCosh);
  registerContextVariants('tanh', safeTanh);
  registerContextVariants('asinh', safeAsinh);
  registerContextVariants('acosh', safeAcosh);
  registerContextVariants('atanh', safeAtanh);
  registerContextVariants('hypot', safeHypot);
  registerContextVariants('min', (...values) => Math.min(...values));
  registerContextVariants('max', (...values) => Math.max(...values));
  registerContextVariants('clamp', clampFunction);
  registerContextVariants('lerp', lerpFunction);
  registerContextVariants('deg', degFunction);
  registerContextVariants('rad', radFunction);
  registerContextVariants('random', randomFunction);
  registerContextVariants('rand', randomFunction);
  registerContextVariants('mod', moduloFunction);
  registerContextVariants('modulo', moduloFunction);
  registerContextVariants('sec', (value) => 1 / Math.cos(Number(value)));
  registerContextVariants('csc', (value) => 1 / Math.sin(Number(value)));
  registerContextVariants('cot', (value) => 1 / Math.tan(Number(value)));
  registerContextVariants('and', (a, b) => (toBoolean(a) && toBoolean(b)) ? 1 : 0);
  registerContextVariants('or', (a, b) => (toBoolean(a) || toBoolean(b)) ? 1 : 0);
  registerContextVariants('xor', (a, b) => {
    const left = toBoolean(a);
    const right = toBoolean(b);
    return left !== right ? 1 : 0;
  });
  registerContextVariants('not', (value) => (toBoolean(value) ? 0 : 1));
  registerContextVariants('if', (condition, whenTrue, whenFalse = 0) => (toBoolean(condition) ? whenTrue : whenFalse));
  registerContextVariants('select', (condition, whenTrue, whenFalse = 0) => (toBoolean(condition) ? whenTrue : whenFalse));

  const phiConstant = (1 + Math.sqrt(5)) / 2;
  const constants = [
    ['Pi', Math.PI],
    ['Tau', Math.PI * 2],
    ['E', Math.E],
    ['Phi', phiConstant],
  ];
  for (const [name, value] of constants) {
    for (const variant of computeNameVariants(name)) {
      addContextEntry(variant, value);
    }
  }

  const expressionContextEntries = Array.from(expressionContext.entries());
  const expressionContextNames = expressionContextEntries.map(([name]) => name);
  const expressionContextValues = expressionContextEntries.map(([, value]) => value);

  function compileExpression(source, argNames) {
    const key = `${argNames.join('|')}::${source}`;
    if (expressionCache.has(key)) {
      return expressionCache.get(key);
    }
    try {
      const evaluator = new Function(
        ...argNames,
        ...expressionContextNames,
        `'use strict'; return (${source});`
      );
      const compiled = (valueMap) => {
        const args = argNames.map((name) => valueMap.get(name) ?? 0);
        return evaluator(...args, ...expressionContextValues);
      };
      expressionCache.set(key, compiled);
      return compiled;
    } catch (error) {
      expressionCache.set(key, null);
      return null;
    }
  }

  function prepareExpressionVariables(variableNames, variableVariants, inputs) {
    const valueMap = new Map();
    const baseValues = [];
    for (let index = 0; index < variableNames.length; index += 1) {
      const baseName = variableNames[index];
      const variants = variableVariants[index];
      const numeric = toNumber(unwrapSingle(inputs[baseName]), Number.NaN);
      const value = Number.isFinite(numeric) ? numeric : 0;
      baseValues.push(value);
      for (const variant of variants) {
        valueMap.set(variant, value);
      }
    }
    return { valueMap, baseValues };
  }

  function executeExpression(expressionValue, compileOrder, valueMap, baseValues) {
    if (typeof expressionValue === 'function') {
      try {
        return expressionValue(...baseValues);
      } catch (error) {
        return null;
      }
    }
    const expressionString = normalizeExpressionSource(toExpressionString(expressionValue));
    if (!expressionString) {
      return null;
    }
    const evaluator = compileExpression(expressionString, compileOrder);
    if (!evaluator) {
      return null;
    }
    try {
      return evaluator(valueMap);
    } catch (error) {
      return null;
    }
  }

  function normalizeExpressionResult(result) {
    if (result === undefined || result === null) {
      return null;
    }
    if (isVectorLike(result)) {
      return ensureVector(result);
    }
    if (typeof result === 'boolean') {
      return result ? 1 : 0;
    }
    const numeric = Number(result);
    if (!Number.isNaN(numeric)) {
      return numeric;
    }
    if (typeof result === 'number' && Number.isNaN(result)) {
      return Number.NaN;
    }
    return null;
  }

  function registerExpressionComponent(identifiers, variableNames) {
    const variableVariants = variableNames.map((name) => computeNameVariants(name));
    const compileOrder = [];
    for (const variants of variableVariants) {
      for (const variant of variants) {
        if (!compileOrder.includes(variant)) {
          compileOrder.push(variant);
        }
      }
    }

    const inputMap = {
      F: 'expression',
      f: 'expression',
      Function: 'expression',
      function: 'expression',
      Expression: 'expression',
      expression: 'expression',
      Expr: 'expression',
      expr: 'expression',
      Formula: 'expression',
      formula: 'expression',
      Equation: 'expression',
      equation: 'expression',
    };

    for (let index = 0; index < variableNames.length; index += 1) {
      const baseName = variableNames[index];
      const variants = variableVariants[index];
      for (const variant of variants) {
        inputMap[variant] = baseName;
      }
      inputMap[`Variable ${baseName}`] = baseName;
      inputMap[`variable ${baseName}`] = baseName;
      const upper = baseName.toUpperCase();
      inputMap[`Variable ${upper}`] = baseName;
      const capitalized = baseName.charAt(0).toUpperCase() + baseName.slice(1);
      inputMap[`Variable ${capitalized}`] = baseName;
      inputMap[`Var ${baseName}`] = baseName;
      inputMap[`var ${baseName}`] = baseName;
      inputMap[`Var ${upper}`] = baseName;
      inputMap[`var ${upper}`] = baseName;
      inputMap[`Var ${capitalized}`] = baseName;
      inputMap[`var ${capitalized}`] = baseName;
    }

    const outputMap = {
      Result: 'result',
      result: 'result',
      R: 'result',
      r: 'result',
      Y: 'result',
      y: 'result',
      Output: 'result',
      output: 'result',
      Out: 'result',
      out: 'result',
    };

    register(identifiers, {
      type: 'math',
      pinMap: {
        inputs: inputMap,
        outputs: outputMap,
      },
      eval: ({ inputs }) => {
        const { valueMap, baseValues } = prepareExpressionVariables(variableNames, variableVariants, inputs);
        const expressionValue = unwrapSingle(inputs.expression);
        const rawResult = executeExpression(expressionValue, compileOrder, valueMap, baseValues);
        const result = normalizeExpressionResult(rawResult);
        if (result === null) {
          return {};
        }
        return { result };
      }
    });
  }

  registerExpressionComponent(
    ['{0b7d1129-7b88-4322-aad3-56fd1036a8f6}', 'f1', 'f(x)'],
    ['x']
  );

  registerExpressionComponent(
    ['{00ec9ecd-4e1d-45ba-a8fc-dff716dbd9e4}', 'f2', 'f(x,y)'],
    ['x', 'y']
  );

  registerExpressionComponent(
    ['{2f77b45b-034d-4053-8872-f38d87cbc676}', 'f3', 'f(x,y,z)'],
    ['x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{07efd5e1-d7f4-4205-ab99-83e68175564e}', 'f4', 'f(a,b,c,d)'],
    ['a', 'b', 'c', 'd']
  );

  registerExpressionComponent(
    ['{322f0e6e-d434-4d07-9f8d-f214bb248cb1}', 'f5', 'f(a,b,c,d,x)'],
    ['a', 'b', 'c', 'd', 'x']
  );

  registerExpressionComponent(
    ['{4783b96f-6197-4058-a688-b4ba04c00962}', 'f6', 'f(a,b,c,d,x,y)'],
    ['a', 'b', 'c', 'd', 'x', 'y']
  );

  registerExpressionComponent(
    ['{e9628b21-49d6-4e56-900e-49f4bd4adc85}', 'f7', 'f(a,b,c,d,x,y,z)'],
    ['a', 'b', 'c', 'd', 'x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{f2a97ac6-4f11-4c81-834d-50ecd782675c}', 'f8', 'f(a,b,c,d,w,x,y,z)'],
    ['a', 'b', 'c', 'd', 'w', 'x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{0f3a13d4-5bb7-499e-9b57-56bb6dce93fd}', 'f(a,b,c,d) obsolete', 'f4 obsolete'],
    ['a', 'b', 'c', 'd']
  );

  registerExpressionComponent(
    ['{d2b10b82-f612-4763-91ca-0cbdbe276171}', 'f(x,y) obsolete', 'f2 obsolete'],
    ['x', 'y']
  );

  registerExpressionComponent(
    ['{d3e721b4-f5ea-4e40-85fc-b68616939e47}', 'f(x) obsolete', 'f1 obsolete'],
    ['x']
  );

  registerExpressionComponent(
    ['{e1c4bccc-4ecf-4f18-885d-dfd8983e572a}', 'f(x,y,z) obsolete', 'f3 obsolete'],
    ['x', 'y', 'z']
  );
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

  const expressionCache = new Map();
  const RESERVED_IDENTIFIERS = new Set(['if', 'for', 'while', 'switch', 'case', 'return', 'function', 'var', 'let', 'const', 'class']);

  function isValidIdentifier(name) {
    if (typeof name !== 'string' || !name) {
      return false;
    }
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) {
      return false;
    }
    if (RESERVED_IDENTIFIERS.has(name)) {
      return false;
    }
    return true;
  }

  function computeNameVariants(name) {
    if (typeof name !== 'string' || !name) {
      return [];
    }
    const variants = new Set([name, name.toLowerCase(), name.toUpperCase()]);
    const capitalized = name.charAt(0).toUpperCase() + name.slice(1);
    variants.add(capitalized);
    const result = [];
    for (const variant of variants) {
      if (isValidIdentifier(variant)) {
        result.push(variant);
      }
    }
    return result;
  }

  function toExpressionString(value, depth = 0) {
    if (depth > 8) {
      return '';
    }
    if (value === undefined || value === null) {
      return '';
    }
    if (typeof value === 'string') {
      return value;
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value);
    }
    if (Array.isArray(value)) {
      for (let index = 0; index < value.length; index += 1) {
        const candidate = toExpressionString(value[index], depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      return '';
    }
    if (typeof value === 'object') {
      if ('expression' in value) {
        const candidate = toExpressionString(value.expression, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('code' in value) {
        const candidate = toExpressionString(value.code, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('text' in value) {
        const candidate = toExpressionString(value.text, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('value' in value) {
        const candidate = toExpressionString(value.value, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if ('values' in value) {
        const candidate = toExpressionString(value.values, depth + 1);
        if (candidate) {
          return candidate;
        }
      }
      if (typeof value.toString === 'function' && value.toString !== Object.prototype.toString) {
        const text = `${value}`;
        if (text && text !== '[object Object]') {
          return text;
        }
      }
      return '';
    }
    return '';
  }

  function normalizeExpressionSource(source) {
    if (typeof source !== 'string') {
      return '';
    }
    const trimmed = source.trim();
    if (!trimmed) {
      return '';
    }
    return trimmed
      .replace(/<>/g, '!=')
      .replace(/\^/g, '**')
      .replace(/;+\s*$/g, '');
  }

  const expressionContext = new Map();

  function addContextEntry(name, value) {
    if (!isValidIdentifier(name)) {
      return;
    }
    if (!expressionContext.has(name)) {
      expressionContext.set(name, value);
    }
  }

  function registerContextVariants(baseName, value) {
    for (const variant of computeNameVariants(baseName)) {
      addContextEntry(variant, value);
    }
  }

  const safeSinh = Math.sinh ?? ((v) => (Math.exp(v) - Math.exp(-v)) / 2);
  const safeCosh = Math.cosh ?? ((v) => (Math.exp(v) + Math.exp(-v)) / 2);
  const safeTanh = Math.tanh ?? ((v) => {
    const ePos = Math.exp(v);
    const eNeg = Math.exp(-v);
    return (ePos - eNeg) / (ePos + eNeg);
  });
  const safeAsinh = Math.asinh ?? ((v) => Math.log(v + Math.sqrt(v * v + 1)));
  const safeAcosh = Math.acosh ?? ((v) => Math.log(v + Math.sqrt(v * v - 1)));
  const safeAtanh = Math.atanh ?? ((v) => 0.5 * Math.log((1 + v) / (1 - v)));
  const safeHypot = Math.hypot ?? ((...values) => Math.sqrt(values.reduce((sum, entry) => sum + entry * entry, 0)));

  const signFunction = Math.sign ?? ((v) => {
    const numeric = Number(v);
    if (Number.isNaN(numeric)) {
      return 0;
    }
    if (numeric > 0) return 1;
    if (numeric < 0) return -1;
    return 0;
  });

  const moduloFunction = (a, b) => {
    const dividend = Number(a);
    const divisor = Number(b);
    if (!Number.isFinite(dividend) || !Number.isFinite(divisor) || divisor === 0) {
      return Number.NaN;
    }
    const remainder = dividend % divisor;
    if (remainder === 0) {
      return 0;
    }
    return remainder < 0 ? remainder + (divisor < 0 ? -divisor : divisor) : remainder;
  };

  const clampFunction = (value, min = 0, max = 1) => {
    const numericValue = Number(value);
    const numericMin = Number(min);
    const numericMax = Number(max);
    if (!Number.isFinite(numericValue) || !Number.isFinite(numericMin) || !Number.isFinite(numericMax)) {
      return Number.NaN;
    }
    const lower = Math.min(numericMin, numericMax);
    const upper = Math.max(numericMin, numericMax);
    if (numericValue <= lower) return lower;
    if (numericValue >= upper) return upper;
    return numericValue;
  };

  const lerpFunction = (a = 0, b = 0, t = 0) => {
    const start = Number(a);
    const end = Number(b);
    const parameter = Number(t);
    if (!Number.isFinite(start) || !Number.isFinite(end) || !Number.isFinite(parameter)) {
      return Number.NaN;
    }
    return start + (end - start) * parameter;
  };

  const degFunction = (value) => Number(value) * (180 / Math.PI);
  const radFunction = (value) => Number(value) * (Math.PI / 180);
  const fracFunction = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) {
      return Number.NaN;
    }
    return numeric - Math.trunc(numeric);
  };
  const randomFunction = (min = 0, max = 1) => {
    const numericMin = Number(min);
    const numericMax = Number(max);
    if (!Number.isFinite(numericMin) || !Number.isFinite(numericMax)) {
      return Math.random();
    }
    if (numericMin === numericMax) {
      return numericMin;
    }
    const lower = Math.min(numericMin, numericMax);
    const upper = Math.max(numericMin, numericMax);
    return lower + Math.random() * (upper - lower);
  };

  registerContextVariants('abs', Math.abs);
  registerContextVariants('sign', signFunction);
  registerContextVariants('sgn', signFunction);
  registerContextVariants('floor', Math.floor);
  registerContextVariants('ceil', Math.ceil);
  registerContextVariants('ceiling', Math.ceil);
  registerContextVariants('round', Math.round);
  registerContextVariants('trunc', Math.trunc ?? ((v) => (v < 0 ? Math.ceil(v) : Math.floor(v))));
  registerContextVariants('frac', fracFunction);
  registerContextVariants('sqrt', Math.sqrt);
  registerContextVariants('power', Math.pow);
  registerContextVariants('pow', Math.pow);
  registerContextVariants('exp', Math.exp);
  registerContextVariants('ln', Math.log);
  registerContextVariants('log', Math.log);
  registerContextVariants('log10', Math.log10 ?? ((v) => Math.log(v) / Math.LN10));
  registerContextVariants('log2', Math.log2 ?? ((v) => Math.log(v) / Math.LN2));
  registerContextVariants('sin', Math.sin);
  registerContextVariants('cos', Math.cos);
  registerContextVariants('tan', Math.tan);
  registerContextVariants('asin', Math.asin);
  registerContextVariants('acos', Math.acos);
  registerContextVariants('atan', Math.atan);
  registerContextVariants('atan2', Math.atan2);
  registerContextVariants('sinh', safeSinh);
  registerContextVariants('cosh', safeCosh);
  registerContextVariants('tanh', safeTanh);
  registerContextVariants('asinh', safeAsinh);
  registerContextVariants('acosh', safeAcosh);
  registerContextVariants('atanh', safeAtanh);
  registerContextVariants('hypot', safeHypot);
  registerContextVariants('min', (...values) => Math.min(...values));
  registerContextVariants('max', (...values) => Math.max(...values));
  registerContextVariants('clamp', clampFunction);
  registerContextVariants('lerp', lerpFunction);
  registerContextVariants('deg', degFunction);
  registerContextVariants('rad', radFunction);
  registerContextVariants('random', randomFunction);
  registerContextVariants('rand', randomFunction);
  registerContextVariants('mod', moduloFunction);
  registerContextVariants('modulo', moduloFunction);
  registerContextVariants('sec', (value) => 1 / Math.cos(Number(value)));
  registerContextVariants('csc', (value) => 1 / Math.sin(Number(value)));
  registerContextVariants('cot', (value) => 1 / Math.tan(Number(value)));
  registerContextVariants('and', (a, b) => (toBoolean(a) && toBoolean(b)) ? 1 : 0);
  registerContextVariants('or', (a, b) => (toBoolean(a) || toBoolean(b)) ? 1 : 0);
  registerContextVariants('xor', (a, b) => {
    const left = toBoolean(a);
    const right = toBoolean(b);
    return left !== right ? 1 : 0;
  });
  registerContextVariants('not', (value) => (toBoolean(value) ? 0 : 1));
  registerContextVariants('if', (condition, whenTrue, whenFalse = 0) => (toBoolean(condition) ? whenTrue : whenFalse));
  registerContextVariants('select', (condition, whenTrue, whenFalse = 0) => (toBoolean(condition) ? whenTrue : whenFalse));

  const phiConstant = (1 + Math.sqrt(5)) / 2;
  const constants = [
    ['Pi', Math.PI],
    ['Tau', Math.PI * 2],
    ['E', Math.E],
    ['Phi', phiConstant],
  ];
  for (const [name, value] of constants) {
    for (const variant of computeNameVariants(name)) {
      addContextEntry(variant, value);
    }
  }

  const expressionContextEntries = Array.from(expressionContext.entries());
  const expressionContextNames = expressionContextEntries.map(([name]) => name);
  const expressionContextValues = expressionContextEntries.map(([, value]) => value);

  function compileExpression(source, argNames) {
    const key = `${argNames.join('|')}::${source}`;
    if (expressionCache.has(key)) {
      return expressionCache.get(key);
    }
    try {
      const evaluator = new Function(
        ...argNames,
        ...expressionContextNames,
        `'use strict'; return (${source});`
      );
      const compiled = (valueMap) => {
        const args = argNames.map((name) => valueMap.get(name) ?? 0);
        return evaluator(...args, ...expressionContextValues);
      };
      expressionCache.set(key, compiled);
      return compiled;
    } catch (error) {
      expressionCache.set(key, null);
      return null;
    }
  }

  function prepareExpressionVariables(variableNames, variableVariants, inputs) {
    const valueMap = new Map();
    const baseValues = [];
    for (let index = 0; index < variableNames.length; index += 1) {
      const baseName = variableNames[index];
      const variants = variableVariants[index];
      const numeric = toNumber(unwrapSingle(inputs[baseName]), Number.NaN);
      const value = Number.isFinite(numeric) ? numeric : 0;
      baseValues.push(value);
      for (const variant of variants) {
        valueMap.set(variant, value);
      }
    }
    return { valueMap, baseValues };
  }

  function executeExpression(expressionValue, compileOrder, valueMap, baseValues) {
    if (typeof expressionValue === 'function') {
      try {
        return expressionValue(...baseValues);
      } catch (error) {
        return null;
      }
    }
    const expressionString = normalizeExpressionSource(toExpressionString(expressionValue));
    if (!expressionString) {
      return null;
    }
    const evaluator = compileExpression(expressionString, compileOrder);
    if (!evaluator) {
      return null;
    }
    try {
      return evaluator(valueMap);
    } catch (error) {
      return null;
    }
  }

  function normalizeExpressionResult(result) {
    if (result === undefined || result === null) {
      return null;
    }
    if (isVectorLike(result)) {
      return ensureVector(result);
    }
    if (typeof result === 'boolean') {
      return result ? 1 : 0;
    }
    const numeric = Number(result);
    if (!Number.isNaN(numeric)) {
      return numeric;
    }
    if (typeof result === 'number' && Number.isNaN(result)) {
      return Number.NaN;
    }
    return null;
  }

  function registerExpressionComponent(identifiers, variableNames) {
    const variableVariants = variableNames.map((name) => computeNameVariants(name));
    const compileOrder = [];
    for (const variants of variableVariants) {
      for (const variant of variants) {
        if (!compileOrder.includes(variant)) {
          compileOrder.push(variant);
        }
      }
    }

    const inputMap = {
      F: 'expression',
      f: 'expression',
      Function: 'expression',
      function: 'expression',
      Expression: 'expression',
      expression: 'expression',
      Expr: 'expression',
      expr: 'expression',
      Formula: 'expression',
      formula: 'expression',
      Equation: 'expression',
      equation: 'expression',
    };

    for (let index = 0; index < variableNames.length; index += 1) {
      const baseName = variableNames[index];
      const variants = variableVariants[index];
      for (const variant of variants) {
        inputMap[variant] = baseName;
      }
      inputMap[`Variable ${baseName}`] = baseName;
      inputMap[`variable ${baseName}`] = baseName;
      const upper = baseName.toUpperCase();
      inputMap[`Variable ${upper}`] = baseName;
      const capitalized = baseName.charAt(0).toUpperCase() + baseName.slice(1);
      inputMap[`Variable ${capitalized}`] = baseName;
      inputMap[`Var ${baseName}`] = baseName;
      inputMap[`var ${baseName}`] = baseName;
      inputMap[`Var ${upper}`] = baseName;
      inputMap[`var ${upper}`] = baseName;
      inputMap[`Var ${capitalized}`] = baseName;
      inputMap[`var ${capitalized}`] = baseName;
    }

    const outputMap = {
      Result: 'result',
      result: 'result',
      R: 'result',
      r: 'result',
      Y: 'result',
      y: 'result',
      Output: 'result',
      output: 'result',
      Out: 'result',
      out: 'result',
    };

    register(identifiers, {
      type: 'math',
      pinMap: {
        inputs: inputMap,
        outputs: outputMap,
      },
      eval: ({ inputs }) => {
        const { valueMap, baseValues } = prepareExpressionVariables(variableNames, variableVariants, inputs);
        const expressionValue = unwrapSingle(inputs.expression);
        const rawResult = executeExpression(expressionValue, compileOrder, valueMap, baseValues);
        const result = normalizeExpressionResult(rawResult);
        if (result === null) {
          return {};
        }
        return { result };
      }
    });
  }

  registerExpressionComponent(
    ['{0b7d1129-7b88-4322-aad3-56fd1036a8f6}', 'f1', 'f(x)'],
    ['x']
  );

  registerExpressionComponent(
    ['{00ec9ecd-4e1d-45ba-a8fc-dff716dbd9e4}', 'f2', 'f(x,y)'],
    ['x', 'y']
  );

  registerExpressionComponent(
    ['{2f77b45b-034d-4053-8872-f38d87cbc676}', 'f3', 'f(x,y,z)'],
    ['x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{07efd5e1-d7f4-4205-ab99-83e68175564e}', 'f4', 'f(a,b,c,d)'],
    ['a', 'b', 'c', 'd']
  );

  registerExpressionComponent(
    ['{322f0e6e-d434-4d07-9f8d-f214bb248cb1}', 'f5', 'f(a,b,c,d,x)'],
    ['a', 'b', 'c', 'd', 'x']
  );

  registerExpressionComponent(
    ['{4783b96f-6197-4058-a688-b4ba04c00962}', 'f6', 'f(a,b,c,d,x,y)'],
    ['a', 'b', 'c', 'd', 'x', 'y']
  );

  registerExpressionComponent(
    ['{e9628b21-49d6-4e56-900e-49f4bd4adc85}', 'f7', 'f(a,b,c,d,x,y,z)'],
    ['a', 'b', 'c', 'd', 'x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{f2a97ac6-4f11-4c81-834d-50ecd782675c}', 'f8', 'f(a,b,c,d,w,x,y,z)'],
    ['a', 'b', 'c', 'd', 'w', 'x', 'y', 'z']
  );

  registerExpressionComponent(
    ['{0f3a13d4-5bb7-499e-9b57-56bb6dce93fd}', 'f(a,b,c,d) obsolete', 'f4 obsolete'],
    ['a', 'b', 'c', 'd']
  );

  registerExpressionComponent(
    ['{d2b10b82-f612-4763-91ca-0cbdbe276171}', 'f(x,y) obsolete', 'f2 obsolete'],
    ['x', 'y']
  );

  registerExpressionComponent(
    ['{d3e721b4-f5ea-4e40-85fc-b68616939e47}', 'f(x) obsolete', 'f1 obsolete'],
    ['x']
  );

  registerExpressionComponent(
    ['{e1c4bccc-4ecf-4f18-885d-dfd8983e572a}', 'f(x,y,z) obsolete', 'f3 obsolete'],
    ['x', 'y', 'z']
  );

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

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

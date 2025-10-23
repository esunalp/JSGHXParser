const DEFAULT_COMPLEX_EPSILON = 1e-12;

function createComplexToolkit(toNumber, options = {}) {
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to create complex toolkit.');
  }

  const epsilonOption = options?.epsilon;
  const EPSILON = Number.isFinite(epsilonOption) && epsilonOption > 0 ? epsilonOption : DEFAULT_COMPLEX_EPSILON;
  const ZERO_COMPLEX = Object.freeze({ real: 0, imag: 0 });

  function toFiniteNumber(value, fallback = 0) {
    const numeric = toNumber(value, Number.NaN);
    return Number.isFinite(numeric) ? numeric : fallback;
  }

  function createComplex(real, imag) {
    const result = { real, imag };
    if (Number.isFinite(real) && Number.isFinite(imag)) {
      result.magnitude = Math.hypot(real, imag);
      result.argument = Math.atan2(imag, real);
    } else {
      result.magnitude = Number.NaN;
      result.argument = Number.NaN;
    }
    return result;
  }

  function parseComplexString(raw) {
    if (typeof raw !== 'string') {
      return null;
    }
    const trimmed = raw.trim();
    if (!trimmed) {
      return null;
    }
    const sanitized = trimmed.replace(/\s+/g, '');
    if (!sanitized) {
      return null;
    }
    if (sanitized === 'i' || sanitized === '+i') {
      return createComplex(0, 1);
    }
    if (sanitized === '-i') {
      return createComplex(0, -1);
    }
    const iIndex = sanitized.indexOf('i');
    if (iIndex === -1) {
      const realOnly = toNumber(trimmed, Number.NaN);
      return Number.isFinite(realOnly) ? createComplex(realOnly, 0) : null;
    }
    const withoutI = sanitized.slice(0, iIndex);
    if (!withoutI) {
      return createComplex(0, 1);
    }
    let realPartText = '';
    let imagPartText = withoutI;
    for (let idx = withoutI.length - 1; idx > 0; idx--) {
      const char = withoutI[idx];
      if (char === '+' || char === '-') {
        realPartText = withoutI.slice(0, idx);
        imagPartText = withoutI.slice(idx);
        break;
      }
    }
    const parseImagPart = (segment) => {
      if (!segment || segment === '+') {
        return 1;
      }
      if (segment === '-') {
        return -1;
      }
      const numeric = toNumber(segment, Number.NaN);
      return Number.isFinite(numeric) ? numeric : null;
    };
    if (!realPartText) {
      const imagOnly = parseImagPart(imagPartText);
      if (imagOnly !== null) {
        return createComplex(0, imagOnly);
      }
      const realFallback = toNumber(withoutI, Number.NaN);
      return Number.isFinite(realFallback) ? createComplex(realFallback, 0) : null;
    }
    const realValue = toNumber(realPartText, Number.NaN);
    const imagValue = parseImagPart(imagPartText);
    if (Number.isFinite(realValue) && imagValue !== null) {
      return createComplex(realValue, imagValue);
    }
    return null;
  }

  function ensureComplex(value, fallback = ZERO_COMPLEX) {
    const fallbackReal = toFiniteNumber(fallback?.real, 0);
    const fallbackImag = toFiniteNumber(fallback?.imag, 0);
    const useFallback = () => createComplex(fallbackReal, fallbackImag);

    if (value === undefined || value === null) {
      return useFallback();
    }

    if (Array.isArray(value)) {
      if (value.length === 0) {
        return useFallback();
      }
      if (value.length >= 2) {
        const realPart = toFiniteNumber(value[0], fallbackReal);
        const imagPart = toFiniteNumber(value[1], fallbackImag);
        return createComplex(realPart, imagPart);
      }
      return ensureComplex(value[0], { real: fallbackReal, imag: fallbackImag });
    }

    if (typeof value === 'object') {
      if (Object.prototype.hasOwnProperty.call(value, 'value')) {
        return ensureComplex(value.value, { real: fallbackReal, imag: fallbackImag });
      }

      const realCandidates = [
        value.real,
        value.Real,
        value.RE,
        value.re,
        value.x,
        value.X,
        value.a,
        value.A,
      ];
      const imagCandidates = [
        value.imag,
        value.Imag,
        value.IM,
        value.im,
        value.i,
        value.I,
        value.y,
        value.Y,
        value.b,
        value.B,
      ];

      if (realCandidates.some((entry) => entry !== undefined) || imagCandidates.some((entry) => entry !== undefined)) {
        const realPart = toFiniteNumber(realCandidates.find((entry) => entry !== undefined), fallbackReal);
        const imagPart = toFiniteNumber(imagCandidates.find((entry) => entry !== undefined), fallbackImag);
        return createComplex(realPart, imagPart);
      }

      const magnitudeCandidate = value.magnitude ?? value.modulus ?? value.abs ?? value.r ?? value.radius;
      const angleCandidate = value.argument ?? value.angle ?? value.phase ?? value.theta ?? value.phi;
      if (magnitudeCandidate !== undefined) {
        const magnitude = toFiniteNumber(magnitudeCandidate, 0);
        const angle = toFiniteNumber(angleCandidate, 0);
        const realPart = magnitude * Math.cos(angle);
        const imagPart = magnitude * Math.sin(angle);
        return createComplex(realPart, imagPart);
      }

      const nested = value.values ?? value.coords ?? value.components;
      if (nested !== undefined) {
        return ensureComplex(nested, { real: fallbackReal, imag: fallbackImag });
      }

      const numeric = toNumber(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        return createComplex(numeric, 0);
      }
      return useFallback();
    }

    if (typeof value === 'string') {
      const parsed = parseComplexString(value);
      if (parsed) {
        return parsed;
      }
      const numeric = toNumber(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        return createComplex(numeric, 0);
      }
      return useFallback();
    }

    if (typeof value === 'number') {
      if (Number.isFinite(value)) {
        return createComplex(value, 0);
      }
      return useFallback();
    }

    return useFallback();
  }

  function isApproximatelyZero(value) {
    return Math.abs(value) <= EPSILON;
  }

  function isZeroComplex(value) {
    return isApproximatelyZero(value.real) && isApproximatelyZero(value.imag);
  }

  function addComplex(a, b) {
    return createComplex(a.real + b.real, a.imag + b.imag);
  }

  function subtractComplex(a, b) {
    return createComplex(a.real - b.real, a.imag - b.imag);
  }

  function multiplyComplex(a, b) {
    const real = a.real * b.real - a.imag * b.imag;
    const imag = a.real * b.imag + a.imag * b.real;
    return createComplex(real, imag);
  }

  function conjugateComplex(value) {
    return createComplex(value.real, -value.imag);
  }

  function divideComplex(a, b) {
    const denominator = b.real * b.real + b.imag * b.imag;
    if (denominator <= EPSILON * EPSILON) {
      return createComplex(Number.NaN, Number.NaN);
    }
    const real = (a.real * b.real + a.imag * b.imag) / denominator;
    const imag = (a.imag * b.real - a.real * b.imag) / denominator;
    return createComplex(real, imag);
  }

  function squareComplex(value) {
    const real = value.real * value.real - value.imag * value.imag;
    const imag = 2 * value.real * value.imag;
    return createComplex(real, imag);
  }

  function sqrtComplex(value) {
    const modulus = Math.hypot(value.real, value.imag);
    if (modulus === 0) {
      return createComplex(0, 0);
    }
    const rootModulus = Math.sqrt(modulus);
    const angle = Math.atan2(value.imag, value.real) / 2;
    const real = rootModulus * Math.cos(angle);
    const imag = rootModulus * Math.sin(angle);
    return createComplex(real, imag);
  }

  function expComplex(value) {
    const expReal = Math.exp(value.real);
    const real = expReal * Math.cos(value.imag);
    const imag = expReal * Math.sin(value.imag);
    return createComplex(real, imag);
  }

  function logComplex(value) {
    const modulus = Math.hypot(value.real, value.imag);
    const angle = Math.atan2(value.imag, value.real);
    if (modulus === 0) {
      return createComplex(Number.NEGATIVE_INFINITY, 0);
    }
    return createComplex(Math.log(modulus), angle);
  }

  function powComplex(base, exponent) {
    if (isZeroComplex(base)) {
      if (isZeroComplex(exponent)) {
        return createComplex(1, 0);
      }
      if (!isApproximatelyZero(exponent.imag)) {
        return createComplex(0, 0);
      }
      if (exponent.real > 0) {
        return createComplex(0, 0);
      }
      if (isApproximatelyZero(exponent.real)) {
        return createComplex(1, 0);
      }
      return createComplex(Number.POSITIVE_INFINITY, 0);
    }

    const modulus = Math.hypot(base.real, base.imag);
    const angle = Math.atan2(base.imag, base.real);
    const logModulus = Math.log(modulus);
    const realExponent = exponent.real;
    const imagExponent = exponent.imag;

    const resultModulus = Math.exp(realExponent * logModulus - imagExponent * angle);
    const resultAngle = imagExponent * logModulus + realExponent * angle;
    const real = resultModulus * Math.cos(resultAngle);
    const imag = resultModulus * Math.sin(resultAngle);
    return createComplex(real, imag);
  }

  return {
    EPSILON,
    ZERO_COMPLEX,
    toFiniteNumber,
    createComplex,
    parseComplexString,
    ensureComplex,
    isApproximatelyZero,
    isZeroComplex,
    addComplex,
    subtractComplex,
    multiplyComplex,
    divideComplex,
    conjugateComplex,
    squareComplex,
    sqrtComplex,
    expComplex,
    logComplex,
    powComplex,
  };
}

export function registerComplexPolynomialsComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register complex polynomial components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register complex polynomial components.');
  }

  const {
    ensureComplex,
    squareComplex,
    sqrtComplex,
    expComplex,
    logComplex,
    powComplex,
  } = createComplexToolkit(toNumber);

  register([
    '{0b0f1203-2ea8-4250-a45a-cca7ad2e5b76}',
    'square',
    'sqr',
  ], {
    type: 'complex',
    pinMap: {
      inputs: { x: 'value', X: 'value', Input: 'value', input: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Output: 'result', output: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: squareComplex(value) };
    },
  });

  register([
    '{2d6cb24f-da89-4fab-be0f-e5d439e0217a}',
    'power',
    'pow',
  ], {
    type: 'complex',
    pinMap: {
      inputs: {
        A: 'base',
        a: 'base',
        'First number': 'base',
        'first number': 'base',
        B: 'exponent',
        b: 'exponent',
        'Second number': 'exponent',
        'second number': 'exponent',
      },
      outputs: { R: 'result', r: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const base = ensureComplex(inputs.base);
      const exponent = ensureComplex(inputs.exponent, { real: 1, imag: 0 });
      return { result: powComplex(base, exponent) };
    },
  });

  register([
    '{582f96c6-ed0c-4710-9b5e-a05addba9f42}',
    'exponential',
    'exp',
  ], {
    type: 'complex',
    pinMap: {
      inputs: { x: 'value', X: 'value', Input: 'value', input: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Output: 'result', output: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: expComplex(value) };
    },
  });

  register([
    '{5a22dc1a-907c-4e2f-b8da-0e496c4e25bb}',
    'square root',
    'sqrt',
  ], {
    type: 'complex',
    pinMap: {
      inputs: { x: 'value', X: 'value', Input: 'value', input: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Output: 'result', output: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: sqrtComplex(value) };
    },
  });

  register([
    '{bc4a27fc-cbb9-4802-bd4a-17ab33ad1826}',
    'logarithm',
    'ln',
  ], {
    type: 'complex',
    pinMap: {
      inputs: { x: 'value', X: 'value', Input: 'value', input: 'value', Value: 'value', value: 'value' },
      outputs: { y: 'result', Y: 'result', Output: 'result', output: 'result', Result: 'result', result: 'result' },
    },
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: logComplex(value) };
    },
  });
}

export function registerComplexTrigComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register complex trigonometry components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register complex trigonometry components.');
  }

  const {
    ensureComplex,
    createComplex,
    addComplex,
    subtractComplex,
    multiplyComplex,
    divideComplex,
    sqrtComplex,
    logComplex,
  } = createComplexToolkit(toNumber);

  const ONE = createComplex(1, 0);
  const IMAG_UNIT = createComplex(0, 1);
  const NEG_IMAG_UNIT = createComplex(0, -1);
  const HALF_IMAG_UNIT = createComplex(0, 0.5);

  function sinComplex(value) {
    const real = Math.sin(value.real) * Math.cosh(value.imag);
    const imag = Math.cos(value.real) * Math.sinh(value.imag);
    return createComplex(real, imag);
  }

  function cosComplex(value) {
    const real = Math.cos(value.real) * Math.cosh(value.imag);
    const imag = -Math.sin(value.real) * Math.sinh(value.imag);
    return createComplex(real, imag);
  }

  function tanComplex(value) {
    return divideComplex(sinComplex(value), cosComplex(value));
  }

  function secComplex(value) {
    return divideComplex(ONE, cosComplex(value));
  }

  function cosecComplex(value) {
    return divideComplex(ONE, sinComplex(value));
  }

  function cotComplex(value) {
    return divideComplex(cosComplex(value), sinComplex(value));
  }

  function asinComplex(value) {
    const zSquared = multiplyComplex(value, value);
    const underRoot = subtractComplex(ONE, zSquared);
    const sqrtTerm = sqrtComplex(underRoot);
    const iTimesZ = multiplyComplex(IMAG_UNIT, value);
    const inside = addComplex(iTimesZ, sqrtTerm);
    const logValue = logComplex(inside);
    return multiplyComplex(NEG_IMAG_UNIT, logValue);
  }

  function acosComplex(value) {
    const zSquared = multiplyComplex(value, value);
    const zSquaredMinusOne = subtractComplex(zSquared, ONE);
    const sqrtTerm = sqrtComplex(zSquaredMinusOne);
    const inside = addComplex(value, sqrtTerm);
    const logValue = logComplex(inside);
    return multiplyComplex(NEG_IMAG_UNIT, logValue);
  }

  function atanComplex(value) {
    const iTimesZ = multiplyComplex(IMAG_UNIT, value);
    const oneMinusIZ = subtractComplex(ONE, iTimesZ);
    const onePlusIZ = addComplex(ONE, iTimesZ);
    const logMinus = logComplex(oneMinusIZ);
    const logPlus = logComplex(onePlusIZ);
    const difference = subtractComplex(logMinus, logPlus);
    return multiplyComplex(HALF_IMAG_UNIT, difference);
  }

  const pinMap = {
    inputs: { x: 'value', X: 'value', Input: 'value', input: 'value', Value: 'value', value: 'value' },
    outputs: { y: 'result', Y: 'result', Output: 'result', output: 'result', Result: 'result', result: 'result' },
  };

  register([
    '{c53932eb-7c8c-4825-ae98-e36bba97232d}',
    'sine',
    'sin',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: sinComplex(value) };
    },
  });

  register([
    '{7874f26c-6f76-4da8-b527-2d567184b2bd}',
    'cosine',
    'cos',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: cosComplex(value) };
    },
  });

  register([
    '{0bc93049-e1a7-44b5-8068-c7ddc85a9f46}',
    'tangent',
    'tan',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: tanComplex(value) };
    },
  });

  register([
    '{d879e74c-6fe3-4cbf-b3fa-60a7c48b73e7}',
    'secant',
    'sec',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: secComplex(value) };
    },
  });

  register([
    '{99197a17-d5c7-419b-acde-eca2737f3c58}',
    'cosecant',
    'cosec',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: cosecComplex(value) };
    },
  });

  register([
    '{39461433-ac44-4298-94a9-988f983e347c}',
    'cotangent',
    'cotan',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: cotComplex(value) };
    },
  });

  register([
    '{4e8aad42-9111-470c-9acd-7ae365d8bba4}',
    'arctangent',
    'atan',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: atanComplex(value) };
    },
  });

  register([
    '{8640c519-9bf6-4e9a-a108-75f9d89b2c58}',
    'arccosine',
    'acos',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: acosComplex(value) };
    },
  });

  register([
    '{f18091e9-3264-4dd4-9ba6-32c77fca0ac0}',
    'arcsine',
    'asin',
  ], {
    type: 'complex',
    pinMap,
    eval: ({ inputs }) => {
      const value = ensureComplex(inputs.value);
      return { result: asinComplex(value) };
    },
  });
}

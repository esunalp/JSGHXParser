export function registerComplexPolynomialsComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register complex polynomial components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register complex polynomial components.');
  }

  const EPSILON = 1e-12;
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


const DEFAULT_MAX_UNWRAP_DEPTH = 32;
const EPSILON = 1e-9;

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function ensureRegisterFunction(register) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register scalar components.');
  }
}

function ensureToNumberFunction(toNumber) {
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register scalar components.');
  }
}

function createGuidKeys(...guids) {
  const keys = new Set();
  for (const guid of guids) {
    if (!guid && guid !== 0) {
      continue;
    }
    const text = String(guid).trim();
    if (!text) {
      continue;
    }
    const bare = text.replace(/^\{+/, '').replace(/\}+$/, '');
    if (!bare) {
      continue;
    }
    keys.add(bare);
    keys.add(`{${bare}}`);
  }
  return Array.from(keys);
}

function resolveNumeric(input, toNumber, fallback = 0) {
  const stack = [input];
  const seen = new Set();
  let depth = 0;

  const pushCandidate = (candidate) => {
    if (candidate === undefined || candidate === null) {
      return;
    }
    stack.push(candidate);
  };

  while (stack.length && depth < DEFAULT_MAX_UNWRAP_DEPTH) {
    const current = stack.pop();
    depth += 1;

    if (current === undefined || current === null) {
      continue;
    }

    const type = typeof current;
    if (type === 'number' || type === 'string' || type === 'boolean' || type === 'bigint') {
      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        return { value: numeric, valid: true };
      }
      continue;
    }

    if (Array.isArray(current)) {
      for (let index = current.length - 1; index >= 0; index -= 1) {
        pushCandidate(current[index]);
      }
      continue;
    }

    if (type === 'object') {
      if (seen.has(current)) {
        continue;
      }
      seen.add(current);

      if (typeof current.valueOf === 'function') {
        const numeric = toNumber(current.valueOf(), Number.NaN);
        if (Number.isFinite(numeric)) {
          return { value: numeric, valid: true };
        }
      }

      if (typeof current[Symbol.iterator] === 'function') {
        for (const entry of current) {
          pushCandidate(entry);
        }
      }

      if (current.isVector3) {
        const length = current.length?.();
        if (typeof length === 'number' && Number.isFinite(length)) {
          return { value: length, valid: true };
        }
      }

      const candidateKeys = [
        'value', 'Value', 'values', 'Values', 'items', 'Items', 'data', 'Data',
        'number', 'Number', 'numeric', 'Numeric', 'result', 'Result', 'input', 'Input',
        'first', 'First', 'second', 'Second', 'A', 'a', 'B', 'b',
      ];

      let pushed = false;
      for (const key of candidateKeys) {
        if (Object.prototype.hasOwnProperty.call(current, key)) {
          pushed = true;
          pushCandidate(current[key]);
        }
      }

      if (pushed) {
        continue;
      }

      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        return { value: numeric, valid: true };
      }

      continue;
    }
  }

  return { value: fallback, valid: false };
}

function collectNumericValues(input, toNumber) {
  const numbers = [];
  const stack = [input];
  const seen = new Set();
  let depth = 0;

  const pushCandidate = (candidate) => {
    if (candidate === undefined || candidate === null) {
      return;
    }
    stack.push(candidate);
  };

  while (stack.length && depth < DEFAULT_MAX_UNWRAP_DEPTH) {
    const current = stack.pop();
    depth += 1;

    if (current === undefined || current === null) {
      continue;
    }

    const type = typeof current;
    if (type === 'number' || type === 'string' || type === 'boolean' || type === 'bigint') {
      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        numbers.push(numeric);
      }
      continue;
    }

    if (Array.isArray(current)) {
      for (let index = current.length - 1; index >= 0; index -= 1) {
        pushCandidate(current[index]);
      }
      continue;
    }

    if (type === 'object') {
      if (seen.has(current)) {
        continue;
      }
      seen.add(current);

      if (typeof current.valueOf === 'function') {
        const raw = current.valueOf();
        if (raw !== current) {
          pushCandidate(raw);
        } else {
          const numeric = toNumber(raw, Number.NaN);
          if (Number.isFinite(numeric)) {
            numbers.push(numeric);
            continue;
          }
        }
      }

      if (typeof current[Symbol.iterator] === 'function') {
        const items = [];
        for (const entry of current) {
          items.push(entry);
        }
        for (let index = items.length - 1; index >= 0; index -= 1) {
          pushCandidate(items[index]);
        }
        continue;
      }

      if (current.isVector3 && typeof current.length === 'function') {
        const length = current.length();
        if (Number.isFinite(length)) {
          numbers.push(length);
          continue;
        }
      }

      const candidateKeys = [
        'value', 'Value', 'values', 'Values', 'items', 'Items', 'data', 'Data',
        'number', 'Number', 'numeric', 'Numeric', 'result', 'Result', 'input', 'Input',
        'first', 'First', 'second', 'Second', 'A', 'a', 'B', 'b',
      ];

      let pushed = false;
      for (const key of candidateKeys) {
        if (Object.prototype.hasOwnProperty.call(current, key)) {
          pushed = true;
          pushCandidate(current[key]);
        }
      }

      if (pushed) {
        continue;
      }

      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        numbers.push(numeric);
      }
    }
  }

  return numbers;
}

function createBinaryEvaluator({ toNumber, firstFallback = 0, secondFallback = 0, compute }) {
  return ({ inputs }) => {
    const first = resolveNumeric(inputs?.first, toNumber, firstFallback);
    const second = resolveNumeric(inputs?.second, toNumber, secondFallback);

    if (!first.valid && !second.valid) {
      return { result: null };
    }

    const outcome = compute({
      firstValue: first.value,
      secondValue: second.value,
      firstValid: first.valid,
      secondValid: second.valid,
    });

    if (outcome === null || outcome === undefined) {
      return { result: null };
    }

    if (typeof outcome === 'number') {
      return Number.isFinite(outcome) ? { result: outcome } : { result: null };
    }

    return { result: outcome };
  };
}

function createUnaryEvaluator({ toNumber, fallback = 0, compute }) {
  return ({ inputs }) => {
    const value = resolveNumeric(inputs?.value, toNumber, fallback);

    if (!value.valid) {
      return { result: null };
    }

    const outcome = compute({ value: value.value, valid: value.valid });

    if (outcome === null || outcome === undefined) {
      return { result: null };
    }

    if (typeof outcome === 'number') {
      return Number.isFinite(outcome) ? { result: outcome } : { result: null };
    }

    return { result: outcome };
  };
}

const BINARY_PIN_MAP = {
  inputs: {
    A: 'first',
    a: 'first',
    'First number': 'first',
    'Base number': 'first',
    'first number': 'first',
    'base number': 'first',
    B: 'second',
    b: 'second',
    'Second number': 'second',
    'Modulus': 'second',
    'second number': 'second',
    'modulus': 'second',
  },
  outputs: {
    R: 'result',
    r: 'result',
    Result: 'result',
    result: 'result',
    Output: 'result',
    output: 'result',
  },
};

const UNARY_PIN_MAP = {
  inputs: {
    x: 'value',
    X: 'value',
    Input: 'value',
    input: 'value',
    Value: 'value',
    value: 'value',
  },
  outputs: {
    y: 'result',
    Y: 'result',
    Output: 'result',
    output: 'result',
    Result: 'result',
    result: 'result',
  },
};

export function registerScalarOperatorsComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const registerOperation = (keys, evaluator) => {
    const uniqueKeys = Array.from(new Set(keys));
    register(uniqueKeys, {
      type: 'math',
      pinMap: BINARY_PIN_MAP,
      eval: evaluator,
    });
  };

  registerOperation([
    ...createGuidKeys('cae37d1c-8146-4e0b-9cf1-14cb3e337b94'),
    'scalar:addition',
    'scalar addition',
    'scalar-operators:addition',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 0,
    secondFallback: 0,
    compute: ({ firstValue, secondValue }) => firstValue + secondValue,
  }));

  registerOperation([
    ...createGuidKeys('f4a20a34-97e6-4ff5-9b26-7f7ed7a1e333'),
    'scalar:subtraction',
    'scalar subtraction',
    'scalar-operators:subtraction',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 0,
    secondFallback: 0,
    compute: ({ firstValue, secondValue }) => firstValue - secondValue,
  }));

  registerOperation([
    ...createGuidKeys('3e6383e9-af39-427b-801a-19ca916160fa'),
    'scalar:multiplication',
    'scalar multiplication',
    'scalar-operators:multiplication',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 1,
    secondFallback: 1,
    compute: ({ firstValue, secondValue, firstValid, secondValid }) => {
      if (!firstValid && !secondValid) {
        return null;
      }
      return firstValue * secondValue;
    },
  }));

  registerOperation([
    ...createGuidKeys('ec875825-61e4-4c1c-a343-0e0cee0b321b'),
    'scalar:division',
    'scalar division',
    'scalar-operators:division',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 0,
    secondFallback: 1,
    compute: ({ firstValue, secondValue, secondValid }) => {
      if (!secondValid || secondValue === 0) {
        return null;
      }
      return firstValue / secondValue;
    },
  }));

  registerOperation([
    ...createGuidKeys(
      '481e1f0d-a945-4662-809d-f49d1a8f40bd',
      '9ebccbb4-f3e3-4ee1-af31-2f301f2516f0'
    ),
    'scalar:modulus',
    'scalar modulus',
    'scalar-operators:modulus',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 0,
    secondFallback: 1,
    compute: ({ firstValue, secondValue, secondValid }) => {
      if (!secondValid || secondValue === 0) {
        return null;
      }
      const remainder = ((firstValue % secondValue) + secondValue) % secondValue;
      return remainder;
    },
  }));
}

export function registerScalarPolynomialsComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const registerUnaryOperation = (keys, evaluator) => {
    const uniqueKeys = Array.from(new Set(keys));
    register(uniqueKeys, {
      type: 'math',
      pinMap: UNARY_PIN_MAP,
      eval: evaluator,
    });
  };

  const registerBinaryOperation = (keys, evaluator) => {
    const uniqueKeys = Array.from(new Set(keys));
    register(uniqueKeys, {
      type: 'math',
      pinMap: BINARY_PIN_MAP,
      eval: evaluator,
    });
  };

  registerUnaryOperation([
    ...createGuidKeys('5f212b16-82a0-4699-be4c-11529a9810ae'),
    'scalar:power-of-e',
    'scalar-polynomials:power-of-e',
    'scalar:exp',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.exp(value),
  }));

  registerUnaryOperation([
    ...createGuidKeys('8b62751f-6fb4-4d03-a238-11ad6db7483e'),
    'scalar:natural-logarithm',
    'scalar-polynomials:natural-logarithm',
    'scalar:ln',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      if (value <= 0) {
        return null;
      }
      return Math.log(value);
    },
  }));

  registerBinaryOperation([
    ...createGuidKeys('96c8c5f2-5f8e-4bb3-b19f-eb61d9cefa46'),
    'scalar:power',
    'scalar-polynomials:power',
    'scalar:pow',
  ], createBinaryEvaluator({
    toNumber,
    firstFallback: 1,
    secondFallback: 1,
    compute: ({ firstValue, secondValue, firstValid, secondValid }) => {
      if (!firstValid || !secondValid) {
        return null;
      }
      return Math.pow(firstValue, secondValue);
    },
  }));

  registerUnaryOperation([
    ...createGuidKeys('a8bc9c24-1bce-4b92-b7ba-abced2457c22'),
    'scalar:power-of-two',
    'scalar-polynomials:power-of-two',
    'scalar:pow2',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.pow(2, value),
  }));

  registerUnaryOperation([
    ...createGuidKeys('d0787f37-d976-48c9-a4b0-29d6c4059cf3'),
    'scalar:logarithm',
    'scalar-polynomials:logarithm',
    'scalar:log10',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      if (value <= 0) {
        return null;
      }
      if (typeof Math.log10 === 'function') {
        return Math.log10(value);
      }
      return Math.log(value) / Math.LN10;
    },
  }));

  registerUnaryOperation([
    ...createGuidKeys('ed766861-662d-4462-90f6-29f87f8529cf'),
    'scalar:power-of-ten',
    'scalar-polynomials:power-of-ten',
    'scalar:pow10',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.pow(10, value),
  }));
}

export function registerScalarUtilComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const registerOperation = (keys, pinMap, evaluator) => {
    const uniqueKeys = Array.from(new Set(keys));
    register(uniqueKeys, {
      type: 'math',
      pinMap,
      eval: evaluator,
    });
  };

  const registerBinaryOperation = (keys, evaluatorConfig) => {
    registerOperation(keys, BINARY_PIN_MAP, createBinaryEvaluator(evaluatorConfig));
  };

  registerOperation(
    [
      ...createGuidKeys('0bb7682f-333c-4bb7-b6fe-91ed2c886100'),
      'scalar:mean',
      'scalar-util:mean',
    ],
    {
      inputs: { I: 'values', Input: 'values' },
      outputs: {
        AM: 'arithmeticMean',
        'Arithmetic mean': 'arithmeticMean',
        GM: 'geometricMean',
        'Geometric mean': 'geometricMean',
        HM: 'harmonicMean',
        'Harmonic mean': 'harmonicMean',
      },
    },
    ({ inputs }) => {
      const numbers = collectNumericValues(inputs.values, toNumber);
      if (!numbers.length) {
        return {
          arithmeticMean: null,
          geometricMean: null,
          harmonicMean: null,
        };
      }

      let sum = 0;
      let logSum = 0;
      let hasZero = false;
      let reciprocalSum = 0;
      let geometricValid = true;
      let harmonicValid = true;

      for (const value of numbers) {
        sum += value;

        if (geometricValid) {
          if (value < 0) {
            geometricValid = false;
          } else if (value === 0) {
            hasZero = true;
          } else {
            logSum += Math.log(value);
          }
        }

        if (harmonicValid) {
          if (value === 0) {
            harmonicValid = false;
          } else {
            reciprocalSum += 1 / value;
          }
        }
      }

      const count = numbers.length;
      const arithmeticMean = sum / count;

      let geometricMean = null;
      if (geometricValid) {
        geometricMean = hasZero ? 0 : Math.exp(logSum / count);
      }

      let harmonicMean = null;
      if (harmonicValid && Math.abs(reciprocalSum) > EPSILON) {
        harmonicMean = count / reciprocalSum;
      } else if (harmonicValid) {
        harmonicValid = false;
      }

      return {
        arithmeticMean,
        geometricMean: geometricValid ? geometricMean : null,
        harmonicMean: harmonicValid ? harmonicMean : null,
      };
    }
  );

  registerBinaryOperation(
    [
      ...createGuidKeys('532b722d-9368-42ee-b99d-64a4732ee99a'),
      'scalar:minimum',
      'scalar-util:minimum',
    ],
    {
      toNumber,
      firstFallback: Number.POSITIVE_INFINITY,
      secondFallback: Number.POSITIVE_INFINITY,
      compute: ({ firstValue, secondValue, firstValid, secondValid }) => {
        if (!firstValid && !secondValid) {
          return null;
        }
        if (!firstValid) {
          return secondValue;
        }
        if (!secondValid) {
          return firstValue;
        }
        return Math.min(firstValue, secondValue);
      },
    }
  );

  registerBinaryOperation(
    [
      ...createGuidKeys('e9b807a3-dd48-4c2c-bada-e4f8e0edbbdb'),
      'scalar:maximum',
      'scalar-util:maximum',
    ],
    {
      toNumber,
      firstFallback: Number.NEGATIVE_INFINITY,
      secondFallback: Number.NEGATIVE_INFINITY,
      compute: ({ firstValue, secondValue, firstValid, secondValid }) => {
        if (!firstValid && !secondValid) {
          return null;
        }
        if (!firstValid) {
          return secondValue;
        }
        if (!secondValid) {
          return firstValue;
        }
        return Math.max(firstValue, secondValue);
      },
    }
  );

  const massAdditionPinMap = {
    inputs: { I: 'values', Input: 'values' },
    outputs: {
      R: 'result',
      Result: 'result',
      Pr: 'partialResults',
      'Partial Result': 'partialResults',
    },
  };

  const computeMassAddition = (values) => {
    const numbers = collectNumericValues(values, toNumber);
    let total = 0;
    const partialResults = [];
    for (const value of numbers) {
      total += value;
      partialResults.push(total);
    }
    return { total, partialResults };
  };

  registerOperation(
    [
      ...createGuidKeys('74d95062-0bec-4a4e-9026-5141fca954a6'),
      'scalar:mass-addition-with-partials',
      'scalar-util:mass-addition-with-partials',
    ],
    massAdditionPinMap,
    ({ inputs }) => {
      const { total, partialResults } = computeMassAddition(inputs.values);
      return {
        result: total,
        partialResults,
      };
    }
  );

  registerOperation(
    [
      ...createGuidKeys('bb64b2fb-f87a-432f-86f8-393f4ee21310'),
      'scalar:mass-addition',
      'scalar-util:mass-addition',
    ],
    {
      inputs: { I: 'values', Input: 'values' },
      outputs: { R: 'result', Result: 'result' },
    },
    ({ inputs }) => {
      const { total } = computeMassAddition(inputs.values);
      return { result: total };
    }
  );

  registerOperation(
    [
      ...createGuidKeys('a8de2000-073d-412d-a0b2-3a4894ba71f8'),
      'scalar:truncate',
      'scalar-util:truncate',
    ],
    {
      inputs: { I: 'values', Input: 'values', t: 'factor', 'Truncation factor': 'factor' },
      outputs: {
        T: 'truncated',
        Result: 'truncated',
        'Truncated set': 'truncated',
      },
    },
    ({ inputs }) => {
      const numbers = collectNumericValues(inputs.values, toNumber);
      const factorValue = toNumber(inputs.factor, 0);
      if (!numbers.length) {
        return { truncated: [] };
      }

      const factor = Number.isFinite(factorValue) ? clamp(factorValue, 0, 1) : 0;
      if (numbers.length === 0 || factor <= 0) {
        return { truncated: [...numbers] };
      }

      const sorted = [...numbers].sort((a, b) => a - b);
      const removeTotal = Math.round(sorted.length * factor);
      const removeLower = Math.floor(removeTotal / 2);
      const removeUpper = removeTotal - removeLower;
      const start = Math.min(removeLower, sorted.length);
      const end = Math.max(start, sorted.length - removeUpper);
      const kept = sorted.slice(start, end);

      const remainingCounts = new Map();
      for (const value of kept) {
        remainingCounts.set(value, (remainingCounts.get(value) ?? 0) + 1);
      }

      const truncated = [];
      for (const value of numbers) {
        const count = remainingCounts.get(value) ?? 0;
        if (count > 0) {
          truncated.push(value);
          remainingCounts.set(value, count - 1);
        }
      }

      return { truncated };
    }
  );
}

export function registerScalarTrigComponents({ register, toNumber }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);

  const registerOperation = (keys, evaluator) => {
    const uniqueKeys = Array.from(new Set(keys));
    register(uniqueKeys, {
      type: 'math',
      pinMap: UNARY_PIN_MAP,
      eval: evaluator,
    });
  };

  registerOperation([
    ...createGuidKeys('ecee923b-1b93-4cf2-acd6-680835503437'),
    'scalar:sine',
    'scalar-trig:sine',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.sin(value),
  }));

  registerOperation([
    ...createGuidKeys('12278a4b-c131-4735-a3ee-bcb783083856'),
    'scalar:cosine',
    'scalar-trig:cosine',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.cos(value),
  }));

  registerOperation([
    ...createGuidKeys('002b2feb-5d1b-41ea-913f-9f203c615792'),
    'scalar:tangent',
    'scalar-trig:tangent',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      const cosValue = Math.cos(value);
      if (Math.abs(cosValue) < EPSILON) {
        return null;
      }
      return Math.tan(value);
    },
  }));

  registerOperation([
    ...createGuidKeys('22bba82d-32e8-448c-a59c-f054c8843ee3'),
    'scalar:arcsine',
    'scalar-trig:arcsine',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      if (value < -1 - EPSILON || value > 1 + EPSILON) {
        return null;
      }
      return Math.asin(clamp(value, -1, 1));
    },
  }));

  registerOperation([
    ...createGuidKeys('cfc280bb-332a-4828-bb4e-aca6d88859aa'),
    'scalar:arccosine',
    'scalar-trig:arccosine',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      if (value < -1 - EPSILON || value > 1 + EPSILON) {
        return null;
      }
      return Math.acos(clamp(value, -1, 1));
    },
  }));

  registerOperation([
    ...createGuidKeys('7b312903-4782-438f-aa37-ba43f5083460'),
    'scalar:arctangent',
    'scalar-trig:arctangent',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => Math.atan(value),
  }));

  registerOperation([
    ...createGuidKeys('da4be42b-ba75-4249-a685-69ce78b6ee44'),
    'scalar:sinc',
    'scalar-trig:sinc',
  ], createUnaryEvaluator({
    toNumber,
    compute: ({ value }) => {
      if (Math.abs(value) < EPSILON) {
        return 1;
      }
      return Math.sin(value) / value;
    },
  }));
}

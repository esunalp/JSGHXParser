const DEFAULT_MAX_UNWRAP_DEPTH = 32;

function ensureRegisterFunction(register) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register scalar operator components.');
  }
}

function ensureToNumberFunction(toNumber) {
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register scalar operator components.');
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

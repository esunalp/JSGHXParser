function createIntersectComponentUtils({ toNumber }) {
  function ensureNumber(value, fallback = 0) {
    const numeric = toNumber(value, Number.NaN);
    return Number.isFinite(numeric) ? numeric : fallback;
  }

  function isDataTree(value) {
    return Boolean(value && typeof value === 'object' && value.type === 'tree' && Array.isArray(value.branches));
  }

  function ensureSimpleArray(values) {
    if (Array.isArray(values)) {
      return values;
    }
    if (values === undefined || values === null) {
      return [];
    }
    return [values];
  }

  function ensureArray(value) {
    if (value === undefined || value === null) {
      return [];
    }
    if (Array.isArray(value)) {
      const result = [];
      value.forEach((entry) => {
        result.push(...ensureArray(entry));
      });
      return result;
    }
    if (isDataTree(value)) {
      const values = [];
      for (const branch of value.branches) {
        if (!branch) continue;
        values.push(...ensureSimpleArray(branch.values));
      }
      return values;
    }
    return [value];
  }

  function resolveItem(value) {
    const list = ensureArray(value);
    for (const entry of list) {
      if (entry !== undefined && entry !== null) {
        return entry;
      }
    }
    return null;
  }

  function isPlainObject(value) {
    if (!value || typeof value !== 'object') {
      return false;
    }
    const proto = Object.getPrototypeOf(value);
    return proto === Object.prototype || proto === null;
  }

  function cloneShape(value) {
    if (value === undefined || value === null) {
      return value;
    }
    if (typeof value.clone === 'function') {
      try {
        return value.clone();
      } catch (error) {
        // Fall through to other strategies if clone fails.
      }
    }
    if (Array.isArray(value)) {
      return value.map((entry) => cloneShape(entry));
    }
    if (isDataTree(value)) {
      return {
        type: 'tree',
        branches: value.branches.map((branch) => ({
          path: Array.isArray(branch?.path) ? [...branch.path] : [],
          values: ensureSimpleArray(branch?.values).map((entry) => cloneShape(entry)),
        })),
      };
    }
    if (isPlainObject(value)) {
      const clone = { ...value };
      if (isPlainObject(clone.metadata)) {
        clone.metadata = { ...clone.metadata };
      }
      return clone;
    }
    return value;
  }

  function annotateShape(shape, metadata = {}, { kind = null } = {}) {
    if (shape === undefined || shape === null) {
      return shape;
    }
    const clone = cloneShape(shape);
    if (clone && typeof clone === 'object') {
      const baseMetadata = isPlainObject(clone.metadata) ? clone.metadata : {};
      clone.metadata = {
        ...baseMetadata,
        ...(kind ? { kind } : {}),
        ...metadata,
      };
    }
    return clone;
  }

  function prepareList(values, { kind, operation, role }) {
    const list = ensureArray(values);
    return list.map((value, index) => annotateShape(value, { operation, role, index }, { kind }));
  }

  function createDataTree(branches = []) {
    return { type: 'tree', branches };
  }

  function createTreeFromItems(items, mapper) {
    return createDataTree(items.map((item, index) => {
      const mapped = ensureSimpleArray(mapper(item, index));
      return { path: [index], values: mapped };
    }));
  }

  return {
    ensureNumber,
    isDataTree,
    ensureSimpleArray,
    ensureArray,
    resolveItem,
    isPlainObject,
    cloneShape,
    annotateShape,
    prepareList,
    createDataTree,
    createTreeFromItems,
  };
}

export function registerIntersectShapeComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register intersect shape components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register intersect shape components.');
  }

  const {
    ensureNumber,
    ensureArray,
    resolveItem,
    annotateShape,
    prepareList,
    createDataTree,
    createTreeFromItems,
  } = createIntersectComponentUtils({ toNumber });

  function createBrepBooleanSummary(operation, primaryValues, secondaryValues = [], extras = {}, planeValue = null) {
    const primary = prepareList(primaryValues, { kind: 'brep', operation, role: 'primary' });
    const secondary = prepareList(secondaryValues, { kind: 'brep', operation, role: 'secondary' });
    if (!primary.length && !secondary.length) {
      return null;
    }
    const summary = {
      type: 'brep-boolean',
      operation,
      primary,
      metadata: {
        operation,
        primaryCount: primary.length,
        secondaryCount: secondary.length,
        ...extras,
      },
    };
    if (secondary.length) {
      summary.secondary = secondary;
    }
    if (planeValue) {
      summary.plane = annotateShape(planeValue, { operation, role: 'plane' }, { kind: 'plane' });
    }
    summary.breps = [...primary, ...secondary];
    return summary;
  }

  function createMeshBooleanSummary(operation, primaryValues, secondaryValues = [], extras = {}) {
    const primary = prepareList(primaryValues, { kind: 'mesh', operation, role: 'primary' });
    const secondary = prepareList(secondaryValues, { kind: 'mesh', operation, role: 'secondary' });
    if (!primary.length && !secondary.length) {
      return null;
    }
    const summary = {
      type: 'mesh-boolean',
      operation,
      primary,
      metadata: {
        operation,
        primaryCount: primary.length,
        secondaryCount: secondary.length,
        ...extras,
      },
    };
    if (secondary.length) {
      summary.secondary = secondary;
    }
    summary.meshes = [...primary, ...secondary];
    return summary;
  }

  function createRegionBooleanSummary(operation, primaryValues, secondaryValues = [], extras = {}, planeValue = null) {
    const primary = prepareList(primaryValues, { kind: 'region', operation, role: 'primary' });
    const secondary = prepareList(secondaryValues, { kind: 'region', operation, role: 'secondary' });
    if (!primary.length && !secondary.length) {
      return null;
    }
    const summary = {
      type: 'region-boolean',
      operation,
      primary,
      metadata: {
        operation,
        primaryCount: primary.length,
        secondaryCount: secondary.length,
        ...extras,
      },
    };
    if (secondary.length) {
      summary.secondary = secondary;
    }
    if (planeValue) {
      summary.plane = annotateShape(planeValue, { operation, role: 'plane' }, { kind: 'plane' });
    }
    summary.curves = [...primary, ...secondary];
    return summary;
  }

  function createTargetOperationSummary({
    type,
    operation,
    targetValue,
    secondaryValues,
    kind,
    collectionName,
    secondaryRole,
    countLabel,
    extras = {},
  }) {
    const secondaryArray = ensureArray(secondaryValues);
    const target = targetValue
      ? annotateShape(targetValue, { operation, role: 'target', [countLabel]: secondaryArray.length }, { kind })
      : null;
    const secondary = secondaryArray.map((value, index) => annotateShape(value, { operation, role: secondaryRole, index }, { kind }));
    if (!target) {
      return null;
    }
    const summary = {
      type,
      operation,
      target,
      [collectionName]: secondary,
      metadata: {
        operation,
        [countLabel]: secondary.length,
        ...extras,
      },
    };
    if (kind === 'brep') {
      summary.breps = [target, ...secondary];
    } else if (kind === 'mesh') {
      summary.meshes = [target, ...secondary];
    } else {
      summary.shapes = [target, ...secondary];
    }
    return summary;
  }

  function createRegionSlitOutputs(regions, width, gap) {
    const regionList = ensureArray(regions);
    const regionsTree = createTreeFromItems(regionList, (region, index) => [
      annotateShape(region, { operation: 'slit', width, gap, index }, { kind: 'region' }),
    ]);
    const topologyTree = createTreeFromItems(regionList, (_, index) => [{
      type: 'region-slit-topology',
      index,
      width,
      gap,
      metadata: { index, width, gap },
    }]);
    return { regions: regionsTree, topology: topologyTree };
  }

  function createBoxSlitOutputs(boxes, gap) {
    const boxList = ensureArray(boxes);
    const brepsTree = createTreeFromItems(boxList, (box, index) => [
      annotateShape(box, { operation: 'slit', gap, index }, { kind: 'brep' }),
    ]);
    const topologyTree = createTreeFromItems(boxList, (_, index) => [{
      type: 'box-slit-topology',
      index,
      gap,
      metadata: { index, gap },
    }]);
    return { breps: brepsTree, topology: topologyTree };
  }

  function createBoundaryVolumeSummary(boundaries) {
    const boundaryList = prepareList(boundaries, { kind: 'brep', operation: 'boundary-volume', role: 'boundary' });
    if (!boundaryList.length) {
      return null;
    }
    return {
      type: 'brep-volume',
      operation: 'boundary-volume',
      boundaries: boundaryList,
      breps: boundaryList,
      metadata: {
        operation: 'boundary-volume',
        boundaryCount: boundaryList.length,
      },
    };
  }

  register('{03f22640-ff80-484e-bb53-a4025c5faa07}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'brep',
        Brep: 'brep',
        brep: 'brep',
        C: 'cutters',
        Cutters: 'cutters',
        cutters: 'cutters',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const brep = resolveItem(inputs.brep);
      if (!brep) {
        return { result: [] };
      }
      const cutters = ensureArray(inputs.cutters);
      const summary = createTargetOperationSummary({
        type: 'brep-split',
        operation: 'split',
        targetValue: brep,
        secondaryValues: cutters,
        kind: 'brep',
        collectionName: 'cutters',
        secondaryRole: 'cutter',
        countLabel: 'cutterCount',
        extras: {
          mode: cutters.length > 1 ? 'multiple' : 'single',
        },
      });
      return { result: summary ? [summary] : [] };
    },
  });

  register('{0feeeaca-8f1f-4d7c-a24a-8e7dd68604a2}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        R: 'regions',
        Regions: 'regions',
        regions: 'regions',
        W: 'width',
        Width: 'width',
        width: 'width',
        G: 'gap',
        Gap: 'gap',
        gap: 'gap',
      },
      outputs: {
        R: 'regions',
        Regions: 'regions',
        regions: 'regions',
        T: 'topology',
        Topology: 'topology',
        topology: 'topology',
      },
    },
    eval: ({ inputs }) => {
      const regions = ensureArray(inputs.regions);
      if (!regions.length) {
        return {
          regions: createDataTree(),
          topology: createDataTree(),
        };
      }
      const width = ensureNumber(inputs.width, 0);
      const gap = ensureNumber(inputs.gap, 0);
      const { regions: regionsTree, topology } = createRegionSlitOutputs(regions, width, gap);
      return { regions: regionsTree, topology };
    },
  });

  register('{10434a15-da85-4281-bb64-a2b3a995b9c6}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'breps',
        Brep: 'breps',
        Breps: 'breps',
        brep: 'breps',
        breps: 'breps',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createBrepBooleanSummary('union', inputs.breps);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{1222394f-0d33-4f31-9101-7281bde89fe5}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
        P: 'plane',
        Plane: 'plane',
        plane: 'plane',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createRegionBooleanSummary('union', inputs.curves, [], {}, inputs.plane);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{2d3b6ef3-5c26-4e2f-bcb3-8ffb9fb0f7c3}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'boxes',
        Box: 'boxes',
        Boxes: 'boxes',
        box: 'boxes',
        boxes: 'boxes',
        G: 'gap',
        Gap: 'gap',
        gap: 'gap',
      },
      outputs: {
        B: 'breps',
        Breps: 'breps',
        breps: 'breps',
        T: 'topology',
        Topology: 'topology',
        topology: 'topology',
      },
    },
    eval: ({ inputs }) => {
      const boxes = ensureArray(inputs.boxes);
      if (!boxes.length) {
        return {
          breps: createDataTree(),
          topology: createDataTree(),
        };
      }
      const gap = ensureNumber(inputs.gap, 0);
      const { breps, topology } = createBoxSlitOutputs(boxes, gap);
      return { breps, topology };
    },
  });

  register('{477c2e7b-c5e5-421e-b8b2-ba60cdf5398b}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'curvesA',
        'Curves A': 'curvesA',
        B: 'curvesB',
        'Curves B': 'curvesB',
        P: 'plane',
        Plane: 'plane',
        plane: 'plane',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createRegionBooleanSummary('intersection', inputs.curvesA, inputs.curvesB, {}, inputs.plane);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{4f3147f4-9fcd-4a7e-be0e-b1841caa5f97}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'meshesA',
        'Meshes A': 'meshesA',
        B: 'meshesB',
        'Meshes B': 'meshesB',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createMeshBooleanSummary('difference', inputs.meshesA, inputs.meshesB);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{5723c845-cafc-442d-a667-8c76532845e6}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'brepsA',
        'Breps A': 'brepsA',
        B: 'brepsB',
        'Breps B': 'brepsB',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createBrepBooleanSummary('intersection', inputs.brepsA, inputs.brepsB);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{88060a82-0bf7-46bb-9af8-bdc860cf7e1d}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        M: 'meshes',
        Mesh: 'meshes',
        Meshes: 'meshes',
        mesh: 'meshes',
        meshes: 'meshes',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createMeshBooleanSummary('union', inputs.meshes);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{95aef4f6-66fc-477e-b8f8-32395a837831}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'meshesA',
        'Meshes A': 'meshesA',
        B: 'meshesB',
        'Meshes B': 'meshesB',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createMeshBooleanSummary('intersection', inputs.meshesA, inputs.meshesB);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{afbf2fe0-4965-48d2-8470-9e991540093b}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        M: 'mesh',
        Mesh: 'mesh',
        mesh: 'mesh',
        S: 'splitters',
        Splitters: 'splitters',
        splitters: 'splitters',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createTargetOperationSummary({
        type: 'mesh-split',
        operation: 'split',
        targetValue: resolveItem(inputs.mesh),
        secondaryValues: inputs.splitters,
        kind: 'mesh',
        collectionName: 'splitters',
        secondaryRole: 'splitter',
        countLabel: 'splitterCount',
      });
      return { result: summary ? [summary] : [] };
    },
  });

  register('{b57bf805-046a-4360-ad76-51aeddfe9720}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'boundaries',
        Boundaries: 'boundaries',
        boundaries: 'boundaries',
      },
      outputs: {
        S: 'solid',
        Solid: 'solid',
        solid: 'solid',
      },
    },
    eval: ({ inputs }) => {
      const summary = createBoundaryVolumeSummary(inputs.boundaries);
      return { solid: summary };
    },
  });

  register('{ef6b26f4-f820-48d6-b0c5-85898ef8888b}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'brep',
        Brep: 'brep',
        brep: 'brep',
        C: 'cutter',
        Cutter: 'cutter',
        cutter: 'cutter',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const brep = resolveItem(inputs.brep);
      if (!brep) {
        return { result: [] };
      }
      const cutters = ensureArray(inputs.cutter);
      const summary = createTargetOperationSummary({
        type: 'brep-split',
        operation: 'split',
        targetValue: brep,
        secondaryValues: cutters,
        kind: 'brep',
        collectionName: 'cutters',
        secondaryRole: 'cutter',
        countLabel: 'cutterCount',
        extras: {
          mode: cutters.length > 1 ? 'multiple' : 'single',
        },
      });
      return { result: summary ? [summary] : [] };
    },
  });

  register('{f0b70e8e-7337-4ce4-a7bb-317fc971f918}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        S: 'shape',
        Shape: 'shape',
        shape: 'shape',
        T: 'cutters',
        Cutters: 'cutters',
        cutters: 'cutters',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createTargetOperationSummary({
        type: 'solid-trim',
        operation: 'trim',
        targetValue: resolveItem(inputs.shape),
        secondaryValues: inputs.cutters,
        kind: 'shape',
        collectionName: 'cutters',
        secondaryRole: 'cutter',
        countLabel: 'cutterCount',
      });
      return { result: summary ? [summary] : [] };
    },
  });

  register('{f72c480b-7ee6-42ef-9821-c371e9203b44}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'curvesA',
        'Curves A': 'curvesA',
        B: 'curvesB',
        'Curves B': 'curvesB',
        P: 'plane',
        Plane: 'plane',
        plane: 'plane',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createRegionBooleanSummary('difference', inputs.curvesA, inputs.curvesB, {}, inputs.plane);
      return { result: summary ? [summary] : [] };
    },
  });

  register('{fab11c30-2d9c-4d15-ab3c-2289f1ae5c21}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'brepsA',
        'Breps A': 'brepsA',
        B: 'brepsB',
        'Breps B': 'brepsB',
      },
      outputs: {
        R: 'result',
        Result: 'result',
        result: 'result',
      },
    },
    eval: ({ inputs }) => {
      const summary = createBrepBooleanSummary('difference', inputs.brepsA, inputs.brepsB);
      return { result: summary ? [summary] : [] };
    },
  });
}

export function registerIntersectPhysicalComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register intersect physical components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register intersect physical components.');
  }

  const {
    ensureNumber,
    ensureArray,
    resolveItem,
    annotateShape,
  } = createIntersectComponentUtils({ toNumber });

  function createPrimarySecondarySummary({
    type,
    operation,
    primaryValue,
    primaryKind,
    secondaryValues = [],
    secondaryKind = null,
    extras = {},
  }) {
    const primary = primaryValue
      ? annotateShape(primaryValue, { operation, role: 'primary' }, { kind: primaryKind ?? 'shape' })
      : null;
    const secondary = ensureArray(secondaryValues).map((value, index) =>
      annotateShape(value, { operation, role: 'secondary', index }, { kind: secondaryKind ?? primaryKind ?? 'shape' })
    );
    if (!primary && !secondary.length) {
      return null;
    }
    const summary = {
      type,
      operation,
      metadata: {
        operation,
        hasPrimary: Boolean(primary),
        secondaryCount: secondary.length,
        ...extras,
      },
    };
    const shapes = [];
    if (primary) {
      summary.primary = primary;
      shapes.push(primary);
    }
    if (secondary.length) {
      summary.secondary = secondary;
      shapes.push(...secondary);
    }
    if (shapes.length) {
      summary.shapes = shapes;
    }
    return summary;
  }

  function createCollectionSummary({
    type,
    operation,
    values,
    kind = 'shape',
    role = 'participant',
    collectionName = 'participants',
    extras = {},
  }) {
    const list = ensureArray(values).map((value, index) =>
      annotateShape(value, { operation, role, index }, { kind })
    );
    if (!list.length) {
      return null;
    }
    const summary = {
      type,
      operation,
      metadata: {
        operation,
        [collectionName === 'participants' ? 'participantCount' : `${collectionName}Count`]: list.length,
        role,
        ...extras,
      },
      shapes: list,
    };
    summary[collectionName] = list;
    return summary;
  }

  function createSummaryOutput(summary, role, extras = {}) {
    if (!summary) {
      return null;
    }
    return {
      type: 'intersect-output',
      role,
      summary,
      metadata: {
        role,
        operation: summary.operation,
        ...extras,
      },
    };
  }

  function outputList(summary, role, extras = {}) {
    const entry = createSummaryOutput(summary, role, extras);
    return entry ? [entry] : [];
  }

  function outputItem(summary, role, extras = {}) {
    return createSummaryOutput(summary, role, extras);
  }

  function createCollisionSummary({ mode, colliderValue, obstacleValues = [] }) {
    if (mode === 'one-many') {
      return createPrimarySecondarySummary({
        type: 'collision-query',
        operation: 'collision',
        primaryValue: colliderValue,
        primaryKind: 'shape',
        secondaryValues: obstacleValues,
        extras: {
          mode,
        },
      });
    }
    return createCollectionSummary({
      type: 'collision-query',
      operation: 'collision',
      values: colliderValue,
      kind: 'shape',
      role: 'collider',
      collectionName: 'colliders',
      extras: {
        mode,
      },
    });
  }

  register('{0991ac99-6a0b-47a9-b07d-dd510ca57f0f}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        C: 'curve',
        Curve: 'curve',
        curve: 'curve',
      },
      outputs: {
        P: 'points',
        Points: 'points',
        points: 'points',
        t: 'params',
        Params: 'params',
        params: 'params',
      },
    },
    eval: ({ inputs }) => {
      const curve = resolveItem(inputs.curve);
      const summary = createPrimarySecondarySummary({
        type: 'curve-self-intersection',
        operation: 'curve-self',
        primaryValue: curve,
        primaryKind: 'curve',
        extras: { mode: 'self' },
      });
      return {
        points: outputList(summary, 'points', { expects: 'points' }),
        params: outputList(summary, 'parameters', { expects: 'parameters' }),
      };
    },
  });

  register('{19632848-4b95-4e5e-9e86-b79b47987a46}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        M: 'mesh',
        Mesh: 'mesh',
        mesh: 'mesh',
        C: 'curve',
        Curve: 'curve',
        curve: 'curve',
      },
      outputs: {
        X: 'points',
        Points: 'points',
        points: 'points',
        F: 'faces',
        Faces: 'faces',
        faces: 'faces',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'mesh-curve-intersection',
        operation: 'mesh-curve',
        primaryValue: resolveItem(inputs.mesh),
        primaryKind: 'mesh',
        secondaryValues: inputs.curve,
        secondaryKind: 'curve',
      });
      return {
        points: outputList(summary, 'points', { expects: 'points' }),
        faces: outputList(summary, 'faces', { expects: 'face-indices' }),
      };
    },
  });

  register('{20ef81e8-df15-4a0c-acf1-993a7607cafb}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        B: 'brep',
        Brep: 'brep',
        brep: 'brep',
        C: 'curve',
        Curve: 'curve',
        curve: 'curve',
      },
      outputs: {
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
        P: 'points',
        Points: 'points',
        points: 'points',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'brep-curve-intersection',
        operation: 'brep-curve',
        primaryValue: resolveItem(inputs.brep),
        primaryKind: 'brep',
        secondaryValues: inputs.curve,
        secondaryKind: 'curve',
      });
      return {
        curves: outputList(summary, 'curves', { expects: 'curves' }),
        points: outputList(summary, 'points', { expects: 'points' }),
      };
    },
  });

  register('{2168853c-acd8-4a63-9c9b-ecde9e239eae}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        C: 'colliders',
        Colliders: 'colliders',
        colliders: 'colliders',
      },
      outputs: {
        C: 'collision',
        Collision: 'collision',
        collision: 'collision',
        I: 'indices',
        Indices: 'indices',
        indices: 'indices',
      },
    },
    eval: ({ inputs }) => {
      const summary = createCollisionSummary({ mode: 'many-many', colliderValue: inputs.colliders });
      return {
        collision: outputList(summary, 'collisions', { expects: 'collider-status' }),
        indices: outputList(summary, 'indices', { expects: 'collider-index' }),
      };
    },
  });

  register('{21b6a605-9568-4bf8-acc1-631565d609d7}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'meshA',
        'Mesh A': 'meshA',
        B: 'meshB',
        'Mesh B': 'meshB',
      },
      outputs: {
        X: 'intersections',
        Intersections: 'intersections',
        intersections: 'intersections',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'mesh-mesh-intersection',
        operation: 'mesh-mesh',
        primaryValue: resolveItem(inputs.meshA),
        primaryKind: 'mesh',
        secondaryValues: resolveItem(inputs.meshB),
        secondaryKind: 'mesh',
      });
      return {
        intersections: outputList(summary, 'curves', { expects: 'intersection-curves' }),
      };
    },
  });

  register('{4439a51b-8d24-4924-b8e2-f77e7f8f5bec}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'firstSet',
        'First Set': 'firstSet',
        B: 'secondSet',
        'Second Set': 'secondSet',
        D: 'distance',
        Distance: 'distance',
        d: 'distance',
        L: 'limit',
        'Result Limit': 'limit',
        limit: 'limit',
      },
      outputs: {
        N: 'count',
        'Clash Count': 'count',
        count: 'count',
        P: 'points',
        'Clash Points': 'points',
        points: 'points',
        R: 'radii',
        'Clash Radii': 'radii',
        radii: 'radii',
        i: 'firstIndex',
        'First Index': 'firstIndex',
        first: 'firstIndex',
        j: 'secondIndex',
        'Second index': 'secondIndex',
        second: 'secondIndex',
      },
    },
    eval: ({ inputs }) => {
      const firstSet = ensureArray(inputs.firstSet).map((value, index) =>
        annotateShape(value, { operation: 'clash', role: 'first', index }, { kind: 'shape' })
      );
      const secondSet = ensureArray(inputs.secondSet).map((value, index) =>
        annotateShape(value, { operation: 'clash', role: 'second', index }, { kind: 'shape' })
      );
      const distance = ensureNumber(inputs.distance, 0);
      const limit = ensureNumber(inputs.limit, 0);
      const summary = {
        type: 'clash-analysis',
        operation: 'clash',
        first: firstSet,
        second: secondSet,
        shapes: [...firstSet, ...secondSet],
        metadata: {
          operation: 'clash',
          firstCount: firstSet.length,
          secondCount: secondSet.length,
          distance,
          limit,
        },
      };
      return {
        count: outputItem(summary, 'count', { expects: 'clash-count' }),
        points: outputList(summary, 'points', { expects: 'clash-points' }),
        radii: outputList(summary, 'radii', { expects: 'clash-radii' }),
        firstIndex: outputList(summary, 'first-index', { expects: 'first-collider-index' }),
        secondIndex: outputList(summary, 'second-index', { expects: 'second-collider-index' }),
      };
    },
  });

  register('{68546dd0-aa82-471c-87e9-81cb16ac50ed}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        S: 'surface',
        Surface: 'surface',
        surface: 'surface',
        C: 'curve',
        Curve: 'curve',
        curve: 'curve',
      },
      outputs: {
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
        P: 'points',
        Points: 'points',
        points: 'points',
        uv: 'uvPoints',
        'UV Points': 'uvPoints',
        UV: 'uvPoints',
        N: 'normals',
        Normals: 'normals',
        normals: 'normals',
        t: 'parameters',
        Parameters: 'parameters',
        parameters: 'parameters',
        T: 'tangents',
        Tangents: 'tangents',
        tangents: 'tangents',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'surface-curve-intersection',
        operation: 'surface-curve',
        primaryValue: resolveItem(inputs.surface),
        primaryKind: 'surface',
        secondaryValues: inputs.curve,
        secondaryKind: 'curve',
      });
      return {
        curves: outputList(summary, 'curves', { expects: 'curves' }),
        points: outputList(summary, 'points', { expects: 'points' }),
        uvPoints: outputList(summary, 'uv', { expects: 'uv-coordinates' }),
        normals: outputList(summary, 'normals', { expects: 'normals' }),
        parameters: outputList(summary, 'parameters', { expects: 'curve-parameters' }),
        tangents: outputList(summary, 'tangents', { expects: 'curve-tangents' }),
      };
    },
  });

  register('{7db14002-c09c-4d7b-9f80-e4e2b00dfa1d}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        S: 'surface',
        Surface: 'surface',
        surface: 'surface',
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
      },
      outputs: {
        F: 'fragments',
        Fragments: 'fragments',
        fragments: 'fragments',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'surface-split',
        operation: 'surface-split',
        primaryValue: resolveItem(inputs.surface),
        primaryKind: 'surface',
        secondaryValues: inputs.curves,
        secondaryKind: 'curve',
      });
      return {
        fragments: outputList(summary, 'fragments', { expects: 'surface-fragments' }),
      };
    },
  });

  register('{84627490-0fb2-4498-8138-ad134ee4cb36}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'curveA',
        'Curve A': 'curveA',
        B: 'curveB',
        'Curve B': 'curveB',
      },
      outputs: {
        P: 'points',
        Points: 'points',
        points: 'points',
        tA: 'paramsA',
        'Params A': 'paramsA',
        paramsA: 'paramsA',
        tB: 'paramsB',
        'Params B': 'paramsB',
        paramsB: 'paramsB',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'curve-curve-intersection',
        operation: 'curve-curve',
        primaryValue: resolveItem(inputs.curveA),
        primaryKind: 'curve',
        secondaryValues: resolveItem(inputs.curveB),
        secondaryKind: 'curve',
      });
      return {
        points: outputList(summary, 'points', { expects: 'points' }),
        paramsA: outputList(summary, 'params-a', { expects: 'curve-a-parameters' }),
        paramsB: outputList(summary, 'params-b', { expects: 'curve-b-parameters' }),
      };
    },
  });

  register('{904e4b56-484a-4814-b35f-aa4baf362117}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        A: 'brepA',
        'Brep A': 'brepA',
        B: 'brepB',
        'Brep B': 'brepB',
      },
      outputs: {
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
        P: 'points',
        Points: 'points',
        points: 'points',
      },
    },
    eval: ({ inputs }) => {
      const summary = createPrimarySecondarySummary({
        type: 'brep-brep-intersection',
        operation: 'brep-brep',
        primaryValue: resolveItem(inputs.brepA),
        primaryKind: 'brep',
        secondaryValues: resolveItem(inputs.brepB),
        secondaryKind: 'brep',
      });
      return {
        curves: outputList(summary, 'curves', { expects: 'curves' }),
        points: outputList(summary, 'points', { expects: 'points' }),
      };
    },
  });

  register('{931e6030-ccb3-4a7b-a89a-99dcce8770cd}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        C: 'curves',
        Curves: 'curves',
        curves: 'curves',
      },
      outputs: {
        P: 'points',
        Points: 'points',
        points: 'points',
        iA: 'indexA',
        'Index A': 'indexA',
        indexA: 'indexA',
        iB: 'indexB',
        'Index B': 'indexB',
        indexB: 'indexB',
        tA: 'paramA',
        'Param A': 'paramA',
        paramA: 'paramA',
        tB: 'paramB',
        'Param B': 'paramB',
        paramB: 'paramB',
      },
    },
    eval: ({ inputs }) => {
      const summary = createCollectionSummary({
        type: 'multi-curve-intersection',
        operation: 'curve-multi',
        values: inputs.curves,
        kind: 'curve',
        role: 'curve',
        collectionName: 'curves',
      });
      return {
        points: outputList(summary, 'points', { expects: 'points' }),
        indexA: outputList(summary, 'index-a', { expects: 'curve-a-index' }),
        indexB: outputList(summary, 'index-b', { expects: 'curve-b-index' }),
        paramA: outputList(summary, 'param-a', { expects: 'curve-a-parameter' }),
        paramB: outputList(summary, 'param-b', { expects: 'curve-b-parameter' }),
      };
    },
  });

  register('{bb6c6501-0500-4678-859b-b838348981d1}', {
    type: 'intersect',
    pinMap: {
      inputs: {
        C: 'collider',
        Collider: 'collider',
        collider: 'collider',
        O: 'obstacles',
        Obstacles: 'obstacles',
        obstacles: 'obstacles',
      },
      outputs: {
        C: 'collision',
        Collision: 'collision',
        collision: 'collision',
        I: 'index',
        Index: 'index',
        index: 'index',
      },
    },
    eval: ({ inputs }) => {
      const summary = createCollisionSummary({
        mode: 'one-many',
        colliderValue: resolveItem(inputs.collider),
        obstacleValues: inputs.obstacles,
      });
      return {
        collision: outputItem(summary, 'collisions', { expects: 'collision-flag' }),
        index: outputItem(summary, 'index', { expects: 'obstacle-index' }),
      };
    },
  });
}

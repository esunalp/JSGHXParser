export function registerIntersectShapeComponents({ register, toNumber }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register intersect shape components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register intersect shape components.');
  }

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

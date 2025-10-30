import { loadThreeWebGPU } from './three-loader.js';

const THREE = await loadThreeWebGPU();

function cloneTypedArray(array) {
  if (!array) {
    return null;
  }
  if (typeof array.slice === 'function') {
    return array.slice(0);
  }
  const Ctor = array.constructor;
  return new Ctor(array);
}

function serializeBufferAttribute(attribute, transferables) {
  if (!attribute || !attribute.array) {
    return null;
  }
  const clonedArray = cloneTypedArray(attribute.array);
  if (!clonedArray) {
    return null;
  }
  if (transferables && clonedArray.buffer) {
    transferables.push(clonedArray.buffer);
  }
  return {
    array: clonedArray,
    itemSize: Number(attribute.itemSize) || 1,
    normalized: Boolean(attribute.normalized),
    count: Number(attribute.count) || Math.floor(clonedArray.length / (Number(attribute.itemSize) || 1)),
  };
}

function serializeBoundingBox(box) {
  if (!box || !box.isBox3) {
    return null;
  }
  return {
    min: box.min.toArray(),
    max: box.max.toArray(),
  };
}

function serializeBoundingSphere(sphere) {
  if (!sphere || !sphere.isSphere) {
    return null;
  }
  return {
    center: sphere.center.toArray(),
    radius: sphere.radius,
  };
}

function serializeBufferGeometry(geometry, transferables) {
  if (!geometry || !geometry.isBufferGeometry) {
    return null;
  }
  const attributes = {};
  for (const [name, attribute] of Object.entries(geometry.attributes ?? {})) {
    const serialized = serializeBufferAttribute(attribute, transferables);
    if (serialized) {
      attributes[name] = serialized;
    }
  }
  let index = null;
  if (geometry.index) {
    index = serializeBufferAttribute(geometry.index, transferables);
  }
  const drawRange = geometry.drawRange
    ? { start: geometry.drawRange.start ?? 0, count: geometry.drawRange.count ?? Infinity }
    : null;
  return {
    kind: 'buffer-geometry',
    attributes,
    index,
    drawRange,
    boundingBox: serializeBoundingBox(geometry.boundingBox),
    boundingSphere: serializeBoundingSphere(geometry.boundingSphere),
  };
}

function serializeMaterial(material) {
  if (!material) {
    return null;
  }
  const base = {
    type: material.type ?? 'Material',
    color: material.color?.toArray?.() ?? null,
    opacity: Number.isFinite(material.opacity) ? material.opacity : 1,
    transparent: Boolean(material.transparent),
    wireframe: Boolean(material.wireframe),
    side: Number.isFinite(material.side) ? material.side : undefined,
  };
  if (material.isPointsMaterial) {
    base.size = Number.isFinite(material.size) ? material.size : undefined;
    base.sizeAttenuation = material.sizeAttenuation !== undefined ? Boolean(material.sizeAttenuation) : undefined;
  }
  if (material.isLineBasicMaterial || material.isLineDashedMaterial) {
    base.linewidth = Number.isFinite(material.linewidth) ? material.linewidth : undefined;
    base.dashed = Boolean(material.isLineDashedMaterial);
    if (material.dashSize !== undefined) {
      base.dashSize = material.dashSize;
    }
    if (material.gapSize !== undefined) {
      base.gapSize = material.gapSize;
    }
  }
  return base;
}

function extractTransform(object) {
  if (!object) {
    return {};
  }
  return {
    position: object.position?.toArray?.() ?? null,
    quaternion: object.quaternion?.toArray?.() ?? null,
    scale: object.scale?.toArray?.() ?? null,
    matrix: object.matrix?.elements ? object.matrix.elements.slice() : null,
    matrixAutoUpdate: object.matrixAutoUpdate !== undefined ? Boolean(object.matrixAutoUpdate) : true,
    visible: object.visible !== undefined ? Boolean(object.visible) : true,
    name: typeof object.name === 'string' ? object.name : null,
    castShadow: Boolean(object.castShadow),
    receiveShadow: Boolean(object.receiveShadow),
  };
}

function serializeObject3D(object, transferables, visited = new Set()) {
  if (!object) {
    return null;
  }
  if (visited.has(object)) {
    return null;
  }
  visited.add(object);

  if (Array.isArray(object)) {
    const serializedArray = object
      .map((entry) => serializeObject3D(entry, transferables, visited))
      .filter(Boolean);
    return serializedArray.length ? serializedArray : null;
  }

  if (object.isBufferGeometry) {
    return serializeBufferGeometry(object, transferables);
  }

  if (object.type === 'field-display') {
    const payload = object.payload ?? null;
    return {
      kind: 'field-display',
      payload,
    };
  }

  const transform = extractTransform(object);
  const base = {
    kind: 'object3d',
    transform,
    children: [],
  };

  if (object.isMesh || object.isSkinnedMesh) {
    const geometry = serializeBufferGeometry(object.geometry, transferables);
    const material = Array.isArray(object.material)
      ? object.material.map((entry) => serializeMaterial(entry))
      : serializeMaterial(object.material);
    const children = (object.children ?? [])
      .map((child) => serializeObject3D(child, transferables, visited))
      .filter(Boolean);
    return {
      ...base,
      kind: object.isSkinnedMesh ? 'skinned-mesh' : 'mesh',
      geometry,
      material,
      children,
    };
  }

  if (object.isLine || object.isLineSegments) {
    const geometry = serializeBufferGeometry(object.geometry, transferables);
    const material = serializeMaterial(object.material);
    const children = (object.children ?? [])
      .map((child) => serializeObject3D(child, transferables, visited))
      .filter(Boolean);
    return {
      ...base,
      kind: object.isLineSegments ? 'line-segments' : 'line',
      geometry,
      material,
      children,
    };
  }

  if (object.isPoints) {
    const geometry = serializeBufferGeometry(object.geometry, transferables);
    const material = serializeMaterial(object.material);
    const children = (object.children ?? [])
      .map((child) => serializeObject3D(child, transferables, visited))
      .filter(Boolean);
    return {
      ...base,
      kind: 'points',
      geometry,
      material,
      children,
    };
  }

  if (object.isGroup || object.isObject3D) {
    const children = (object.children ?? [])
      .map((child) => serializeObject3D(child, transferables, visited))
      .filter(Boolean);
    return {
      ...base,
      kind: object.isGroup ? 'group' : 'object3d',
      children,
    };
  }

  return null;
}

function serializeOverlayData(overlay) {
  if (!overlay || typeof overlay !== 'object') {
    return { segments: [], points: [] };
  }
  const segments = Array.isArray(overlay.segments)
    ? overlay.segments
        .map((segment) => {
          const start = segment?.start?.toArray?.();
          const end = segment?.end?.toArray?.();
          if (!start || !end) {
            return null;
          }
          return { start, end };
        })
        .filter(Boolean)
    : [];
  const points = Array.isArray(overlay.points)
    ? overlay.points
        .map((point) => (point?.toArray ? point.toArray() : null))
        .filter(Boolean)
    : [];
  return { segments, points };
}

export function serializeDisplayPayload(displayPayload) {
  const transferables = [];
  if (!displayPayload) {
    return { payload: null, transferables };
  }
  const serializedMain = serializeObject3D(displayPayload.main, transferables);
  const serialized = {
    type: displayPayload.type ?? null,
    main: serializedMain,
    overlays: serializeOverlayData(displayPayload.overlays),
  };
  if (displayPayload.graphId !== undefined && displayPayload.graphId !== null) {
    serialized.graphId = String(displayPayload.graphId);
  }
  if (displayPayload.graphMetadata && typeof displayPayload.graphMetadata === 'object') {
    serialized.graphMetadata = { ...displayPayload.graphMetadata };
  }
  return { payload: serialized, transferables };
}

function deserializeBufferAttribute(entry) {
  if (!entry || !entry.array) {
    return null;
  }
  return new THREE.BufferAttribute(entry.array, entry.itemSize ?? 1, Boolean(entry.normalized));
}

function deserializeBufferGeometry(data) {
  if (!data) {
    return null;
  }
  const geometry = new THREE.BufferGeometry();
  for (const [name, attribute] of Object.entries(data.attributes ?? {})) {
    const bufferAttribute = deserializeBufferAttribute(attribute);
    if (bufferAttribute) {
      geometry.setAttribute(name, bufferAttribute);
    }
  }
  if (data.index) {
    const indexAttribute = deserializeBufferAttribute(data.index);
    if (indexAttribute) {
      geometry.setIndex(indexAttribute);
    }
  }
  if (data.drawRange) {
    geometry.setDrawRange(data.drawRange.start ?? 0, data.drawRange.count ?? Infinity);
  }
  if (data.boundingBox?.min && data.boundingBox?.max) {
    geometry.boundingBox = new THREE.Box3(
      new THREE.Vector3().fromArray(data.boundingBox.min),
      new THREE.Vector3().fromArray(data.boundingBox.max),
    );
  }
  if (data.boundingSphere?.center && Number.isFinite(data.boundingSphere.radius)) {
    geometry.boundingSphere = new THREE.Sphere(
      new THREE.Vector3().fromArray(data.boundingSphere.center),
      data.boundingSphere.radius,
    );
  }
  return geometry;
}

function deserializeMaterial(entry) {
  if (!entry) {
    return null;
  }
  const color = Array.isArray(entry.color) ? new THREE.Color().fromArray(entry.color) : undefined;
  const commonOptions = {
    color,
    opacity: entry.opacity,
    transparent: entry.transparent,
    wireframe: entry.wireframe,
    side: entry.side,
  };
  switch (entry.type) {
    case 'MeshBasicMaterial':
      return new THREE.MeshBasicMaterial(commonOptions);
    case 'MeshLambertMaterial':
      return new THREE.MeshLambertMaterial(commonOptions);
    case 'MeshPhongMaterial':
      return new THREE.MeshPhongMaterial(commonOptions);
    case 'MeshPhysicalMaterial':
      return new THREE.MeshPhysicalMaterial(commonOptions);
    case 'MeshStandardMaterial':
    default:
      if (entry.type === 'MeshStandardMaterial' || entry.type === 'MeshPhysicalMaterial' || entry.type === 'MeshLambertMaterial' || entry.type === 'MeshPhongMaterial') {
        return new THREE.MeshStandardMaterial(commonOptions);
      }
      if (entry.type === 'LineDashedMaterial') {
        return new THREE.LineDashedMaterial({
          ...commonOptions,
          linewidth: entry.linewidth,
          dashSize: entry.dashSize,
          gapSize: entry.gapSize,
        });
      }
      if (entry.type === 'LineBasicMaterial') {
        return new THREE.LineBasicMaterial({
          ...commonOptions,
          linewidth: entry.linewidth,
        });
      }
      if (entry.type === 'PointsMaterial') {
        return new THREE.PointsMaterial({
          ...commonOptions,
          size: entry.size ?? 1,
          sizeAttenuation: entry.sizeAttenuation !== undefined ? entry.sizeAttenuation : true,
        });
      }
      return new THREE.MeshStandardMaterial(commonOptions);
  }
}

function applyTransform(object, transform) {
  if (!object || !transform) {
    return;
  }
  if (Array.isArray(transform.position)) {
    object.position.fromArray(transform.position);
  }
  if (Array.isArray(transform.quaternion)) {
    object.quaternion.fromArray(transform.quaternion);
  }
  if (Array.isArray(transform.scale)) {
    object.scale.fromArray(transform.scale);
  }
  if (Array.isArray(transform.matrix) && transform.matrix.length === 16) {
    object.matrix.fromArray(transform.matrix);
    if (transform.matrixAutoUpdate === false) {
      object.matrixAutoUpdate = false;
      object.matrix.decompose(object.position, object.quaternion, object.scale);
    } else {
      object.matrixAutoUpdate = true;
    }
  } else if (transform.matrixAutoUpdate === false) {
    object.matrixAutoUpdate = false;
    object.updateMatrix();
  } else {
    object.matrixAutoUpdate = true;
  }
  if (typeof transform.visible === 'boolean') {
    object.visible = transform.visible;
  }
  if (typeof transform.name === 'string') {
    object.name = transform.name;
  }
  if (typeof transform.castShadow === 'boolean') {
    object.castShadow = transform.castShadow;
  }
  if (typeof transform.receiveShadow === 'boolean') {
    object.receiveShadow = transform.receiveShadow;
  }
}

function deserializeObject3D(data) {
  if (!data) {
    return null;
  }
  if (Array.isArray(data)) {
    return data.map((entry) => deserializeObject3D(entry)).filter(Boolean);
  }
  if (data.kind === 'buffer-geometry') {
    return deserializeBufferGeometry(data);
  }
  if (data.kind === 'field-display') {
    return { type: 'field-display', payload: data.payload ?? null };
  }

  let object = null;
  if (data.kind === 'mesh' || data.kind === 'skinned-mesh') {
    const geometry = deserializeBufferGeometry(data.geometry);
    const material = Array.isArray(data.material)
      ? data.material.map((entry) => deserializeMaterial(entry))
      : deserializeMaterial(data.material);
    object = new THREE.Mesh(geometry, material);
  } else if (data.kind === 'line' || data.kind === 'line-segments') {
    const geometry = deserializeBufferGeometry(data.geometry);
    const material = deserializeMaterial(data.material);
    object = data.kind === 'line-segments'
      ? new THREE.LineSegments(geometry, material)
      : new THREE.Line(geometry, material);
  } else if (data.kind === 'points') {
    const geometry = deserializeBufferGeometry(data.geometry);
    const material = deserializeMaterial(data.material);
    object = new THREE.Points(geometry, material);
  } else if (data.kind === 'group' || data.kind === 'object3d') {
    object = new THREE.Group();
  }

  if (!object) {
    return null;
  }

  applyTransform(object, data.transform);

  if (Array.isArray(data.children) && data.children.length) {
    for (const childData of data.children) {
      const child = deserializeObject3D(childData);
      if (child) {
        object.add(child);
      }
    }
  }

  return object;
}

function deserializeOverlay(data) {
  const safe = { segments: [], points: [] };
  if (!data || typeof data !== 'object') {
    return safe;
  }
  if (Array.isArray(data.segments)) {
    for (const segment of data.segments) {
      const start = Array.isArray(segment?.start) ? new THREE.Vector3().fromArray(segment.start) : null;
      const end = Array.isArray(segment?.end) ? new THREE.Vector3().fromArray(segment.end) : null;
      if (start && end) {
        safe.segments.push({ start, end });
      }
    }
  }
  if (Array.isArray(data.points)) {
    for (const point of data.points) {
      if (Array.isArray(point)) {
        safe.points.push(new THREE.Vector3().fromArray(point));
      }
    }
  }
  return safe;
}

export function deserializeDisplayPayload(serialized) {
  if (!serialized) {
    return null;
  }
  const main = deserializeObject3D(serialized.main);
  const overlays = deserializeOverlay(serialized.overlays);
  const graphId = serialized.graphId !== undefined && serialized.graphId !== null
    ? String(serialized.graphId)
    : null;
  const graphMetadata = serialized.graphMetadata && typeof serialized.graphMetadata === 'object'
    ? { ...serialized.graphMetadata }
    : null;
  return {
    type: serialized.type ?? null,
    main,
    overlays,
    graphId,
    graphMetadata,
  };
}

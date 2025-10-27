import * as THREE from 'three/webgpu';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
// import { WebGPURenderer } from 'three/addons/renderers/webgpu/WebGPURenderer.js';
import { PhysicalSunSky } from './physical-sun-sky.js';
import { DDGIProbeVolume } from './ddgi-probe-volume.js';
import {
  cloneSurfaceMaterial,
  createStandardSurfaceMaterial,
  convertMaterialToNode,
  ensureGeometryHasVertexNormals,
} from './material-utils.js';

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

// Toggle to enable or disable double sided rendering for viewport meshes.
const ENABLE_DOUBLE_SIDED_MESHES = true;
const ENABLE_DDGI = false;
const DEFAULT_MESH_SIDE = ENABLE_DOUBLE_SIDED_MESHES ? THREE.DoubleSide : THREE.FrontSide;

const AXES_LENGTH_MM = 5000;
const MAX_DRAW_DISTANCE_MM = 100000;
const OVERLAY_SEGMENT_RADIUS_MM = 8;
const OVERLAY_SEGMENT_RADIAL_SEGMENTS = 12;

function getViewportSize(canvas) {
  const width = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth || 1;
  const height = canvas.clientHeight || canvas.parentElement?.clientHeight || window.innerHeight || 1;
  return {
    width: Math.max(width, 1),
    height: Math.max(height, 1),
    pixelRatio: Math.min(window.devicePixelRatio || 1, 2),
  };
}

function applyRendererDefaults(renderer) {
  if (!renderer) {
    return;
  }

  if ('outputColorSpace' in renderer && THREE.SRGBColorSpace) {
    renderer.outputColorSpace = THREE.SRGBColorSpace;
  } else if ('outputEncoding' in renderer) {
    renderer.outputEncoding = THREE.sRGBEncoding;
  }

  if ('physicallyCorrectLights' in renderer) {
    renderer.physicallyCorrectLights = true;
  }

  if ('toneMapping' in renderer && THREE.ACESFilmicToneMapping) {
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
  }

  if (renderer.shadowMap) {
    renderer.shadowMap.enabled = true;
    if ('type' in renderer.shadowMap && THREE.PCFSoftShadowMap) {
      renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    }
  }

  if (typeof renderer.setClearColor === 'function') {
    renderer.setClearColor(0x060910, 1);
  }

  if ('useLegacyLights' in renderer) {
    renderer.useLegacyLights = false;
  }
}

function applyViewportToRenderer(renderer, viewport) {
  if (!renderer || !viewport) {
    return;
  }

  if (typeof renderer.setPixelRatio === 'function') {
    renderer.setPixelRatio(viewport.pixelRatio);
  }

  if (typeof renderer.setSize === 'function') {
    renderer.setSize(viewport.width, viewport.height, false);
  }
}

async function createWebGPURenderer(canvas, viewport) {
  const renderer = new THREE.WebGPURenderer({ canvas, antialias: true });
  await renderer.init();
  applyRendererDefaults(renderer);
  applyViewportToRenderer(renderer, viewport);
  return renderer;
}

async function createWebGLRenderer(canvas, viewport) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  applyRendererDefaults(renderer);
  applyViewportToRenderer(renderer, viewport);
  return renderer;
}

const AXIS_REFERENCE_UP = new THREE.Vector3(0, 1, 0);
const AXIS_DIRECTIONS = [
  new THREE.Vector3(1, 0, 0),
  new THREE.Vector3(0, 1, 0),
  new THREE.Vector3(0, 0, 1),
];
const AXIS_COLOURS = [0xd1495b, 0x2fbf71, 0x3066be];

function createAxisComponent(length, color, direction) {
  if (!Number.isFinite(length) || length <= 0) {
    return null;
  }

  if (!direction?.isVector3) {
    return null;
  }

  const safeLength = Math.max(length, 1);
  const shaftLength = safeLength * 0.88;
  const tipLength = safeLength - shaftLength;
  const shaftRadius = Math.max(safeLength * 0.01, 15);
  const tipRadius = shaftRadius * 1.75;

  const shaftGeometry = new THREE.CylinderGeometry(shaftRadius, shaftRadius, shaftLength, 24, 1, true);
  const tipGeometry = new THREE.ConeGeometry(tipRadius, tipLength, 24, 1, true);

  const material = createStandardSurfaceMaterial(
    {
      color,
      metalness: 0.1,
      roughness: 0.35,
    },
    { side: DEFAULT_MESH_SIDE },
  );

  const axisGroup = new THREE.Group();
  axisGroup.name = 'GHXAxesHelperAxis';

  const shaft = new THREE.Mesh(shaftGeometry, material);
  shaft.position.y = shaftLength * 0.5;
  shaft.castShadow = false;
  shaft.receiveShadow = false;
  axisGroup.add(shaft);

  const tip = new THREE.Mesh(tipGeometry, material);
  tip.position.y = shaftLength;
  tip.castShadow = false;
  tip.receiveShadow = false;
  axisGroup.add(tip);

  const directionVector = direction.clone().normalize();
  const quaternion = new THREE.Quaternion().setFromUnitVectors(AXIS_REFERENCE_UP, directionVector);
  axisGroup.quaternion.copy(quaternion);

  axisGroup.userData.dispose = () => {
    try {
      shaftGeometry.dispose();
    } catch (error) {
      console.warn('Axes helper shaft dispose error', error);
    }
    try {
      tipGeometry.dispose();
    } catch (error) {
      console.warn('Axes helper tip dispose error', error);
    }
    try {
      material.dispose?.();
    } catch (error) {
      console.warn('Axes helper material dispose error', error);
    }
  };

  return axisGroup;
}

function createAxesHelper(length) {
  const axesGroup = new THREE.Group();
  axesGroup.name = 'GHXAxesHelper';

  AXIS_DIRECTIONS.forEach((direction, index) => {
    const axis = createAxisComponent(length, AXIS_COLOURS[index % AXIS_COLOURS.length], direction);
    if (axis) {
      axesGroup.add(axis);
    }
  });

  if (!axesGroup.children.length) {
    return null;
  }

  axesGroup.userData.dispose = () => {
    axesGroup.children.forEach((child) => {
      try {
        child.userData?.dispose?.();
      } catch (error) {
        console.warn('Axes helper dispose error', error);
      }
    });
  };

  return axesGroup;
}

function addHelpers(scene) {
  const axes = createAxesHelper(AXES_LENGTH_MM);
  if (axes) {
    scene.add(axes);
  }
}

const FIELD_EPSILON = 1e-6;
const FIELD_AXIS_COLOURS = [0xd1495b, 0x3066be, 0x2fbf71];
const OVERLAY_LINE_COLOR = new THREE.Color(0x000000);
const OVERLAY_POINT_COLOR = new THREE.Color(0x000000);
const POINT_SPHERE_RADIUS_MM = 10;
const POINT_SPHERE_WIDTH_SEGMENTS = 16;
const POINT_SPHERE_HEIGHT_SEGMENTS = 12;

function parseDelimitedColorText(text) {
  if (typeof text !== 'string') {
    return null;
  }

  const segments = text
    .split(/[;,]/)
    .map((segment) => segment.trim())
    .filter((segment) => segment.length);

  if (segments.length < 3) {
    return null;
  }

  const values = segments.slice(0, 3).map((segment) => Number(segment));
  if (!values.every((value) => Number.isFinite(value))) {
    return null;
  }

  const requiresScaling = values.some((value) => Math.abs(value) > 1);
  const [r, g, b] = requiresScaling
    ? values.map((value) => value / 255)
    : values;

  const clamp01 = (value) => {
    if (!Number.isFinite(value)) {
      return 0;
    }
    if (value <= 0) {
      return 0;
    }
    if (value >= 1) {
      return 1;
    }
    return value;
  };

  return new THREE.Color(clamp01(r), clamp01(g), clamp01(b));
}

function ensureColor(value, fallback = new THREE.Color(0xffffff)) {
  if (value?.isColor) {
    return value.clone();
  }
  if (typeof value === 'number') {
    return new THREE.Color(value);
  }
  if (typeof value === 'string') {
    try {
      const delimitedColor = parseDelimitedColorText(value.trim());
      if (delimitedColor) {
        return delimitedColor;
      }

      return new THREE.Color(value);
    } catch (error) {
      return fallback.clone();
    }
  }
  if (Array.isArray(value)) {
    const [r = 0, g = 0, b = 0] = value;
    return new THREE.Color(r, g, b);
  }
  return fallback.clone();
}

function applyMaterialSide(material, side) {
  if (!material) {
    return;
  }
  if (Array.isArray(material)) {
    material.forEach((entry) => applyMaterialSide(entry, side));
    return;
  }
  if (material && 'side' in material && material.side !== side) {
    material.side = side;
    material.needsUpdate = true;
  }
  if (material && 'shadowSide' in material && material.shadowSide !== side) {
    material.shadowSide = side;
    material.needsUpdate = true;
  }
}

function applyMeshSide(object, side) {
  if (!object?.isObject3D) {
    return;
  }
  object.traverse((child) => {
    if (child.isMesh) {
      const convertedMaterial = convertMaterialToNode(child.material, { side });
      if (convertedMaterial) {
        child.material = convertedMaterial;
      }
      applyMaterialSide(child.material, side);
      ensureGeometryHasVertexNormals(child.geometry);
    }
  });
}

function getEntryMagnitude(entry, fallback = 0) {
  if (!entry) {
    return fallback;
  }
  const { magnitude, strength } = entry;
  if (Number.isFinite(magnitude)) {
    return Math.abs(magnitude);
  }
  if (Number.isFinite(strength)) {
    return Math.abs(strength);
  }
  return fallback;
}

function createSegmentsObject(segments) {
  if (!Array.isArray(segments) || !segments.length) {
    return null;
  }

  const validSegments = segments
    .map((segment) => {
      const start = segment?.start;
      const end = segment?.end;
      if (!start?.isVector3 || !end?.isVector3) {
        return null;
      }
      const length = start.distanceTo(end);
      if (!(length > FIELD_EPSILON)) {
        return null;
      }

      const startColour = ensureColor(segment.colorStart ?? segment.color ?? OVERLAY_LINE_COLOR);
      const endColour = ensureColor(segment.colorEnd ?? segment.color ?? startColour);
      const color = startColour.clone().lerp(endColour, 0.5);

      return {
        start,
        end,
        length,
        color,
      };
    })
    .filter((entry) => entry);

  if (!validSegments.length) {
    return null;
  }

  const geometry = new THREE.CylinderGeometry(
    OVERLAY_SEGMENT_RADIUS_MM,
    OVERLAY_SEGMENT_RADIUS_MM,
    1,
    OVERLAY_SEGMENT_RADIAL_SEGMENTS,
    1,
    true,
  );

  const material = createStandardSurfaceMaterial(
    {
      color: 0xffffff,
      metalness: 0.15,
      roughness: 0.5,
      vertexColors: true,
    },
    { side: DEFAULT_MESH_SIDE },
  );

  const object = new THREE.InstancedMesh(geometry, material, validSegments.length);
  object.castShadow = false;
  object.receiveShadow = false;

  const up = new THREE.Vector3(0, 1, 0);
  const matrix = new THREE.Matrix4();
  const position = new THREE.Vector3();
  const direction = new THREE.Vector3();
  const quaternion = new THREE.Quaternion();
  const scale = new THREE.Vector3();
  const color = new THREE.Color();

  validSegments.forEach((entry, index) => {
    direction.subVectors(entry.end, entry.start);
    const length = entry.length;
    if (!(length > FIELD_EPSILON)) {
      return;
    }
    direction.normalize();
    position.copy(entry.start).addScaledVector(direction, length * 0.5);
    quaternion.setFromUnitVectors(up, direction);
    scale.set(OVERLAY_SEGMENT_RADIUS_MM, length, OVERLAY_SEGMENT_RADIUS_MM);
    matrix.compose(position, quaternion, scale);
    object.setMatrixAt(index, matrix);

    color.copy(entry.color);
    object.setColorAt(index, color);
  });

  object.instanceMatrix.needsUpdate = true;
  if (object.instanceColor) {
    object.instanceColor.needsUpdate = true;
  }

  return { object, disposables: [geometry, material, object.instanceColor] };
}

function createPointsObject(entries, colourFactory) {
  if (!Array.isArray(entries) || !entries.length) {
    return null;
  }

  const validCount = entries.reduce((count, entry) => (entry?.point?.isVector3 ? count + 1 : count), 0);
  if (!validCount) {
    return null;
  }

  const geometry = new THREE.SphereGeometry(
    POINT_SPHERE_RADIUS_MM,
    POINT_SPHERE_WIDTH_SEGMENTS,
    POINT_SPHERE_HEIGHT_SEGMENTS,
  );

  const material = createStandardSurfaceMaterial(
    {
      color: 0xffffff,
      vertexColors: true,
      metalness: 0,
      roughness: 0.55,
    },
    { side: DEFAULT_MESH_SIDE },
  );

  const object = new THREE.InstancedMesh(geometry, material, validCount);
  const matrix = new THREE.Matrix4();
  const instanceColours = new Float32Array(validCount * 3);

  let instanceIndex = 0;
  entries.forEach((entry, index) => {
    const point = entry?.point;
    if (!point?.isVector3) {
      return;
    }

    matrix.makeTranslation(point.x, point.y, point.z);
    object.setMatrixAt(instanceIndex, matrix);

    const colour = ensureColor(
      typeof colourFactory === 'function' ? colourFactory(entry, index) : OVERLAY_POINT_COLOR,
      OVERLAY_POINT_COLOR,
    );
    const offset = instanceIndex * 3;
    instanceColours[offset + 0] = colour.r;
    instanceColours[offset + 1] = colour.g;
    instanceColours[offset + 2] = colour.b;

    instanceIndex += 1;
  });

  object.instanceMatrix.needsUpdate = true;
  object.instanceColor = new THREE.InstancedBufferAttribute(instanceColours, 3);
  object.instanceColor.needsUpdate = true;
  object.castShadow = false;
  object.receiveShadow = false;

  return { object, disposables: [geometry, material, object.instanceColor] };
}

function buildDirectionSegments(entries) {
  const validEntries = entries.filter((entry) => entry?.point?.isVector3
    && entry?.direction?.isVector3
    && entry.direction.lengthSq() > FIELD_EPSILON);

  if (!validEntries.length) {
    return [];
  }

  let maxMagnitude = 0;
  validEntries.forEach((entry) => {
    const magnitude = getEntryMagnitude(entry, 0);
    if (Number.isFinite(magnitude) && magnitude > maxMagnitude) {
      maxMagnitude = magnitude;
    }
  });

  const safeMax = maxMagnitude > FIELD_EPSILON ? maxMagnitude : 1;
  const baseColour = new THREE.Color(0xf4d35e);
  const headColour = baseColour.clone().offsetHSL(0, 0, 0.1);

  return validEntries.map((entry) => {
    const start = entry.point;
    const direction = entry.direction.clone().normalize();
    const magnitude = getEntryMagnitude(entry, safeMax);
    const normalized = safeMax > FIELD_EPSILON ? Math.min(magnitude / safeMax, 1) : 0;
    const length = 0.35 + normalized * 0.75;
    const end = start.clone().add(direction.multiplyScalar(length));
    return {
      start,
      end,
      colorStart: baseColour.clone(),
      colorEnd: headColour.clone(),
    };
  });
}

function buildTensorSegments(entries) {
  const segments = buildDirectionSegments(entries);

  const axisMagnitudes = [];
  entries.forEach((entry) => {
    const principalAxes = Array.isArray(entry?.principal) ? entry.principal : [];
    principalAxes.forEach((axis) => {
      const magnitude = getEntryMagnitude(axis, null);
      if (Number.isFinite(magnitude)) {
        axisMagnitudes.push(Math.abs(magnitude));
      }
    });
  });

  if (!axisMagnitudes.length) {
    return segments;
  }

  const maxAxisMagnitude = Math.max(...axisMagnitudes, FIELD_EPSILON);

  entries.forEach((entry) => {
    const start = entry?.point;
    if (!start?.isVector3) {
      return;
    }
    const principalAxes = Array.isArray(entry?.principal) ? entry.principal : [];
    principalAxes.forEach((axis, axisIndex) => {
      const direction = axis?.direction;
      if (!direction?.isVector3 || direction.lengthSq() <= FIELD_EPSILON) {
        return;
      }
      const magnitude = getEntryMagnitude(axis, 0);
      const normalized = maxAxisMagnitude > FIELD_EPSILON ? Math.min(magnitude / maxAxisMagnitude, 1) : 0;
      const length = 0.25 + normalized * 0.6;
      const end = start.clone().add(direction.clone().normalize().multiplyScalar(length));
      const baseColour = new THREE.Color(FIELD_AXIS_COLOURS[axisIndex % FIELD_AXIS_COLOURS.length]);
      const headColour = baseColour.clone().offsetHSL(0, 0, 0.1);
      segments.push({
        start,
        end,
        colorStart: baseColour,
        colorEnd: headColour,
      });
    });
  });

  return segments;
}

function createFieldDisplayGroup(display) {
  if (!display || display.type !== 'field-display') {
    return null;
  }

  const group = new THREE.Group();
  group.name = 'FieldDisplay';
  group.userData.display = display;

  const disposables = [];
  const trackDisposable = (resource) => {
    if (resource?.dispose) {
      disposables.push(resource);
    }
  };

  const corners = Array.isArray(display.section?.corners)
    ? display.section.corners.filter((corner) => corner?.isVector3)
    : [];

  if (corners.length >= 4) {
    const vertices = new Float32Array([
      corners[0].x, corners[0].y, corners[0].z,
      corners[1].x, corners[1].y, corners[1].z,
      corners[2].x, corners[2].y, corners[2].z,
      corners[0].x, corners[0].y, corners[0].z,
      corners[2].x, corners[2].y, corners[2].z,
      corners[3].x, corners[3].y, corners[3].z,
    ]);

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    geometry.computeVertexNormals();

    const material = new THREE.MeshBasicMaterial({
      color: 0x1f2933,
      transparent: true,
      opacity: 0.12,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    const plane = new THREE.Mesh(geometry, material);
    group.add(plane);
    trackDisposable(geometry);
    trackDisposable(material);
  }

  const entries = Array.isArray(display.entries)
    ? display.entries.filter((entry) => entry?.point?.isVector3)
    : [];

  if (entries.length) {
    if (display.mode === 'scalar') {
      const magnitudes = entries
        .map((entry) => getEntryMagnitude(entry, null))
        .filter((value) => Number.isFinite(value));
      const minMagnitude = magnitudes.length ? Math.min(...magnitudes) : 0;
      const maxMagnitude = magnitudes.length ? Math.max(...magnitudes) : 0;
      const range = maxMagnitude > minMagnitude ? (maxMagnitude - minMagnitude) : Math.max(maxMagnitude, 1);

      const pointCloud = createPointsObject(entries, (entry) => {
        const magnitude = getEntryMagnitude(entry, minMagnitude);
        const normalized = range > FIELD_EPSILON
          ? THREE.MathUtils.clamp((magnitude - minMagnitude) / range, 0, 1)
          : 0.5;
        const colour = new THREE.Color();
        colour.setHSL(0.66 - 0.66 * normalized, 0.85, 0.55);
        return colour;
      });

      if (pointCloud) {
        group.add(pointCloud.object);
        pointCloud.disposables.forEach(trackDisposable);
      }
    } else if (display.mode === 'perpendicular') {
      const pointCloud = createPointsObject(entries, (entry) => ensureColor(entry.color, new THREE.Color(0xffffff)));
      if (pointCloud) {
        group.add(pointCloud.object);
        pointCloud.disposables.forEach(trackDisposable);
      }
    } else if (display.mode === 'tensor') {
      const tensorSegments = buildTensorSegments(entries);
      const tensorObject = createSegmentsObject(tensorSegments);
      if (tensorObject) {
        group.add(tensorObject.object);
        tensorObject.disposables.forEach(trackDisposable);
      }
    } else {
      const directionSegments = buildDirectionSegments(entries);
      const directionObject = createSegmentsObject(directionSegments);
      if (directionObject) {
        group.add(directionObject.object);
        directionObject.disposables.forEach(trackDisposable);
      }
    }
  }

  group.userData.dispose = () => {
    while (disposables.length) {
      const resource = disposables.pop();
      try {
        resource.dispose?.();
      } catch (error) {
        console.warn('Field display dispose error', error);
      }
    }
  };

  return group;
}

export function initScene(canvas) {
  const scene = new THREE.Scene();
  scene.background = null;

  const camera = new THREE.PerspectiveCamera(50, 1, 0.1, MAX_DRAW_DISTANCE_MM);
  camera.up.set(0, 0, 1);
  camera.position.set(6, 4, 8);

  const clock = new THREE.Clock();

  let viewportState = getViewportSize(canvas);

  let webgpuRenderer = null;
  let webglRenderer = null;
  let webgpuInitPromise = null;
  let webglInitPromise = null;
  let rendererInitPromise = null;
  let webgpuFailed = false;
  let lastWebGPUError = null;
  let rendererInfo = { type: 'none', error: null };

  const webgpuSupported = typeof navigator !== 'undefined'
    && 'gpu' in navigator
    && (typeof THREE.WebGPURenderer.isAvailable !== 'function' || THREE.WebGPURenderer.isAvailable());

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.screenSpacePanning = false;

  const raycaster = new THREE.Raycaster();
  const pointerNdc = new THREE.Vector2();

  const eventTarget = canvas;

  const sunSky = new PhysicalSunSky(scene);
  sunSky.setCamera(camera);
  const ddgiVolume = ENABLE_DDGI
    ? new DDGIProbeVolume(scene, sunSky, {
        probeSpacing: 4000,
        updateBudget: 128,
        hysteresis: 0.96,
        boundsPadding: 800,
        maxDistance: 9000,
      })
    : null;
  addHelpers(scene);

  function updateRendererViewports() {
    if (webgpuRenderer) {
      applyViewportToRenderer(webgpuRenderer, viewportState);
    }
    if (webglRenderer) {
      applyViewportToRenderer(webglRenderer, viewportState);
    }
  }

  const resize = () => {
    viewportState = getViewportSize(canvas);
    const width = viewportState.width;
    const height = viewportState.height;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    updateRendererViewports();
    sunSky.notifyCameraProjectionChanged(camera);
  };
  resize();
  window.addEventListener('resize', resize);

  let currentObject = null;
  let needsFit = true;
  let overlayEnabled = false;
  let currentOverlayGroup = null;
  let latestOverlayData = { segments: [], points: [] };

  function sanitizeOverlayData(raw) {
    const safe = { segments: [], points: [] };
    if (!raw || typeof raw !== 'object') {
      return safe;
    }

    if (Array.isArray(raw.segments)) {
      raw.segments.forEach((segment) => {
        const start = segment?.start;
        const end = segment?.end;
        if (start?.isVector3 && end?.isVector3) {
          safe.segments.push({ start: start.clone(), end: end.clone() });
        }
      });
    }

    if (Array.isArray(raw.points)) {
      raw.points.forEach((point) => {
        if (point?.isVector3) {
          safe.points.push(point.clone());
        }
      });
    }

    return safe;
  }

  function rebuildOverlayGroup() {
    if (currentOverlayGroup) {
      scene.remove(currentOverlayGroup);
      disposeSceneObject(currentOverlayGroup);
      currentOverlayGroup = null;
    }

    if (!overlayEnabled) {
      return;
    }

    const segments = latestOverlayData.segments.map((segment) => ({
      start: segment.start,
      end: segment.end,
      colorStart: OVERLAY_LINE_COLOR,
      colorEnd: OVERLAY_LINE_COLOR,
    }));
    const segmentObject = createSegmentsObject(segments);

    const pointEntries = latestOverlayData.points.map((point) => ({ point }));
    const pointObject = createPointsObject(pointEntries, () => OVERLAY_POINT_COLOR);

    if (!segmentObject && !pointObject) {
      return;
    }

    const group = new THREE.Group();
    group.name = 'GHXCurveOverlay';

    if (segmentObject) {
      group.add(segmentObject.object);
    }
    if (pointObject) {
      group.add(pointObject.object);
    }

    currentOverlayGroup = group;
    scene.add(currentOverlayGroup);
  }

  function setOverlayData(raw) {
    latestOverlayData = sanitizeOverlayData(raw);
    rebuildOverlayGroup();
  }

  function setOverlayEnabled(value) {
    overlayEnabled = Boolean(value);
    rebuildOverlayGroup();

    if (!currentObject && overlayEnabled && currentOverlayGroup) {
      const sphere = computeWorldBoundingSphere(currentOverlayGroup);
      if (sphere) {
        fitCameraToSphere(sphere);
        needsFit = false;
      }
    }

    if (!currentObject && !currentOverlayGroup) {
      needsFit = true;
    }
  }

  function computeWorldBoundingSphere(object) {
    if (!object) {
      return null;
    }

    if (object.geometry) {
      const geometry = object.geometry;
      if (typeof geometry.computeBoundingSphere === 'function') {
        geometry.computeBoundingSphere();
      }
      if (geometry.boundingSphere) {
        const sphere = geometry.boundingSphere.clone();
        if (typeof object.updateWorldMatrix === 'function') {
          object.updateWorldMatrix(true, false);
        } else if (object.updateMatrixWorld) {
          object.updateMatrixWorld(true);
        }
        sphere.center.applyMatrix4(object.matrixWorld);

        const scale = new THREE.Vector3();
        const position = new THREE.Vector3();
        const quaternion = new THREE.Quaternion();
        object.matrixWorld.decompose(position, quaternion, scale);
        const maxScale = Math.max(scale.x, scale.y, scale.z);
        if (Number.isFinite(maxScale) && maxScale > 0) {
          sphere.radius *= maxScale;
        }
        return sphere;
      }
    }

    if (object.isObject3D) {
      if (typeof object.updateWorldMatrix === 'function') {
        object.updateWorldMatrix(true, true);
      } else if (object.updateMatrixWorld) {
        object.updateMatrixWorld(true);
      }
      const box = new THREE.Box3().setFromObject(object);
      if (box.isEmpty()) {
        return null;
      }
      const sphere = new THREE.Sphere();
      box.getBoundingSphere(sphere);
      if (!Number.isFinite(sphere.radius) || sphere.radius <= 0) {
        const size = new THREE.Vector3();
        box.getSize(size);
        const fallbackRadius = size.length() / 2;
        sphere.radius = Number.isFinite(fallbackRadius) && fallbackRadius > 0 ? fallbackRadius : 1;
      }
      return sphere;
    }

    return null;
  }

  function fitCameraToSphere(sphere) {
    if (!sphere) {
      return;
    }

    const center = sphere.center.clone();
    const radius = Math.max(sphere.radius, 0.0001);

    const previousTarget = controls.target.clone();
    let direction = camera.position.clone().sub(previousTarget);
    if (!Number.isFinite(direction.lengthSq()) || direction.lengthSq() < 1e-6) {
      direction.set(0, 0, 1);
    }
    direction.normalize();

    const aspect = camera.aspect || 1;
    const halfVerticalFov = THREE.MathUtils.degToRad(camera.fov) / 2;
    const safeHalfVertical = halfVerticalFov > 1e-4 ? halfVerticalFov : 1e-4;
    const halfHorizontalFov = Math.atan(Math.tan(safeHalfVertical) * aspect);
    const safeHalfHorizontal = halfHorizontalFov > 1e-4 ? halfHorizontalFov : 1e-4;

    const verticalDistance = radius / Math.tan(safeHalfVertical);
    const horizontalDistance = radius / Math.tan(safeHalfHorizontal);
    const distance = Math.max(verticalDistance, horizontalDistance, radius * 1.5, 1);

    const newPosition = center.clone().add(direction.multiplyScalar(distance));

    controls.target.copy(center);
    camera.position.copy(newPosition);
    camera.near = Math.max(distance / 100, 0.01);
    camera.far = Math.max(distance * 4, distance + radius * 4, MAX_DRAW_DISTANCE_MM);
    camera.updateProjectionMatrix();
    sunSky.notifyCameraProjectionChanged(camera);
    controls.update();
  }

  function updateOrbitTarget(targetPoint) {
    if (!targetPoint?.isVector3) {
      return;
    }

    controls.target.copy(targetPoint);

    controls.update();
  }

  function updatePointerFromEvent(event) {
    const bounds = eventTarget.getBoundingClientRect();
    const width = bounds.width;
    const height = bounds.height;
    if (!width || !height) {
      return false;
    }

    pointerNdc.x = ((event.clientX - bounds.left) / width) * 2 - 1;
    pointerNdc.y = -((event.clientY - bounds.top) / height) * 2 + 1;
    return true;
  }

  function findPointerIntersection(event) {
    if (!updatePointerFromEvent(event)) {
      return null;
    }

    raycaster.setFromCamera(pointerNdc, camera);

    if (!currentObject) {
      return null;
    }

    const intersections = raycaster.intersectObject(currentObject, true);
    if (!intersections.length) {
      return null;
    }

    return intersections[0].point.clone();
  }

  function isOrbitMouseButton(event) {
    if (!event) {
      return false;
    }

    if (!controls.enableRotate) {
      return false;
    }

    if (event.pointerType === 'touch') {
      return true;
    }

    if (event.shiftKey) {
      return false;
    }

    const mapping = controls.mouseButtons ?? {};
    const buttonMap = {
      0: mapping.LEFT,
      1: mapping.MIDDLE,
      2: mapping.RIGHT,
    };
    const action = buttonMap[event.button];
    return action === THREE.MOUSE.ROTATE;
  }

  const ORBIT_CLICK_DISTANCE_SQ = 4 * 4; // squared distance threshold (~4px)
  let pendingOrbitTarget = null;

  function resetPendingOrbitTarget() {
    pendingOrbitTarget = null;
  }

  function handlePointerDown(event) {
    if (!isOrbitMouseButton(event)) {
      resetPendingOrbitTarget();
      return;
    }

    const intersection = findPointerIntersection(event);
    const targetPoint = intersection ?? null;

    pendingOrbitTarget = {
      pointerId: event.pointerId,
      clientX: event.clientX,
      clientY: event.clientY,
      moved: false,
      targetPoint,
    };
  }

  function handlePointerMove(event) {
    if (!pendingOrbitTarget || event.pointerId !== pendingOrbitTarget.pointerId) {
      return;
    }

    const dx = event.clientX - pendingOrbitTarget.clientX;
    const dy = event.clientY - pendingOrbitTarget.clientY;
    if (dx * dx + dy * dy > ORBIT_CLICK_DISTANCE_SQ) {
      pendingOrbitTarget.moved = true;
    }
  }

  function handlePointerUp(event) {
    if (!pendingOrbitTarget || event.pointerId !== pendingOrbitTarget.pointerId) {
      return;
    }

    const info = pendingOrbitTarget;
    resetPendingOrbitTarget();

    if (info.moved) {
      return;
    }

    if (!isOrbitMouseButton(event)) {
      return;
    }

    updateOrbitTarget(info.targetPoint);
  }

  function handlePointerCancel(event) {
    if (pendingOrbitTarget && event.pointerId === pendingOrbitTarget.pointerId) {
      resetPendingOrbitTarget();
    }
  }

  eventTarget.addEventListener('pointerdown', handlePointerDown);
  eventTarget.addEventListener('pointermove', handlePointerMove);
  eventTarget.addEventListener('pointerup', handlePointerUp);
  eventTarget.addEventListener('pointerleave', handlePointerCancel);
  eventTarget.addEventListener('pointercancel', handlePointerCancel);

  function setRendererInfo(type, error = null) {
    rendererInfo = { type, error: error ?? null };
  }

  async function ensureWebGLRenderer() {
    if (webglRenderer) {
      return webglRenderer;
    }

    if (!webglInitPromise) {
      webglInitPromise = createWebGLRenderer(canvas, viewportState)
        .then((renderer) => {
          webglRenderer = renderer;
          sunSky.setRenderer(renderer);
          updateRendererViewports();
          if (webgpuSupported && (webgpuFailed || lastWebGPUError)) {
            setRendererInfo('webgl-fallback', lastWebGPUError);
          } else {
            setRendererInfo('webgl');
          }
          return renderer;
        })
        .catch((error) => {
          webglInitPromise = null;
          throw error;
        });
    }

    return webglInitPromise;
  }

  async function ensureWebGPURenderer() {
    if (!webgpuSupported || webgpuFailed) {
      throw new Error('WebGPU wordt niet ondersteund in deze omgeving.');
    }

    if (webgpuRenderer) {
      return webgpuRenderer;
    }

    if (!webgpuInitPromise) {
      webgpuInitPromise = createWebGPURenderer(canvas, viewportState)
        .then((renderer) => {
          webgpuRenderer = renderer;
          sunSky.setRenderer(renderer);
          updateRendererViewports();
          setRendererInfo('webgpu');
          return renderer;
        })
        .catch((error) => {
          webgpuInitPromise = null;
          webgpuRenderer = null;
          throw error;
        });
    }

    return webgpuInitPromise;
  }

  async function ensureRenderer() {
    if (webgpuRenderer) {
      return webgpuRenderer;
    }
    if (webglRenderer) {
      return webglRenderer;
    }
    if (!rendererInitPromise) {
      rendererInitPromise = (async () => {
        if (webgpuSupported && !webgpuFailed) {
          try {
            return await ensureWebGPURenderer();
          } catch (error) {
            webgpuFailed = true;
            lastWebGPUError = error;
            console.warn('WebGPU initialisatie mislukt', error);
          }
        }
        return ensureWebGLRenderer();
      })()
        .catch((error) => {
          rendererInitPromise = null;
          throw error;
        })
        .finally(() => {
          rendererInitPromise = null;
        });
    }
    return rendererInitPromise;
  }

  function isGpuRenderingEnabled() {
    return Boolean(webgpuRenderer || webglRenderer);
  }

  function isWebGPUSupported() {
    return webgpuSupported && !webgpuFailed;
  }

  function whenRendererReady() {
    return ensureRenderer();
  }

  whenRendererReady().catch((error) => {
    console.warn('Renderer initialisatie mislukt', error);
  });

  function disposeSceneObject(object) {
    if (!object) {
      return;
    }
    if (typeof object.userData?.dispose === 'function') {
      object.userData.dispose();
      return;
    }
    const disposeMaterial = (material) => {
      if (!material) {
        return;
      }
      if (Array.isArray(material)) {
        material.forEach(disposeMaterial);
        return;
      }
      material.dispose?.();
    };
    if (object.isMesh || object.isLine || object.isLineSegments || object.isPoints) {
      object.geometry?.dispose?.();
      disposeMaterial(object.material);
      return;
    }
    if (object.isObject3D) {
      object.traverse((child) => {
        if (child.isMesh || child.isLine || child.isLineSegments || child.isPoints) {
          child.geometry?.dispose?.();
          disposeMaterial(child.material);
        }
      });
    }
  }

  function applyShadowDefaults(object) {
    if (!object?.isObject3D) {
      return;
    }
    object.traverse((child) => {
      if (child.isMesh) {
        const preparedMaterial = convertMaterialToNode(child.material, { side: DEFAULT_MESH_SIDE });
        if (preparedMaterial) {
          child.material = preparedMaterial;
        }
        applyMaterialSide(child.material, DEFAULT_MESH_SIDE);
        ensureGeometryHasVertexNormals(child.geometry);
      }
      if (child.isMesh || child.isLine || child.isLineSegments || child.isPoints) {
        child.castShadow = true;
        child.receiveShadow = true;
      }
    });
  }

  function buildSceneObject(renderable) {
    if (!renderable) {
      return null;
    }

    if (Array.isArray(renderable)) {
      const group = new THREE.Group();
      group.name = 'GHXRenderableGroup';
      for (const entry of renderable) {
        const child = buildSceneObject(entry);
        if (child) {
          group.add(child);
        }
      }
      return group.children.length ? group : null;
    }

    if (renderable.type === 'field-display') {
      const group = createFieldDisplayGroup(renderable);
      if (!group) {
        console.warn('updateMesh: kon veldweergave niet maken', renderable);
      }
      return group;
    }

    if (renderable.isObject3D) {
      return renderable;
    }

    if (renderable.isBufferGeometry || renderable.isGeometry) {
      const geometry = renderable.clone ? renderable.clone() : renderable;
      ensureGeometryHasVertexNormals(geometry);
      const material = createStandardSurfaceMaterial(
        {
          color: 0x2c9cf5,
          metalness: 0.1,
          roughness: 0.65,
        },
        { side: DEFAULT_MESH_SIDE },
      );
      const mesh = new THREE.Mesh(geometry, material);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      return mesh;
    }

    if (renderable.geometry) {
      const geometry = renderable.geometry.clone ? renderable.geometry.clone() : renderable.geometry;
      ensureGeometryHasVertexNormals(geometry);
      const baseMaterial = cloneSurfaceMaterial(renderable.material);
      const material = convertMaterialToNode(baseMaterial, { side: DEFAULT_MESH_SIDE })
        ?? createStandardSurfaceMaterial({ color: 0x2c9cf5 }, { side: DEFAULT_MESH_SIDE });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      return mesh;
    }

    return null;
  }

  function updateMesh(payload) {
    const isDisplayPayload = payload && typeof payload === 'object' && payload.type === 'ghx-display';
    const geometryOrMesh = isDisplayPayload ? payload.main ?? null : payload;
    const overlayData = isDisplayPayload ? payload.overlays ?? null : null;

    if (geometryOrMesh && geometryOrMesh === currentObject) {
      setOverlayData(overlayData);
      const fitTarget = currentObject ?? (overlayEnabled ? currentOverlayGroup : null);
      const sphere = computeWorldBoundingSphere(fitTarget);
      if (sphere && needsFit) {
        fitCameraToSphere(sphere);
        needsFit = false;
      }
      ddgiVolume?.setSceneRoot(currentObject);
      return;
    }

    if (currentObject) {
      scene.remove(currentObject);
      disposeSceneObject(currentObject);
      currentObject = null;
      ddgiVolume?.setSceneRoot(null);
    }

    if (!geometryOrMesh) {
      setOverlayData(overlayData);
      const fitTarget = overlayEnabled ? currentOverlayGroup : null;
      const sphere = computeWorldBoundingSphere(fitTarget);
      if (sphere) {
        if (needsFit) {
          fitCameraToSphere(sphere);
          needsFit = false;
        }
      } else {
        needsFit = true;
      }
      ddgiVolume?.setSceneRoot(null);
      return;
    }

    const nextObject = buildSceneObject(geometryOrMesh);

    if (!nextObject) {
      if (geometryOrMesh) {
        console.warn('updateMesh: onbekend objecttype', geometryOrMesh);
      }
      setOverlayData(overlayData);
      needsFit = true;
      return;
    }

    applyShadowDefaults(nextObject);

    if (ENABLE_DOUBLE_SIDED_MESHES) {
      applyMeshSide(nextObject, THREE.DoubleSide);
    }

    currentObject = nextObject;
    scene.add(currentObject);
    ddgiVolume?.setSceneRoot(currentObject);

    setOverlayData(overlayData);

    const fitTarget = currentObject ?? (overlayEnabled ? currentOverlayGroup : null);
    const sphere = computeWorldBoundingSphere(fitTarget);
    if (sphere) {
      if (needsFit) {
        fitCameraToSphere(sphere);
        needsFit = false;
      }
    } else if (needsFit) {
      controls.target.set(0, 0, 0);
      controls.update();
      needsFit = false;
    }
  }

  function animate() {
    requestAnimationFrame(animate);
    const deltaTime = clock.getDelta();
    const elapsed = clock.elapsedTime;
    controls.update();
    sunSky.updateFrame(camera);
    ddgiVolume?.update(deltaTime, scene, elapsed, camera.position);
    const activeRenderer = webgpuRenderer ?? webglRenderer;
    if (activeRenderer) {
      try {
        activeRenderer.render(scene, camera);
      } catch (error) {
        if (activeRenderer === webgpuRenderer) {
          console.warn('WebGPU renderfout, val terug op WebGL', error);
          try {
            activeRenderer.dispose?.();
          } catch (disposeError) {
            console.warn('Kon WebGPU renderer niet opruimen', disposeError);
          }
          webgpuRenderer = null;
          webgpuFailed = true;
          lastWebGPUError = error;
          ensureRenderer().catch((initError) => {
            console.warn('Kon fallback-renderer niet initialiseren', initError);
          });
        } else {
          console.error('Rendererfout', error);
        }
      }
    }
  }
  animate();

  const api = {
    scene,
    camera,
    controls,
    sunSky,
    ddgiVolume,
    updateMesh,
    setOverlayEnabled,
    isGpuRenderingEnabled,
    isWebGPUSupported,
    whenRendererReady,
    getRendererInfo: () => ({ ...rendererInfo }),
  };

  Object.defineProperty(api, 'renderer', {
    get() {
      return webgpuRenderer ?? webglRenderer;
    },
  });

  return api;
}

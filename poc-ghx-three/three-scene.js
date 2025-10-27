import * as THREE from 'three/webgpu';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
// import { WebGPURenderer } from 'three/addons/renderers/webgpu/WebGPURenderer.js';

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

// Toggle to enable or disable double sided rendering for viewport meshes.
const ENABLE_DOUBLE_SIDED_MESHES = true;
const DEFAULT_MESH_SIDE = ENABLE_DOUBLE_SIDED_MESHES ? THREE.DoubleSide : THREE.FrontSide;

const AXES_LENGTH_MM = 5000;
const MAX_DRAW_DISTANCE_MM = 100000;
const SKY_DOME_RADIUS = MAX_DRAW_DISTANCE_MM * 0.95;
const TEMP_CAMERA_DIRECTION = new THREE.Vector3();

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

  if (typeof renderer.setClearColor === 'function') {
    renderer.setClearColor(0x060910, 1);
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

function addDefaultLights(scene) {
  const ambient = new THREE.AmbientLight(0xffffff, 0.4);
  scene.add(ambient);

  const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
  dirLight.position.set(5, 8, 4);
  scene.add(dirLight);
}

function addHelpers(scene) {
  const axes = new THREE.AxesHelper(AXES_LENGTH_MM);
  scene.add(axes);
}

function createSkyDome(scene) {
  const uniforms = {
    topColor: { value: new THREE.Color(0x20a7db) },
    horizonColor: { value: new THREE.Color(0xcfecf7) },
    bottomColor: { value: new THREE.Color(0xe4eef2) },
    horizonOffset: { value: 0 },
    gradientExponent: { value: 1.25 },
  };

  const material = new THREE.ShaderMaterial({
    uniforms,
    vertexShader: `
      varying vec3 vWorldPosition;
      void main() {
        vec4 worldPosition = modelMatrix * vec4(position, 1.0);
        vWorldPosition = worldPosition.xyz;
        gl_Position = projectionMatrix * viewMatrix * worldPosition;
      }
    `,
    fragmentShader: `
      uniform vec3 topColor;
      uniform vec3 horizonColor;
      uniform vec3 bottomColor;
      uniform float horizonOffset;
      uniform float gradientExponent;
      varying vec3 vWorldPosition;

      void main() {
        vec3 direction = normalize(vWorldPosition);
        float base = clamp(direction.z * 0.5 + 0.5 + horizonOffset, 0.0, 1.0);
        float shaped = pow(base, gradientExponent);
        vec3 color = mix(bottomColor, horizonColor, smoothstep(0.0, 0.65, base));
        color = mix(color, topColor, smoothstep(0.25, 1.0, shaped));
        gl_FragColor = vec4(color, 1.0);
      }
    `,
    side: THREE.BackSide,
    depthWrite: false,
    depthTest: false,
    fog: false,
  });

  const geometry = new THREE.SphereGeometry(SKY_DOME_RADIUS, 32, 24);
  const skyMesh = new THREE.Mesh(geometry, material);
  skyMesh.name = 'SkyGradient';
  skyMesh.frustumCulled = false;
  skyMesh.renderOrder = -1;

  scene.add(skyMesh);

  return {
    update(camera) {
      skyMesh.position.copy(camera.position);
      camera.getWorldDirection(TEMP_CAMERA_DIRECTION);
      const desiredOffset = THREE.MathUtils.clamp(TEMP_CAMERA_DIRECTION.z * 0.35, -0.4, 0.4);
      uniforms.horizonOffset.value = THREE.MathUtils.lerp(uniforms.horizonOffset.value, desiredOffset, 0.08);
    },
  };
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
}

function applyMeshSide(object, side) {
  if (!object?.isObject3D) {
    return;
  }
  object.traverse((child) => {
    if (child.isMesh) {
      applyMaterialSide(child.material, side);
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

  const positions = [];
  const colours = [];

  segments.forEach((segment) => {
    const start = segment.start;
    const end = segment.end;
    if (!start?.isVector3 || !end?.isVector3) {
      return;
    }
    const startColour = ensureColor(segment.colorStart ?? segment.color ?? 0xffffff);
    const endColour = ensureColor(segment.colorEnd ?? segment.color ?? startColour);
    positions.push(start.x, start.y, start.z, end.x, end.y, end.z);
    colours.push(startColour.r, startColour.g, startColour.b, endColour.r, endColour.g, endColour.b);
  });

  if (!positions.length) {
    return null;
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('color', new THREE.Float32BufferAttribute(colours, 3));

  const material = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.95,
    depthWrite: false,
  });

  const object = new THREE.LineSegments(geometry, material);
  return { object, disposables: [geometry, material] };
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

  const material = new THREE.MeshStandardMaterial({
    color: 0xffffff,
    vertexColors: true,
    metalness: 0,
    roughness: 0.55,
  });

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

  let viewportState = getViewportSize(canvas);

  let webgpuRenderer = null;

  const webgpuSupported = typeof navigator !== 'undefined'
    && 'gpu' in navigator
    && (typeof WebGPURenderer.isAvailable !== 'function' || WebGPURenderer.isAvailable());

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.screenSpacePanning = false;

  const raycaster = new THREE.Raycaster();
  const pointerNdc = new THREE.Vector2();

  const eventTarget = canvas;

  const sky = createSkyDome(scene);
  addDefaultLights(scene);
  addHelpers(scene);

  function updateRendererViewports() {
    if (webgpuRenderer) {
      applyViewportToRenderer(webgpuRenderer, viewportState);
    }
  }

  const resize = () => {
    viewportState = getViewportSize(canvas);
    const width = viewportState.width;
    const height = viewportState.height;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    updateRendererViewports();
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

  let webgpuInitPromise = null;

  async function ensureWebGPURenderer() {
    if (!webgpuSupported) {
      throw new Error('WebGPU wordt niet ondersteund in deze omgeving.');
    }

    if (webgpuRenderer) {
      return webgpuRenderer;
    }

    if (!webgpuInitPromise) {
      webgpuInitPromise = createWebGPURenderer(canvas, viewportState)
        .then((renderer) => {
          webgpuRenderer = renderer;
          updateRendererViewports();
          return webgpuRenderer;
        })
        .catch((error) => {
          webgpuInitPromise = null;
          webgpuRenderer = null;
          throw error;
        });
    }

    return webgpuInitPromise;
  }

  function isGpuRenderingEnabled() {
    return Boolean(webgpuRenderer);
  }

  function isWebGPUSupported() {
    return webgpuSupported;
  }

  function whenRendererReady() {
    return ensureWebGPURenderer();
  }

  if (webgpuSupported) {
    whenRendererReady().catch((error) => {
      console.warn('WebGPU initialisatie mislukt', error);
    });
  }

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
      const material = new THREE.MeshStandardMaterial({
        color: 0x2c9cf5,
        metalness: 0.1,
        roughness: 0.65,
        side: DEFAULT_MESH_SIDE,
      });
      const mesh = new THREE.Mesh(renderable, material);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      return mesh;
    }

    if (renderable.geometry) {
      const material = renderable.material || new THREE.MeshStandardMaterial({
        color: 0x2c9cf5,
        side: DEFAULT_MESH_SIDE,
      });
      const mesh = new THREE.Mesh(renderable.geometry, material);
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
      return;
    }

    if (currentObject) {
      scene.remove(currentObject);
      disposeSceneObject(currentObject);
      currentObject = null;
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
    controls.update();
    sky.update(camera);
    if (webgpuRenderer) {
      webgpuRenderer.render(scene, camera);
    }
  }
  animate();

  const api = {
    scene,
    camera,
    controls,
    updateMesh,
    setOverlayEnabled,
    isGpuRenderingEnabled,
    isWebGPUSupported,
    whenRendererReady,
  };

  Object.defineProperty(api, 'renderer', {
    get() {
      return webgpuRenderer;
    },
  });

  return api;
}

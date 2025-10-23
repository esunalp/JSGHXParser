import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const GRID_CELL_SIZE_MM = 1000;
const GRID_DIVISIONS = 20;
const GRID_SIZE_MM = GRID_CELL_SIZE_MM * GRID_DIVISIONS;
const AXES_LENGTH_MM = GRID_CELL_SIZE_MM * 5;

function createRenderer(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.setSize(canvas.clientWidth || window.innerWidth, canvas.clientHeight || window.innerHeight, false);
  renderer.outputEncoding = THREE.sRGBEncoding;
  renderer.setClearColor(0x111111, 1);
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
  const grid = new THREE.GridHelper(GRID_SIZE_MM, GRID_DIVISIONS, 0x888888, 0x444444);
  grid.position.y = -0.5 * GRID_CELL_SIZE_MM;
  scene.add(grid);

  const axes = new THREE.AxesHelper(AXES_LENGTH_MM);
  scene.add(axes);
}

const FIELD_EPSILON = 1e-6;
const FIELD_AXIS_COLOURS = [0xd1495b, 0x3066be, 0x2fbf71];

function ensureColor(value, fallback = new THREE.Color(0xffffff)) {
  if (value?.isColor) {
    return value.clone();
  }
  if (typeof value === 'number') {
    return new THREE.Color(value);
  }
  if (typeof value === 'string') {
    try {
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

  const positions = new Float32Array(entries.length * 3);
  const colours = new Float32Array(entries.length * 3);

  entries.forEach((entry, index) => {
    const point = entry?.point;
    if (!point?.isVector3) {
      return;
    }
    const colour = ensureColor(colourFactory(entry, index), new THREE.Color(0xffffff));
    const offset = index * 3;
    positions[offset + 0] = point.x;
    positions[offset + 1] = point.y;
    positions[offset + 2] = point.z;
    colours[offset + 0] = colour.r;
    colours[offset + 1] = colour.g;
    colours[offset + 2] = colour.b;
  });

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('color', new THREE.Float32BufferAttribute(colours, 3));

  const material = new THREE.PointsMaterial({
    size: 0.12,
    sizeAttenuation: true,
    vertexColors: true,
    transparent: true,
    opacity: 0.95,
    depthWrite: false,
  });

  const object = new THREE.Points(geometry, material);
  return { object, disposables: [geometry, material] };
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
  scene.background = new THREE.Color(0x050505);

  const camera = new THREE.PerspectiveCamera(50, 1, 0.1, 1000);
  camera.position.set(6, 4, 8);

  const renderer = createRenderer(canvas);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;

  addDefaultLights(scene);
  addHelpers(scene);

  const resize = () => {
    const width = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth;
    const height = canvas.clientHeight || canvas.parentElement?.clientHeight || window.innerHeight;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };
  resize();
  window.addEventListener('resize', resize);

  let currentObject = null;
  let needsFit = true;

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
    camera.far = Math.max(distance * 4, distance + radius * 4);
    camera.updateProjectionMatrix();
    controls.update();
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

  function updateMesh(geometryOrMesh) {
    if (geometryOrMesh && geometryOrMesh === currentObject) {
      const sphere = computeWorldBoundingSphere(currentObject);
      if (sphere) {
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
      needsFit = true;
      return;
    }

    let nextObject = null;
    if (geometryOrMesh.type === 'field-display') {
      nextObject = createFieldDisplayGroup(geometryOrMesh);
      if (!nextObject) {
        console.warn('updateMesh: kon veldweergave niet maken', geometryOrMesh);
        needsFit = true;
        return;
      }
    } else if (geometryOrMesh.isObject3D) {
      nextObject = geometryOrMesh;
    } else if (geometryOrMesh.isBufferGeometry || geometryOrMesh.isGeometry) {
      const material = new THREE.MeshStandardMaterial({ color: 0x2c9cf5, metalness: 0.1, roughness: 0.65 });
      nextObject = new THREE.Mesh(geometryOrMesh, material);
    } else if (geometryOrMesh.geometry) {
      const material = geometryOrMesh.material || new THREE.MeshStandardMaterial({ color: 0x2c9cf5 });
      nextObject = new THREE.Mesh(geometryOrMesh.geometry, material);
    } else {
      console.warn('updateMesh: onbekend objecttype', geometryOrMesh);
      needsFit = true;
      return;
    }

    if (!nextObject) {
      needsFit = true;
      return;
    }

    if (nextObject.isMesh || nextObject.isLine || nextObject.isLineSegments || nextObject.isPoints) {
      nextObject.castShadow = true;
      nextObject.receiveShadow = true;
    }

    currentObject = nextObject;
    scene.add(currentObject);

    const sphere = computeWorldBoundingSphere(currentObject);
    if (sphere) {
      fitCameraToSphere(sphere);
      needsFit = false;
    } else if (needsFit) {
      controls.target.set(0, 0, 0);
      controls.update();
      needsFit = false;
    }
  }

  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();

  return { scene, camera, renderer, controls, updateMesh };
}

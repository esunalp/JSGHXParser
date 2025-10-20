//import * as THREE from './vendor/three.module.js?version=3';
//import { OrbitControls } from './vendor/OrbitControls.js?version=3';

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
  const grid = new THREE.GridHelper(20, 20, 0x888888, 0x444444);
  grid.position.y = -0.5;
  scene.add(grid);

  const axes = new THREE.AxesHelper(5);
  scene.add(axes);
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

  let currentMesh = null;

  function updateMesh(geometryOrMesh) {
    if (currentMesh) {
      scene.remove(currentMesh);
      currentMesh.geometry?.dispose?.();
      currentMesh.material?.dispose?.();
      currentMesh = null;
    }

    if (!geometryOrMesh) {
      return;
    }

    if (geometryOrMesh.isMesh) {
      currentMesh = geometryOrMesh;
    } else if (geometryOrMesh.isBufferGeometry || geometryOrMesh.isGeometry) {
      const material = new THREE.MeshStandardMaterial({ color: 0x2c9cf5, metalness: 0.1, roughness: 0.65 });
      currentMesh = new THREE.Mesh(geometryOrMesh, material);
    } else if (geometryOrMesh.geometry) {
      const material = geometryOrMesh.material || new THREE.MeshStandardMaterial({ color: 0x2c9cf5 });
      currentMesh = new THREE.Mesh(geometryOrMesh.geometry, material);
    } else {
      console.warn('updateMesh: onbekend objecttype', geometryOrMesh);
      return;
    }

    currentMesh.castShadow = true;
    currentMesh.receiveShadow = true;
    scene.add(currentMesh);
  }

  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();

  return { scene, camera, renderer, controls, updateMesh };
}

import { setupUi } from './ui.js';
import { createThreeApp } from './three_integration.js';

function normalizeGeometryItems(value) {
  if (!value || typeof value !== 'object') {
    return [];
  }
  const items = value.items;
  return Array.isArray(items) ? items : [];
}

function toNumericOrNull(value) {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : null;
}

function normalizeSliders(value) {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .filter((entry) => entry && typeof entry === 'object')
    .map((entry) => ({
      id: String(entry.id ?? ''),
      name:
        entry.name !== undefined && entry.name !== null
          ? String(entry.name)
          : entry.id !== undefined && entry.id !== null
            ? String(entry.id)
            : 'Slider',
      min: toNumericOrNull(entry.min),
      max: toNumericOrNull(entry.max),
      step: toNumericOrNull(entry.step),
      value: toNumericOrNull(entry.value) ?? 0,
    }));
}

async function init() {
  const ui = setupUi();
  ui.setStatus('Initialiseren van WebAssembly en Three.js…');

  const three = createThreeApp(ui.canvas);
  try {
    await three.ready;
  } catch (error) {
    console.warn('Three.js kon niet worden geïnitialiseerd:', error);
    ui.setStatus(error?.message ?? 'Three.js kon niet worden geïnitialiseerd.');
  }

  if (!three.isWebGPUSupported()) {
    ui.setStatus(
      'WebGPU wordt niet ondersteund in deze browser. De UI werkt, maar er wordt geen 3D-weergave getoond.'
    );
  }

  let wasmModule;
  try {
    wasmModule = await import('./pkg/ghx_engine.js');
  } catch (error) {
    console.error('Kon de WebAssembly module niet laden:', error);
    ui.setStatus(
      'Kon de WebAssembly module niet laden. Draai `wasm-pack build --target web` in `alpha/ghx-engine` en plaats de output in `alpha/web/pkg`.'
    );
    return;
  }

  const { default: initWasm, Engine } = wasmModule;

  try {
    await initWasm();
  } catch (error) {
    console.error('Fout bij initialisatie van de WebAssembly module:', error);
    ui.setStatus('Fout bij het initialiseren van de GHX-engine: ' + (error?.message ?? String(error)));
    return;
  }

  const engine = new Engine();

  function syncSliders({ replace = false } = {}) {
    let sliderData;
    try {
      sliderData = engine.get_sliders();
    } catch (error) {
      if (replace) {
        ui.renderSliders([]);
      }
      console.warn('Kon slidergegevens niet ophalen:', error);
      return [];
    }

    const sliders = normalizeSliders(sliderData);

    if (replace) {
      ui.renderSliders(sliders);
      return sliders;
    }

    let requiresRerender = false;
    for (const slider of sliders) {
      const updated = ui.updateSliderValue(slider.id, slider.value);
      if (!updated) {
        requiresRerender = true;
      }
    }

    if (requiresRerender) {
      ui.renderSliders(sliders);
    }

    return sliders;
  }

  function evaluateAndRender({ announce } = {}) {
    try {
      engine.evaluate();
    } catch (error) {
      console.error('Evaluatiefout:', error);
      three.updateGeometry([]);
      ui.setStatus('Evaluatie mislukt: ' + (error?.message ?? String(error)));
      return;
    }

    let geometry;
    try {
      geometry = engine.get_geometry();
    } catch (error) {
      console.error('Kon geometrie niet ophalen:', error);
      three.updateGeometry([]);
      ui.setStatus('Geometrie ophalen mislukt: ' + (error?.message ?? String(error)));
      return;
    }

    const items = normalizeGeometryItems(geometry);
    three.updateGeometry(items);

    if (announce) {
      ui.setStatus(announce);
    }
  }

  async function loadGhxFromText(contents, label) {
    if (typeof contents !== 'string' || !contents.trim()) {
      ui.setStatus('Het GHX-bestand is leeg of ongeldig.');
      return;
    }

    ui.showLoading(true);
    try {
      engine.load_ghx(contents);
      syncSliders({ replace: true });
      evaluateAndRender({ announce: label ? `GHX geladen (${label})` : 'GHX-bestand geladen.' });
    } catch (error) {
      console.error('Fout bij het laden van GHX:', error);
      ui.renderSliders([]);
      three.updateGeometry([]);
      ui.setStatus('Fout bij het laden van het GHX-bestand: ' + (error?.message ?? String(error)));
    } finally {
      syncSliders();
      ui.showLoading(false);
    }
  }

  async function loadDefaultSample() {
    const sampleName = 'minimal_line.ghx';
    try {
      const response = await fetch(`../tools/ghx-samples/${sampleName}`, { cache: 'no-store' });
      if (!response.ok) {
        throw new Error(`Kon ${sampleName} niet ophalen (status ${response.status}).`);
      }
      const text = await response.text();
      await loadGhxFromText(text, sampleName);
    } catch (error) {
      console.warn('Kon standaard GHX niet laden:', error);
      ui.setStatus('Selecteer een GHX-bestand om te starten.');
      ui.renderSliders([]);
      three.updateGeometry([]);
    }
  }

  async function handleFileSelection(file) {
    if (!file) {
      ui.setStatus('Geen bestand geselecteerd.');
      return;
    }

    try {
      const text = await file.text();
      await loadGhxFromText(text, file.name ?? 'gekozen bestand');
    } catch (error) {
      console.error('Fout bij lezen van bestand:', error);
      ui.setStatus('Kon het geselecteerde bestand niet lezen: ' + (error?.message ?? String(error)));
    }
  }

  function handleSliderChange(sliderId, value) {
    if (!sliderId) {
      return;
    }
    try {
      engine.set_slider_value(sliderId, value);
      syncSliders();
      evaluateAndRender();
    } catch (error) {
      console.error('Slider-update mislukt:', error);
      syncSliders({ replace: true });
      ui.setStatus('Kon slider niet aanpassen: ' + (error?.message ?? String(error)));
    }
  }

  function handleOverlayToggle(enabled) {
    three.setOverlayEnabled(enabled);
    ui.setOverlayState(enabled);
  }

  ui.setHandlers({
    onFileSelected: handleFileSelection,
    onSliderChange: handleSliderChange,
    onOverlayToggle: handleOverlayToggle,
  });

  ui.setOverlayState(true);
  three.setOverlayEnabled(true);

  await loadDefaultSample();
}

init().catch((error) => {
  console.error('Onherstelbare fout tijdens initialisatie:', error);
});

//# sourceMappingURL=main.js.map

const DEFAULT_SLIDER_MIN = 0;
const DEFAULT_SLIDER_MAX = 100;

function formatValue(value, step) {
  if (!Number.isFinite(value)) {
    return '';
  }
  const precision = step && !Number.isInteger(step) ? Math.min(6, Math.ceil(-Math.log10(step))) : 0;
  return Number.parseFloat(value).toFixed(Math.max(0, precision));
}

function createSliderElement(slider, handlers) {
  const wrapper = document.createElement('div');
  wrapper.className = 'slider';
  wrapper.dataset.sliderId = slider.id;

  const labelRow = document.createElement('div');
  labelRow.className = 'slider-label';

  const nameSpan = document.createElement('span');
  nameSpan.textContent = slider.name ?? slider.id ?? 'Slider';
  const valueSpan = document.createElement('span');
  labelRow.append(nameSpan, valueSpan);

  const inputsRow = document.createElement('div');
  inputsRow.className = 'slider-inputs';

  const rangeInput = document.createElement('input');
  rangeInput.type = 'range';

  const numberInput = document.createElement('input');
  numberInput.type = 'number';

  const hasMin = Number.isFinite(slider.min);
  const hasMax = Number.isFinite(slider.max);
  const hasStep = Number.isFinite(slider.step) && slider.step > 0;

  const lowerBound = hasMin ? slider.min : DEFAULT_SLIDER_MIN;
  let upperBound = hasMax ? slider.max : DEFAULT_SLIDER_MAX;
  if (upperBound < lowerBound) {
    upperBound = lowerBound;
  }

  const numericStep = hasStep ? slider.step : Math.max((upperBound - lowerBound) / 100, 0.0001);
  const formatStep = hasStep ? Math.abs(slider.step) : undefined;

  rangeInput.min = String(lowerBound);
  rangeInput.max = String(upperBound);
  rangeInput.step = String(numericStep);
  rangeInput.setAttribute('aria-label', `${nameSpan.textContent} (range)`);

  if (hasMin) {
    numberInput.min = String(lowerBound);
  }
  if (hasMax) {
    numberInput.max = String(upperBound);
  }
  numberInput.step = hasStep ? String(slider.step) : 'any';
  numberInput.setAttribute('aria-label', `${nameSpan.textContent} (nummer)`);

  const clamp = (value) => {
    let result = Number.isFinite(value) ? value : lowerBound;
    if (hasMin && result < lowerBound) {
      result = lowerBound;
    }
    if (hasMax && result > upperBound) {
      result = upperBound;
    }
    return result;
  };

  let committedValue = clamp(Number(slider.value));

  const applyValue = (value, { updateRange = true, updateNumber = true } = {}) => {
    committedValue = clamp(value);
    valueSpan.textContent = formatValue(committedValue, formatStep);
    if (updateRange) {
      rangeInput.value = String(committedValue);
    }
    if (updateNumber) {
      numberInput.value = String(committedValue);
    }
  };

  applyValue(committedValue, { updateRange: true, updateNumber: true });

  const emitChange = (value) => {
    if (typeof handlers.onSliderChange === 'function') {
      handlers.onSliderChange(slider.id, clamp(value));
    }
  };

  rangeInput.addEventListener('input', (event) => {
    const numeric = Number(event.target.value);
    if (Number.isNaN(numeric)) {
      return;
    }
    applyValue(numeric, { updateRange: false });
    emitChange(numeric);
  });

  const commitNumberValue = (rawValue) => {
    const numeric = Number(rawValue);
    if (Number.isNaN(numeric)) {
      applyValue(committedValue);
      return;
    }
    applyValue(numeric);
    emitChange(numeric);
  };

  numberInput.addEventListener('change', (event) => {
    commitNumberValue(event.target.value);
  });

  numberInput.addEventListener('blur', (event) => {
    commitNumberValue(event.target.value);
  });

  numberInput.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      commitNumberValue(event.target.value);
    }
  });

  numberInput.addEventListener('input', (event) => {
    const numeric = Number(event.target.value);
    if (Number.isNaN(numeric)) {
      valueSpan.textContent = '';
      return;
    }
    valueSpan.textContent = formatValue(numeric, formatStep);
  });

  inputsRow.append(rangeInput, numberInput);
  wrapper.append(labelRow, inputsRow);

  return {
    element: wrapper,
    controller: {
      update(value) {
        applyValue(value);
      },
    },
  };
}

export function setupUi() {
  const canvas = document.getElementById('viewport');
  const fileInput = document.getElementById('ghx-input');
  const statusOutput = document.getElementById('status');
  const sliderContainer = document.getElementById('slider-container');
  const overlayToggle = document.getElementById('overlay-toggle');
  const overlayState = document.getElementById('overlay-state');
  const loadingOverlay = document.getElementById('loading-overlay');

  const sliderElements = new Map();
  const handlers = {
    onFileSelected: null,
    onSliderChange: null,
    onOverlayToggle: null,
  };

  const setStatus = (text) => {
    if (statusOutput) {
      statusOutput.textContent = text;
    }
  };

  const showLoading = (active) => {
    if (!loadingOverlay) {
      return;
    }
    if (active) {
      loadingOverlay.classList.remove('loading-overlay--hidden');
      loadingOverlay.setAttribute('aria-hidden', 'false');
    } else {
      loadingOverlay.classList.add('loading-overlay--hidden');
      loadingOverlay.setAttribute('aria-hidden', 'true');
    }
  };

  const setOverlayState = (enabled) => {
    const isEnabled = Boolean(enabled);
    if (overlayToggle) {
      overlayToggle.checked = isEnabled;
      overlayToggle.setAttribute('aria-checked', isEnabled ? 'true' : 'false');
    }
    if (overlayState) {
      overlayState.textContent = isEnabled ? 'true' : 'false';
    }
  };

  const renderSliders = (sliders = []) => {
    sliderElements.clear();

    if (!sliderContainer) {
      return;
    }

    sliderContainer.innerHTML = '';

    if (!sliders.length) {
      const empty = document.createElement('p');
      empty.textContent = 'Geen sliders beschikbaar voor dit model.';
      empty.style.opacity = '0.65';
      empty.style.fontSize = '0.9rem';
      sliderContainer.appendChild(empty);
      return;
    }

    for (const slider of sliders) {
      if (!slider?.id) {
        continue;
      }
      const { element, controller } = createSliderElement(slider, handlers);
      sliderContainer.appendChild(element);
      sliderElements.set(String(slider.id), controller);
    }
  };

  const updateSliderValue = (sliderId, value) => {
    const controller = sliderElements.get(String(sliderId));
    if (!controller) {
      return false;
    }
    controller.update(value);
    return true;
  };

  const setHandlers = (newHandlers = {}) => {
    handlers.onFileSelected = typeof newHandlers.onFileSelected === 'function' ? newHandlers.onFileSelected : null;
    handlers.onSliderChange = typeof newHandlers.onSliderChange === 'function' ? newHandlers.onSliderChange : null;
    handlers.onOverlayToggle = typeof newHandlers.onOverlayToggle === 'function' ? newHandlers.onOverlayToggle : null;
  };

  if (fileInput) {
    fileInput.addEventListener('change', () => {
      const file = fileInput.files && fileInput.files[0] ? fileInput.files[0] : null;
      if (typeof handlers.onFileSelected === 'function') {
        handlers.onFileSelected(file);
      }
      // Laat dezelfde bestandsselectie opnieuw toe
      fileInput.value = '';
    });
  }

  if (overlayToggle) {
    overlayToggle.addEventListener('change', () => {
      const checked = overlayToggle.checked;
      setOverlayState(checked);
      if (typeof handlers.onOverlayToggle === 'function') {
        handlers.onOverlayToggle(checked);
      }
    });
  }

  return {
    canvas,
    setHandlers,
    renderSliders,
    updateSliderValue,
    setStatus,
    showLoading,
    setOverlayState,
  };
}

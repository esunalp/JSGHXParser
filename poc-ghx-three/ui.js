function formatValue(value, step) {
  const precision = step && !Number.isInteger(step) ? Math.min(6, Math.ceil(-Math.log10(step))) : 0;
  return Number.parseFloat(value ?? 0).toFixed(precision);
}

export function createSliderUI({ container, engine }) {
  if (!container) {
    throw new Error('Slider container ontbreekt');
  }

  function render() {
    const sliders = engine.listSliders();
    container.innerHTML = '';

    if (!sliders.length) {
      const empty = document.createElement('p');
      empty.textContent = 'Geen sliders beschikbaar voor deze graph.';
      empty.style.opacity = '0.65';
      empty.style.fontSize = '0.9rem';
      container.appendChild(empty);
      return;
    }

    for (const slider of sliders) {
      const wrapper = document.createElement('div');
      wrapper.className = 'slider';
      wrapper.dataset.nodeId = slider.id;

      const label = document.createElement('div');
      label.className = 'slider-label';

      const inputs = document.createElement('div');
      inputs.className = 'slider-inputs';

      const range = document.createElement('input');
      range.type = 'range';
      const toNumeric = (value, fallback) => {
        const parsed = Number(value);
        return Number.isNaN(parsed) ? fallback : parsed;
      };

      const defaultMin = 0;
      const defaultMax = 100;
      const hasMin = slider.min !== undefined && slider.min !== null && slider.min !== '';
      const hasMax = slider.max !== undefined && slider.max !== null && slider.max !== '';
      const rawMin = toNumeric(slider.min, defaultMin);
      const rawMax = toNumeric(slider.max, defaultMax);
      let lowerBound = hasMin ? rawMin : defaultMin;
      let upperBound = hasMax ? rawMax : defaultMax;
      if (upperBound < lowerBound) {
        upperBound = lowerBound;
      }
      const hasExplicitStep = slider.step !== undefined && slider.step !== null && slider.step !== '';
      const stepAttribute = hasExplicitStep ? slider.step : 1;
      const stepForFormatting = hasExplicitStep ? Number(slider.step) : undefined;
      const displayStep = Number.isNaN(stepForFormatting) ? undefined : stepForFormatting;

      range.min = lowerBound;
      range.max = upperBound;
      range.step = stepAttribute;
      range.value = slider.value;
      range.setAttribute('aria-label', `${slider.label} (range)`);

      const number = document.createElement('input');
      number.type = 'number';
      number.min = lowerBound;
      number.max = upperBound;
      number.step = stepAttribute;
      number.value = slider.value;
      number.setAttribute('aria-label', `${slider.label} (nummer)`);

      const setLabel = (text) => {
        label.innerHTML = `<span>${slider.label}</span><span>${text}</span>`;
      };

      const clampValue = (value) => {
        if (!Number.isFinite(value)) {
          return lowerBound;
        }
        if (value < lowerBound) return lowerBound;
        if (value > upperBound) return upperBound;
        return value;
      };

      let committedValue = clampValue(toNumeric(slider.value, lowerBound));

      const syncInputs = (value, { skipRange = false, skipNumber = false } = {}) => {
        if (!Number.isFinite(value)) {
          setLabel('');
          return;
        }
        setLabel(formatValue(value, displayStep));
        if (!skipRange) {
          range.value = clampValue(value);
        }
        if (!skipNumber) {
          number.value = value;
        }
      };

      syncInputs(committedValue);

      const handleRangeInput = (event) => {
        const numeric = Number(event.target.value);
        if (Number.isNaN(numeric)) {
          return;
        }
        syncInputs(numeric);
      };

      const handleNumberInput = (event) => {
        const rawValue = event.target.value;
        if (rawValue === '') {
          setLabel('');
          return;
        }
        const numeric = Number(rawValue);
        if (Number.isNaN(numeric)) {
          return;
        }
        syncInputs(numeric, { skipRange: true, skipNumber: true });
      };

      const commitDraft = (value) => {
        if (!Number.isFinite(value)) {
          syncInputs(committedValue);
          return;
        }
        const clamped = clampValue(value);
        if (clamped === committedValue) {
          syncInputs(clamped);
          return;
        }
        committedValue = clamped;
        syncInputs(clamped);
        engine.setSliderValue(slider.id, clamped);
      };

      const commitFromEvent = (event) => {
        const rawValue = event.target.value;
        if (rawValue === '') {
          commitDraft(committedValue);
          return;
        }
        const numeric = Number(rawValue);
        if (Number.isNaN(numeric)) {
          commitDraft(committedValue);
          return;
        }
        commitDraft(numeric);
      };

      range.addEventListener('input', handleRangeInput);
      range.addEventListener('change', commitFromEvent);
      number.addEventListener('input', handleNumberInput);
      number.addEventListener('change', commitFromEvent);
      number.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
          commitFromEvent(event);
        }
      });

      inputs.appendChild(range);
      inputs.appendChild(number);
      wrapper.appendChild(label);
      wrapper.appendChild(inputs);
      container.appendChild(wrapper);
    }
  }

  return { render };
}

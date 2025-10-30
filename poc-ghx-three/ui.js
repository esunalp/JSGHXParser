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

      const graphCount = Number(
        slider.graphCount ?? (Array.isArray(slider.members) ? slider.members.length : 1),
      );
      if (Number.isFinite(graphCount)) {
        wrapper.dataset.graphCount = String(graphCount);
      }

      const memberWarnings = Array.isArray(slider.members)
        ? slider.members.filter((member) => Array.isArray(member?.notes) && member.notes.length > 0)
        : [];

      if (slider.hasWarnings || (Array.isArray(slider.notes) && slider.notes.length) || memberWarnings.length) {
        wrapper.classList.add('slider-warning');
      }

      const label = document.createElement('div');
      label.className = 'slider-label';

      const labelName = document.createElement('span');
      labelName.className = 'slider-name';

      const labelNameText = document.createElement('span');
      labelNameText.className = 'slider-name-text';
      labelName.appendChild(labelNameText);

      const labelValue = document.createElement('span');
      labelValue.className = 'slider-value';

      label.appendChild(labelName);
      label.appendChild(labelValue);

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

      const sliderName = slider.label ?? slider.nickName ?? slider.id;
      labelNameText.textContent = sliderName ?? 'Slider';

      if (Number.isFinite(graphCount) && graphCount > 0) {
        const badge = document.createElement('span');
        badge.className = 'slider-badge';
        const badgeCount = Number(graphCount);
        const badgeText = badgeCount === 1 ? '×1' : `×${badgeCount}`;
        badge.textContent = badgeText;
        const badgeTitle = badgeCount === 1
          ? 'Deze slider is gekoppeld aan 1 grafiek.'
          : `Deze slider is gekoppeld aan ${badgeCount} grafieken.`;
        badge.title = badgeTitle;
        badge.setAttribute('aria-label', badgeTitle);
        labelName.appendChild(badge);
      }

      const setLabel = (text) => {
        if (text === undefined || text === null) {
          labelValue.textContent = '';
          return;
        }
        labelValue.textContent = String(text);
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
        const hasGroupBinding =
          typeof engine.setSliderGroupValue === 'function' && Array.isArray(slider.members);
        if (hasGroupBinding) {
          engine.setSliderGroupValue(slider.id, clamped);
          return;
        }
        const primaryGraphId =
          slider?.canonicalSource?.graphId ?? slider?.members?.[0]?.graphId ?? slider?.graphId;
        if (typeof engine.setSliderValue === 'function') {
          if (primaryGraphId) {
            engine.setSliderValue(slider.id, clamped, { graphId: primaryGraphId });
          } else {
            engine.setSliderValue(slider.id, clamped);
          }
        }
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

      const metaInfo = document.createElement('div');
      metaInfo.className = 'slider-meta';

      const graphLabels = Array.isArray(slider.members)
        ? slider.members
            .map((member) => member?.graphLabel ?? member?.graphId)
            .filter((entry) => typeof entry === 'string' && entry.trim().length > 0)
        : [];

      const metaParts = [];
      if (graphLabels.length) {
        const preview = graphLabels.slice(0, 3);
        const remainder = graphLabels.length - preview.length;
        const graphLabel =
          graphCount > 1
            ? `Grafieken: ${preview.join(', ')}${remainder > 0 ? ` (+${remainder} meer)` : ''}`
            : `Grafiek: ${preview[0]}`;
        metaParts.push(graphLabel);
      } else if (Number.isFinite(graphCount) && graphCount > 1) {
        metaParts.push(`${graphCount} gekoppelde grafieken`);
      }

      if (Array.isArray(slider.notes) && slider.notes.length) {
        metaParts.push(slider.notes.join(' • '));
      } else if (memberWarnings.length) {
        const warningDescriptions = memberWarnings.map((member) => {
          const source = member.graphLabel ?? member.graphId ?? 'Onbekende grafiek';
          const noteText = member.notes.join(', ');
          return noteText ? `${source}: ${noteText}` : source;
        });
        metaParts.push(warningDescriptions.join(' • '));
      }

      if (metaParts.length) {
        metaInfo.textContent = metaParts.join(' • ');
        wrapper.appendChild(metaInfo);
      }

      const tooltipLines = [];
      if (slider?.canonicalSource?.graphLabel) {
        tooltipLines.push(`Referentie: ${slider.canonicalSource.graphLabel}`);
      }
      if (Array.isArray(slider.members) && slider.members.length) {
        for (const member of slider.members) {
          const memberSource = member?.graphLabel ?? member?.graphId ?? 'Onbekende grafiek';
          const memberLabel = member?.label ?? member?.nickName ?? slider.label ?? slider.id;
          const memberNotes = Array.isArray(member?.notes) && member.notes.length
            ? ` — ${member.notes.join(', ')}`
            : '';
          tooltipLines.push(`${memberSource}: ${memberLabel}${memberNotes}`);
        }
      }
      if (Array.isArray(slider.notes) && slider.notes.length) {
        tooltipLines.push(...slider.notes);
      }
      if (tooltipLines.length) {
        wrapper.title = tooltipLines.join('\n');
      }

      container.appendChild(wrapper);
    }
  }

  return { render };
}

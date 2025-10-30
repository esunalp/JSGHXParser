const NOTE_MESSAGES = {
  'canonical-range-derived': 'Referentieslider mist grenzen; standaardwaarden toegepast.',
  'canonical-step-adjusted': 'Referentieslider-stap herberekend.',
  'canonical-value-clamped': 'Referentiewaarde begrensd tot beschikbaar bereik.',
  'missing-bounds': 'Ontbrekende minimum- of maximumwaarde ingevuld.',
  'no-range': 'Bereik ontbrak (min = max); waarde kan beperkt reageren.',
  'step-adjusted': 'Stapgrootte van gekoppelde slider geharmoniseerd.',
  'value-clamped': 'Waarde lag buiten het originele bereik en is begrensd.',
  normalized: 'Waarde opnieuw geschaald naar referentiebereik.',
  'invalid-value': 'Ongeldige sliderwaarde vervangen door standaard.',
};

const DEFAULT_MIN = 0;
const DEFAULT_MAX = 10;
const DEFAULT_STEP = 0.01;

function toNumber(value) {
  if (value === null || value === undefined) return undefined;
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : undefined;
}

function clamp(value, min, max) {
  if (!Number.isFinite(value)) {
    return min;
  }
  if (value < min) return min;
  if (value > max) return max;
  return value;
}

function slugify(value, fallback = 'slider') {
  if (!value || typeof value !== 'string') {
    return fallback;
  }
  const normalized = value
    .normalize('NFKD')
    .replace(/[^\w\s-]+/g, '')
    .trim()
    .toLowerCase()
    .replace(/[\s_-]+/g, '-');
  return normalized || fallback;
}

function normalizeNickname(nickname) {
  if (!nickname || typeof nickname !== 'string') {
    return '';
  }
  return nickname.trim().toLowerCase();
}

function computeRange(slider) {
  const rawMin = toNumber(slider?.min);
  const rawMax = toNumber(slider?.max);
  const rawStep = toNumber(slider?.step);
  const rawValue = toNumber(slider?.value);

  const hasMin = rawMin !== undefined;
  const hasMax = rawMax !== undefined;

  let min = rawMin;
  let max = rawMax;

  if (!hasMin && !hasMax) {
    min = DEFAULT_MIN;
    max = DEFAULT_MAX;
  } else if (!hasMin) {
    const candidate = hasMax ? rawMax : DEFAULT_MIN;
    min = Number.isFinite(candidate) ? Math.min(candidate, DEFAULT_MIN) : DEFAULT_MIN;
    max = Number.isFinite(rawMax) ? rawMax : DEFAULT_MAX;
  } else if (!hasMax) {
    const candidate = hasMin ? rawMin : DEFAULT_MAX;
    max = Number.isFinite(candidate) ? Math.max(candidate, DEFAULT_MAX) : DEFAULT_MAX;
    min = Number.isFinite(rawMin) ? rawMin : DEFAULT_MIN;
  }

  if (!Number.isFinite(min)) min = DEFAULT_MIN;
  if (!Number.isFinite(max)) max = Number.isFinite(min) ? min + 1 : DEFAULT_MAX;

  if (max < min) {
    const tmp = max;
    max = min;
    min = tmp;
  }

  let span = max - min;
  if (!Number.isFinite(span)) {
    span = 0;
  }

  let step = rawStep;
  let stepAdjusted = false;
  if (!Number.isFinite(step) || step <= 0) {
    step = span > 0 ? span / 100 : DEFAULT_STEP;
    stepAdjusted = true;
  }

  let value = rawValue;
  let valueProvided = rawValue !== undefined;
  if (!Number.isFinite(value)) {
    value = min;
    valueProvided = false;
  }

  const clampedValue = clamp(value, min, max);
  const valueClamped = clampedValue !== value;

  return {
    rawMin,
    rawMax,
    rawStep,
    rawValue,
    hasMin,
    hasMax,
    hasStep: rawStep !== undefined,
    min,
    max,
    span,
    step,
    stepAdjusted,
    value: clampedValue,
    valueClamped,
    valueProvided,
    hasRange: span > 0,
  };
}

function uniqueMessages(codes) {
  const seen = new Set();
  const messages = [];
  for (const code of codes) {
    if (!code || seen.has(code)) continue;
    seen.add(code);
    messages.push(NOTE_MESSAGES[code] ?? code);
  }
  return messages;
}

export class SliderLinker {
  constructor({ primaryRole = 'wireframe' } = {}) {
    this.primaryRole = primaryRole?.toLowerCase?.() ?? 'wireframe';
    this.groups = new Map();
    this.groupOrder = [];
  }

  clear() {
    this.groups.clear();
    this.groupOrder = [];
  }

  reconcile({ sources = [], activeGraphId = null } = {}) {
    this.clear();

    const entries = [];
    let order = 0;

    for (const source of sources) {
      if (!source) continue;
      const { graphId, metadata, sliders } = source;
      if (!graphId || !Array.isArray(sliders)) continue;
      for (const slider of sliders) {
        if (!slider) continue;
        const nickName = slider.nickName ?? slider.label ?? slider.nodeId ?? slider.id;
        entries.push({
          graphId,
          metadata: metadata ?? {},
          slider,
          nickName,
          order: order += 1,
        });
      }
    }

    const groupsByKey = new Map();

    for (const entry of entries) {
      const normalizedNick = normalizeNickname(entry.nickName);
      const key = normalizedNick || `${entry.graphId}:${entry.slider.nodeId ?? entry.slider.id ?? entry.order}`;
      if (!groupsByKey.has(key)) {
        groupsByKey.set(key, {
          key,
          nickName: entry.nickName,
          candidates: [],
          canonical: null,
        });
      }
      const group = groupsByKey.get(key);
      group.candidates.push(entry);
      if (!group.canonical) {
        group.canonical = entry;
      } else if (this.shouldReplaceCanonical(group.canonical, entry, activeGraphId)) {
        group.canonical = entry;
      }
    }

    const usedIds = new Map();

    for (const [key, group] of groupsByKey.entries()) {
      const canonical = group.canonical ?? group.candidates[0];
      const canonicalRange = computeRange(canonical.slider);
      const groupCodes = new Set();

      if (!canonicalRange.hasMin || !canonicalRange.hasMax) {
        groupCodes.add('canonical-range-derived');
      }
      if (canonicalRange.stepAdjusted) {
        groupCodes.add('canonical-step-adjusted');
      }
      if (canonicalRange.valueClamped) {
        groupCodes.add('canonical-value-clamped');
      }

      const canonicalLabel = canonical.slider.label ?? canonical.slider.nickName ?? canonical.slider.nodeId ?? canonical.slider.id ?? 'Slider';
      const canonicalNick = canonical.slider.nickName ?? canonical.slider.label ?? canonicalLabel;

      const members = [];
      const memberRecords = [];

      for (const candidate of group.candidates) {
        const range = computeRange(candidate.slider);
        const memberCodes = new Set();
        if (!range.hasMin || !range.hasMax) {
          memberCodes.add('missing-bounds');
        }
        if (!range.hasRange) {
          memberCodes.add('no-range');
        }
        if (range.stepAdjusted) {
          memberCodes.add('step-adjusted');
        }
        if (range.valueClamped) {
          memberCodes.add('value-clamped');
        }
        if (candidate.slider?.value !== undefined && !Number.isFinite(toNumber(candidate.slider.value))) {
          memberCodes.add('invalid-value');
        }

        let normalizedValue = canonicalRange.value;
        let normalized = false;
        if (canonicalRange.span > 0 && range.span > 0) {
          let relative = (range.value - range.min) / (range.span || 1);
          if (!Number.isFinite(relative)) {
            relative = 0;
          }
          const clampedRelative = Math.min(Math.max(relative, 0), 1);
          if (clampedRelative !== relative) {
            memberCodes.add('normalized');
          }
          normalizedValue = canonicalRange.min + clampedRelative * canonicalRange.span;
          normalized = true;
        } else if (range.span === 0) {
          normalizedValue = canonicalRange.min;
        }

        for (const code of memberCodes) {
          groupCodes.add(code);
        }

        const metadata = candidate.metadata ?? {};
        const memberPublic = {
          graphId: candidate.graphId,
          nodeId: candidate.slider.nodeId ?? candidate.slider.id,
          graphLabel: metadata.label ?? metadata.name ?? candidate.graphId,
          role: metadata.role ?? null,
          label: candidate.slider.label ?? candidate.slider.nickName ?? candidate.slider.nodeId ?? candidate.slider.id,
          nickName: candidate.slider.nickName ?? null,
          value: range.value,
          normalizedValue,
          range: {
            min: range.min,
            max: range.max,
            step: range.step,
          },
          notes: uniqueMessages(memberCodes),
          status: memberCodes.size ? 'warning' : 'ok',
        };

        members.push(memberPublic);
        memberRecords.push({
          public: memberPublic,
          transform: {
            canonicalMin: canonicalRange.min,
            canonicalSpan: canonicalRange.span,
            memberMin: range.min,
            memberMax: range.max,
            memberSpan: range.span,
          },
        });
      }

      const baseId = slugify(canonicalNick ?? canonicalLabel ?? key);
      const count = usedIds.get(baseId) ?? 0;
      usedIds.set(baseId, count + 1);
      const finalId = count ? `${baseId}-${count + 1}` : baseId;

      const groupPublic = {
        id: finalId,
        key,
        label: canonicalLabel,
        nickName: canonicalNick,
        value: canonicalRange.value,
        min: canonicalRange.min,
        max: canonicalRange.max,
        step: canonicalRange.step,
        graphCount: members.length,
        notes: uniqueMessages(groupCodes),
        hasWarnings: groupCodes.size > 0,
        canonicalSource: {
          graphId: canonical.graphId,
          nodeId: canonical.slider.nodeId ?? canonical.slider.id,
          graphLabel: canonical.metadata?.label ?? canonical.metadata?.name ?? canonical.graphId,
          role: canonical.metadata?.role ?? null,
        },
        members,
      };

      this.groups.set(finalId, {
        id: finalId,
        key,
        public: groupPublic,
        members: memberRecords,
        canonical: {
          min: canonicalRange.min,
          max: canonicalRange.max,
          span: canonicalRange.span,
        },
      });
      this.groupOrder.push(finalId);
    }
  }

  shouldReplaceCanonical(current, candidate, activeGraphId) {
    if (!current) return true;
    const currentMeta = current.metadata ?? {};
    const candidateMeta = candidate.metadata ?? {};
    const currentRole = currentMeta.role?.toLowerCase?.();
    const candidateRole = candidateMeta.role?.toLowerCase?.();

    if (candidateRole === this.primaryRole && currentRole !== this.primaryRole) {
      return true;
    }
    if (currentRole === this.primaryRole && candidateRole !== this.primaryRole) {
      return false;
    }

    const currentPrimary = currentMeta.primary === true || currentMeta.isPrimary === true;
    const candidatePrimary = candidateMeta.primary === true || candidateMeta.isPrimary === true;

    if (candidatePrimary && !currentPrimary) {
      return true;
    }
    if (currentPrimary && !candidatePrimary) {
      return false;
    }

    if (candidate.graphId === activeGraphId && current.graphId !== activeGraphId) {
      return true;
    }
    if (current.graphId === activeGraphId && candidate.graphId !== activeGraphId) {
      return false;
    }

    const candidatePriority = Number.isFinite(candidateMeta.priority) ? candidateMeta.priority : null;
    const currentPriority = Number.isFinite(currentMeta.priority) ? currentMeta.priority : null;
    if (candidatePriority !== null && currentPriority !== null && candidatePriority !== currentPriority) {
      return candidatePriority < currentPriority;
    }
    if (candidatePriority !== null && currentPriority === null) {
      return true;
    }
    if (candidatePriority === null && currentPriority !== null) {
      return false;
    }

    return candidate.order < current.order;
  }

  list() {
    const results = [];
    for (const id of this.groupOrder) {
      const record = this.groups.get(id);
      if (!record) continue;
      const group = record.public;
      results.push({
        id: group.id,
        key: group.key,
        label: group.label,
        nickName: group.nickName,
        value: group.value,
        min: group.min,
        max: group.max,
        step: group.step,
        graphCount: group.graphCount,
        notes: group.notes.slice(),
        hasWarnings: group.hasWarnings,
        canonicalSource: { ...group.canonicalSource },
        members: group.members.map((member) => ({
          ...member,
          notes: member.notes.slice(),
          range: { ...member.range },
        })),
      });
    }
    return results;
  }

  mapValue(groupId, canonicalValue) {
    const record = this.groups.get(groupId);
    if (!record) return null;

    const canonicalMin = record.canonical.min;
    const canonicalMax = record.canonical.max;
    const canonicalSpan = record.canonical.span;

    let numeric = toNumber(canonicalValue);
    if (!Number.isFinite(numeric)) {
      numeric = record.public.value;
    }
    const clamped = clamp(numeric, canonicalMin, canonicalMax);

    const updates = [];
    for (const member of record.members) {
      const transform = member.transform;
      const memberMin = transform.memberMin;
      const memberMax = transform.memberMax;
      const memberSpan = transform.memberSpan;

      let mapped = clamped;
      if (memberSpan > 0 && canonicalSpan > 0) {
        let relative = (clamped - canonicalMin) / (canonicalSpan || 1);
        if (!Number.isFinite(relative)) {
          relative = 0;
        }
        const clampedRelative = Math.min(Math.max(relative, 0), 1);
        mapped = memberMin + clampedRelative * memberSpan;
      }
      mapped = clamp(mapped, Math.min(memberMin, memberMax), Math.max(memberMin, memberMax));
      updates.push({
        graphId: member.public.graphId,
        nodeId: member.public.nodeId,
        value: mapped,
      });
    }

    return {
      value: clamped,
      updates,
    };
  }

  hasGroups() {
    return this.groupOrder.length > 0;
  }

  getGroup(groupId) {
    const record = this.groups.get(groupId);
    if (!record) return null;
    const [group] = this.list().filter((entry) => entry.id === groupId);
    return group ?? null;
  }
}

export async function createSliderLinker(options) {
  return new SliderLinker(options);
}

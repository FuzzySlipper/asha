const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/u;
const TASK_REF_PATTERN = /^#\d+$/u;
const EXEMPTION_KINDS = ['fileLineExemptions', 'rootBarrelExemptions'];

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isPositiveInteger(value) {
  return typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
}

function isIsoDate(value) {
  if (typeof value !== 'string' || !DATE_PATTERN.test(value)) {
    return false;
  }
  const parsed = new Date(`${value}T00:00:00Z`);
  return !Number.isNaN(parsed.valueOf()) && parsed.toISOString().slice(0, 10) === value;
}

export function validateBaselineChange(kind, rel, value, failures) {
  if (value === undefined) {
    return undefined;
  }
  if (!isRecord(value)) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange must be an object.`);
    return undefined;
  }

  if (!isIsoDate(value.changedAt)) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.changedAt must be an ISO date.`);
  }
  if (typeof value.changeTask !== 'string' || !TASK_REF_PATTERN.test(value.changeTask)) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.changeTask must be a Den task ref like #5505.`);
  }
  if (typeof value.removalTask !== 'string' || !TASK_REF_PATTERN.test(value.removalTask)) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.removalTask must be a Den task ref like #5506.`);
  }
  if (value.changeTask === value.removalTask) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.removalTask must name a distinct cleanup task.`);
  }
  if (typeof value.reason !== 'string' || value.reason.trim().length < 30) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.reason must explain the temporary raise.`);
  }
  if (value.previousMaxLines !== null && !isPositiveInteger(value.previousMaxLines)) {
    failures.push(
      `FAIL: ${rel} ${kind} baselineChange.previousMaxLines must be null or a positive integer.`,
    );
  }
  if (!isPositiveInteger(value.newMaxLines)) {
    failures.push(`FAIL: ${rel} ${kind} baselineChange.newMaxLines must be a positive integer.`);
  }

  return value;
}

export function auditTsSourceShapePolicy(basePolicy, currentPolicy, failures) {
  if (currentPolicy.maxSourceLines > basePolicy.maxSourceLines) {
    failures.push(
      `FAIL: global TypeScript source cap increased from ${basePolicy.maxSourceLines} to ` +
        `${currentPolicy.maxSourceLines}; split source files instead of raising maxSourceLines.`,
    );
  }

  for (const kind of EXEMPTION_KINDS) {
    const baseEntries = isRecord(basePolicy[kind]) ? basePolicy[kind] : {};
    const currentEntries = isRecord(currentPolicy[kind]) ? currentPolicy[kind] : {};
    for (const [rel, currentEntry] of Object.entries(currentEntries)) {
      if (!isRecord(currentEntry) || !isPositiveInteger(currentEntry.maxLines)) {
        continue;
      }
      const baseEntry = isRecord(baseEntries[rel]) ? baseEntries[rel] : undefined;
      const previousMaxLines = baseEntry?.maxLines;
      const isNew = baseEntry === undefined;
      const isRaised = isPositiveInteger(previousMaxLines) && currentEntry.maxLines > previousMaxLines;
      if (!isNew && !isRaised) {
        continue;
      }

      const change = validateBaselineChange(kind, rel, currentEntry.baselineChange, failures);
      if (change === undefined) {
        const action = isNew ? 'new exemption' : 'baseline increase';
        failures.push(
          `FAIL: ${rel} ${kind} ${action} requires baselineChange audit metadata with ` +
            'changedAt, changeTask, reason, removalTask, previousMaxLines, and newMaxLines.',
        );
        continue;
      }

      const expectedPreviousMaxLines = isNew ? null : previousMaxLines;
      if (change.previousMaxLines !== expectedPreviousMaxLines) {
        failures.push(
          `FAIL: ${rel} ${kind} baselineChange.previousMaxLines must equal ` +
            `${String(expectedPreviousMaxLines)} for this policy diff.`,
        );
      }
      if (change.newMaxLines !== currentEntry.maxLines) {
        failures.push(
          `FAIL: ${rel} ${kind} baselineChange.newMaxLines must equal ` +
            `${currentEntry.maxLines} for this policy diff.`,
        );
      }
    }
  }
}

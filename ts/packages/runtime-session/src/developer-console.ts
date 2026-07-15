import type {
  DeveloperConsoleSnapshot,
  DiagnosticSeverity,
} from '@asha/contracts';

/** A message owned by the consuming UI. It is never folded into Rust authority. */
export interface ConsumerLocalMessage {
  readonly id: string;
  readonly severity: DiagnosticSeverity;
  readonly message: string;
}

export interface DeveloperConsolePresentationEntry {
  readonly channel: 'runtime' | 'localUi';
  readonly severity: DiagnosticSeverity;
  readonly text: string;
  readonly correlation: string | null;
}

export interface DeveloperConsolePresentation {
  readonly runtime: readonly DeveloperConsolePresentationEntry[];
  readonly localUi: readonly DeveloperConsolePresentationEntry[];
  readonly droppedRuntimeRecordCount: number;
}

/** Full pull-down game console: recent runtime records plus a distinct local UI lane. */
export function projectDeveloperConsolePullDown(
  snapshot: DeveloperConsoleSnapshot,
  localMessages: readonly ConsumerLocalMessage[] = [],
  maximumRuntimeEntries = 40,
): DeveloperConsolePresentation {
  return projectDeveloperConsole(snapshot, localMessages, maximumRuntimeEntries, false);
}

/** Compact Studio activity/status view: warnings and errors, independently bounded. */
export function projectDeveloperConsoleActivity(
  snapshot: DeveloperConsoleSnapshot,
  localMessages: readonly ConsumerLocalMessage[] = [],
  maximumRuntimeEntries = 12,
): DeveloperConsolePresentation {
  return projectDeveloperConsole(snapshot, localMessages, maximumRuntimeEntries, true);
}

function projectDeveloperConsole(
  snapshot: DeveloperConsoleSnapshot,
  localMessages: readonly ConsumerLocalMessage[],
  maximumRuntimeEntries: number,
  importantOnly: boolean,
): DeveloperConsolePresentation {
  const boundedMaximum = Math.max(0, Math.floor(maximumRuntimeEntries));
  const eligibleRuntime = importantOnly
    ? snapshot.records.filter((record) => record.severity === 'warning' || record.severity === 'error' || record.severity === 'fatal')
    : snapshot.records;
  const runtimeRecords = boundedMaximum === 0 ? [] : eligibleRuntime.slice(-boundedMaximum);
  const runtime = runtimeRecords.map((record) => ({
    channel: 'runtime' as const,
    severity: record.severity,
    text: `[${record.category}] ${record.message}`,
    correlation: record.correlation,
  }));
  const localUi = localMessages.map((message) => ({
    channel: 'localUi' as const,
    severity: message.severity,
    text: message.message,
    correlation: message.id,
  }));
  return {
    runtime,
    localUi,
    droppedRuntimeRecordCount: snapshot.droppedRecordCount,
  };
}

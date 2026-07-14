import {
  projectDeveloperConsoleActivity,
  projectDeveloperConsolePullDown,
  type ConsumerLocalMessage,
  type DeveloperConsolePresentation,
  type RuntimeSessionFacade,
} from '@asha/runtime-session';

type ConsoleSession = Pick<RuntimeSessionFacade, 'readDeveloperConsole'>;

/** Package-root game consumer example; the shell decides how to draw the rows. */
export function readGamePullDownConsole(
  session: ConsoleSession,
  localMessages: readonly ConsumerLocalMessage[] = [],
): DeveloperConsolePresentation {
  return projectDeveloperConsolePullDown(session.readDeveloperConsole(), localMessages);
}

/** Package-root authoring consumer example; suitable for Studio activity/status UI. */
export function readStudioActivityStatus(
  session: ConsoleSession,
  localMessages: readonly ConsumerLocalMessage[] = [],
): DeveloperConsolePresentation {
  return projectDeveloperConsoleActivity(session.readDeveloperConsole(), localMessages);
}

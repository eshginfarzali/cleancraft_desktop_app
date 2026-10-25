/** English strings. `az.ts` must define exactly the same keys (a test checks it). */
export const en = {
  'app.tagline':
    'Removes cache and temp files that have not been modified for 24 hours. Only the folders you turn on in Settings are touched.',
  'tab.clean': 'Clean',
  'tab.history': 'History',
  'tab.settings': 'Settings',

  'scan.button': 'Scan',
  'scan.again': 'Scan again',
  'scan.scanning': 'Scanning…',
  'scan.empty': 'No targets are turned on. Choose some in Settings.',
  'scan.summary': 'Up to {size} can be freed from {count} files.',
  'scan.target.detail': '{count} files · {size}',
  'scan.target.unreadable': '{count} folders not readable',
  'scan.preview': 'Preview files',

  'clean.selected': 'Clean selected ({size})',
  'clean.cleaning': 'Cleaning…',
  'confirm.wholeTargets':
    'Permanently delete {size} from the selected folders? This cannot be undone.',
  'confirm.delete': 'Delete',
  'confirm.cancel': 'Cancel',
  'result.freed': 'Freed {size} by deleting {count} files.',
  'result.failed':
    '{count} files could not be deleted (in use or protected) and were left in place.',
  'result.rejected':
    '{count} selected items were skipped because they changed or are not allowed.',

  'preview.title': 'Preview: {target}',
  'preview.close': 'Close',
  'preview.selectAll': 'Select all',
  'preview.selectNone': 'Select none',
  'preview.selected': '{count} of {total} selected · {size}',
  'preview.truncated':
    'Showing the largest {shown} items. Cleaning the whole target removes all {total}.',
  'preview.empty': 'Nothing to delete here right now.',
  'preview.deleteSelected': 'Delete selected ({size})',
  'preview.confirm':
    'Permanently delete {count} selected items ({size})? This cannot be undone.',
  'preview.column.path': 'Path',
  'preview.column.size': 'Size',
  'preview.column.modified': 'Last modified',
  'preview.folderOf': 'folder, {count} files',

  'history.empty': 'No cleans yet.',
  'history.total': 'Freed so far: {size} in {count} cleans.',
  'history.clear': 'Clear history',
  'history.clearConfirm':
    'Delete the clean history? Deleted files are not affected.',
  'history.trigger.manual': 'Manual',
  'history.trigger.selective': 'Selected files',
  'history.trigger.scheduled': 'Scheduled',
  'history.entry': '{size} · {count} files',
  'history.entryFailed': '{count} failed',

  'settings.targets': 'What to clean',
  'settings.targets.help':
    'Each folder is fixed in the app; you cannot add your own paths. Only files older than 24 hours are removed.',
  'settings.unavailable': 'Not found on this computer',
  'settings.category.system': 'System',
  'settings.category.browser': 'Browsers',
  'settings.category.developer': 'Developer tools',
  'settings.schedule': 'Weekly clean',
  'settings.schedule.enabled':
    'Clean turned-on targets automatically every week',
  'settings.schedule.when': 'Day and time',
  'settings.schedule.help':
    'Runs while CleanCraft is open or in the system tray. If the computer is off at that time, it runs the next time CleanCraft is running.',
  'settings.appearance': 'Appearance',
  'settings.theme': 'Theme',
  'settings.theme.system': 'System',
  'settings.theme.light': 'Light',
  'settings.theme.dark': 'Dark',
  'settings.language': 'Language',

  'weekday.0': 'Monday',
  'weekday.1': 'Tuesday',
  'weekday.2': 'Wednesday',
  'weekday.3': 'Thursday',
  'weekday.4': 'Friday',
  'weekday.5': 'Saturday',
  'weekday.6': 'Sunday',

  'target.temp': 'Temporary files',
  'target.user-cache': 'Application caches',
  'target.chrome': 'Google Chrome cache',
  'target.edge': 'Microsoft Edge cache',
  'target.firefox': 'Firefox cache',
  'target.npm': 'npm cache',
  'target.yarn': 'Yarn cache',
  'target.pnpm': 'pnpm store',
  'target.xcode-derived-data': 'Xcode DerivedData (per project)',

  'error.prefix': 'Something went wrong: {message}',
} as const;

export type MessageKey = keyof typeof en;

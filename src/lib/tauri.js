// Tauri API adapter with mock implementation for development

// We'll use a simplified approach to avoid import issues
let tauriInvoke = null;
let tauriListen = null;

// Check if we're running in Tauri
const isTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

// If we're in Tauri, try to get the real functions
if (isTauri) {
  try {
    tauriInvoke = window.__TAURI__.invoke;
    tauriListen = window.__TAURI__.event.listen;
    console.log('Successfully loaded Tauri APIs');
  } catch (e) {
    console.error('Failed to load Tauri APIs:', e);
  }
}

// Mock implementations for dialog and notification since they're not directly exported

// Mock implementations
const mockInvoke = async (command, args) => {
  console.log(`Mock invoke: ${command}`, args);

  // Return mock data based on the command
  switch (command) {
    case 'get_settings':
      return {
        api_key: 'mock-api-key',
        welcome_completed: false,
        auto_scan_downloads: true,
        notify_on_scan_completion: true,
        auto_quarantine_malicious: false,
        scan_history_limit: 100,
        theme: 'system',
        startup_with_system: false,
        minimize_to_tray: true,
      };
    case 'get_scan_history':
      return [];
    case 'get_quarantined_files':
      return [];
    case 'scan_file':
      return {
        id: 'mock-scan-id',
        file_path: args.filePath,
        file_name: args.filePath.split(/[\\/]/).pop(),
        file_size: 1024,
        file_hash: 'mock-hash',
        scan_date: new Date().toISOString(),
        status: 'Pending',
        detection_count: null,
        total_engines: null,
        permalink: null,
      };
    default:
      return null;
  }
};

const mockOpen = async (options) => {
  console.log('Mock open dialog', options);
  return 'C:\\mock\\file.txt';
};

const mockListen = async (event, callback) => {
  console.log(`Mock listen: ${event}`);
  return () => console.log(`Mock unlisten: ${event}`);
};

const mockIsPermissionGranted = async () => {
  console.log('Mock isPermissionGranted');
  return true;
};

const mockRequestPermission = async () => {
  console.log('Mock requestPermission');
  return 'granted';
};

const mockSendNotification = async (options) => {
  console.log('Mock sendNotification', options);
};

// Log whether we're using real or mock APIs
console.log(isTauri ? 'Using real Tauri APIs' : 'Using mock Tauri APIs');

// Export the Tauri API functions (either real or mock)
export const invoke = isTauri && tauriInvoke ? tauriInvoke : mockInvoke;
export const listen = isTauri && tauriListen ? tauriListen : mockListen;

// Export mock implementations for dialog and notification
export const open = mockOpen;
export const save = mockOpen;
export const isPermissionGranted = mockIsPermissionGranted;
export const requestPermission = mockRequestPermission;
export const sendNotification = mockSendNotification;

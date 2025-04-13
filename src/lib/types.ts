// types.ts
export interface ScanResult {
    id: string;
    file_name: string;
    file_path: string;
    file_size: number;
    file_hash: string;
    scan_date: string;
    status: 'Clean' | 'Suspicious' | 'Malicious' | 'Failed';
    detection_count?: number;
    total_engines?: number;
    permalink?: string;
    vendor_results?: Record<string, VendorResult>;
}

export interface VendorResult {
    detected: boolean;
    result?: string;
    engine_name: string;
    engine_version?: string;
}

export interface QuarantinedFile {
    id: string;
    original_path: string;
    quarantine_path: string;
    file_name: string;
    file_size: number;
    file_hash: string;
    quarantine_date: string;
    detection_count: number;
    total_engines: number;
}

export interface Settings {
    api_key: string;
    welcome_completed: boolean;
    auto_scan_downloads: boolean;
    notify_on_scan_completion: boolean;
    auto_quarantine_malicious: boolean;
    auto_rescan_interval?: number;
    custom_scan_locations: string[];
    quarantine_dir?: string;
    scan_history_limit: number;
    file_type_filters: string[];
    theme: 'light' | 'dark' | 'system';
    startup_with_system: boolean;
    minimize_to_tray: boolean;
    export_path?: string;
    background_scan_threads: number;
}

export interface FileInfo {
    name: string;
    size: number;
    created: string;
    modified: string;
    type: string;
}

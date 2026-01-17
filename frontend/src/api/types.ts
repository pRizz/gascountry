// API types that mirror backend DTOs

// --- Repos ---

export interface Repo {
  id: string;
  path: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface AddRepoRequest {
  path: string;
  name?: string;
}

export interface ScanRequest {
  directories: string[];
  depth?: number;
}

export interface FoundRepo {
  path: string;
  name: string;
}

export interface ScanResponse {
  found: FoundRepo[];
}

// --- Sessions ---

export type SessionStatus = "idle" | "running" | "completed" | "error" | "cancelled";

export interface Session {
  id: string;
  repo_id: string;
  name: string | null;
  status: SessionStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateSessionRequest {
  repo_id: string;
  name?: string;
}

export type MessageRole = "user" | "assistant" | "system";

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: string;
  created_at: string;
}

export interface SessionDetails {
  id: string;
  repo_id: string;
  name: string | null;
  status: SessionStatus;
  created_at: string;
  updated_at: string;
  messages: Message[];
}

export interface RunSessionRequest {
  prompt: string;
}

export interface RunSessionResponse {
  session_id: string;
  status: SessionStatus;
  message: string;
}

export interface CancelSessionResponse {
  session_id: string;
  status: SessionStatus;
  message: string;
}

export type OutputStream = "stdout" | "stderr";

export interface OutputLog {
  id: number;
  session_id: string;
  stream: OutputStream;
  content: string;
  created_at: string;
}

export interface OutputResponse {
  session_id: string;
  logs: OutputLog[];
  total: number;
}

// --- Git ---

export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  staged: string[];
  unstaged: string[];
  untracked: string[];
}

export interface GitStatusResponse {
  session_id: string;
  branch: string;
  ahead: number;
  behind: number;
  staged: string[];
  unstaged: string[];
  untracked: string[];
}

export interface Commit {
  id: string;
  message: string;
  author: string;
  time: string;
}

export interface GitLogResponse {
  session_id: string;
  commits: Commit[];
}

export interface Branch {
  name: string;
  is_current: boolean;
  is_remote: boolean;
}

export interface GitBranchesResponse {
  session_id: string;
  branches: Branch[];
}

export interface FileDelta {
  path: string;
  added: number;
  removed: number;
}

export interface GitDiffResponse {
  session_id: string;
  files: FileDelta[];
  total_added: number;
  total_removed: number;
}

export interface CommandOutput {
  success: boolean;
  stdout: string;
  stderr: string;
}

export interface GitCommandResponse {
  session_id: string;
  success: boolean;
  stdout: string;
  stderr: string;
}

export interface CommitRequest {
  message: string;
  stage_all?: boolean;
}

export interface ResetRequest {
  confirm: boolean;
}

export interface CheckoutRequest {
  branch: string;
}

// --- Config ---

export interface ConfigResponse {
  config: Record<string, string>;
}

export interface UpdateConfigRequest {
  config: Record<string, string>;
}

export interface ConfigValueResponse {
  key: string;
  value: string | null;
}

export interface SetConfigValueRequest {
  value: string;
}

export interface AiBackend {
  id: string;
  name: string;
  description: string;
}

export interface BackendsResponse {
  backends: AiBackend[];
}

export interface Preset {
  id: string;
  name: string;
  description: string;
}

export interface PresetsResponse {
  presets: Preset[];
}

// --- WebSocket Messages ---

// Client → Server messages
export type WsClientMessage =
  | { type: "subscribe"; session_id: string }
  | { type: "unsubscribe"; session_id: string }
  | { type: "cancel"; session_id: string }
  | { type: "ping" };

// Server → Client messages
export type WsServerMessage =
  | { type: "subscribed"; session_id: string }
  | { type: "unsubscribed"; session_id: string }
  | { type: "output"; session_id: string; stream: OutputStream; content: string }
  | { type: "status"; session_id: string; status: SessionStatus }
  | { type: "error"; message: string }
  | { type: "pong" };

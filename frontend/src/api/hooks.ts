// React Query hooks for API data fetching

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "./client";
import type {
  AddRepoRequest,
  CreateSessionRequest,
  RunSessionRequest,
  CommitRequest,
  ResetRequest,
  CheckoutRequest,
  UpdateConfigRequest,
  SetConfigValueRequest,
} from "./types";

// Query key factories for consistent cache management
export const queryKeys = {
  repos: ["repos"] as const,
  sessions: ["sessions"] as const,
  session: (id: string) => ["sessions", id] as const,
  sessionOutput: (id: string) => ["sessions", id, "output"] as const,
  gitStatus: (sessionId: string) => ["git", sessionId, "status"] as const,
  gitLog: (sessionId: string) => ["git", sessionId, "log"] as const,
  gitBranches: (sessionId: string) => ["git", sessionId, "branches"] as const,
  gitDiff: (sessionId: string) => ["git", sessionId, "diff"] as const,
  config: ["config"] as const,
  configValue: (key: string) => ["config", key] as const,
  backends: ["config", "backends"] as const,
  presets: ["config", "presets"] as const,
};

// --- Repos ---

export function useRepos() {
  return useQuery({
    queryKey: queryKeys.repos,
    queryFn: api.listRepos,
  });
}

export function useAddRepo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: AddRepoRequest) => api.addRepo(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.repos });
    },
  });
}

export function useDeleteRepo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteRepo(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.repos });
    },
  });
}

export function useScanRepos() {
  return useMutation({
    mutationFn: api.scanRepos,
  });
}

// --- Sessions ---

export function useSessions() {
  return useQuery({
    queryKey: queryKeys.sessions,
    queryFn: api.listSessions,
  });
}

export function useSession(id: string | null) {
  return useQuery({
    queryKey: id ? queryKeys.session(id) : ["sessions", "none"],
    queryFn: () => (id ? api.getSession(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useCreateSession() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateSessionRequest) => api.createSession(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
    },
  });
}

export function useDeleteSession() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteSession(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
    },
  });
}

export function useRunSession() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, req }: { id: string; req: RunSessionRequest }) =>
      api.runSession(id, req),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.session(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
    },
  });
}

export function useCancelSession() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.cancelSession(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.session(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
    },
  });
}

export function useSessionOutput(
  id: string | null,
  params?: { stream?: "stdout" | "stderr"; limit?: number; offset?: number }
) {
  return useQuery({
    queryKey: id ? [...queryKeys.sessionOutput(id), params] : ["output", "none"],
    queryFn: () => (id ? api.getSessionOutput(id, params) : Promise.resolve(null)),
    enabled: !!id,
  });
}

// --- Git ---

export function useGitStatus(sessionId: string | null) {
  return useQuery({
    queryKey: sessionId ? queryKeys.gitStatus(sessionId) : ["git", "status", "none"],
    queryFn: () => (sessionId ? api.getGitStatus(sessionId) : Promise.resolve(null)),
    enabled: !!sessionId,
  });
}

export function useGitLog(sessionId: string | null, limit?: number) {
  return useQuery({
    queryKey: sessionId
      ? [...queryKeys.gitLog(sessionId), limit]
      : ["git", "log", "none"],
    queryFn: () => (sessionId ? api.getGitLog(sessionId, limit) : Promise.resolve(null)),
    enabled: !!sessionId,
  });
}

export function useGitBranches(sessionId: string | null) {
  return useQuery({
    queryKey: sessionId ? queryKeys.gitBranches(sessionId) : ["git", "branches", "none"],
    queryFn: () =>
      sessionId ? api.getGitBranches(sessionId) : Promise.resolve(null),
    enabled: !!sessionId,
  });
}

export function useGitDiff(sessionId: string | null) {
  return useQuery({
    queryKey: sessionId ? queryKeys.gitDiff(sessionId) : ["git", "diff", "none"],
    queryFn: () => (sessionId ? api.getGitDiff(sessionId) : Promise.resolve(null)),
    enabled: !!sessionId,
  });
}

export function useGitPull() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (sessionId: string) => api.gitPull(sessionId),
    onSuccess: (_, sessionId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.gitStatus(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.gitLog(sessionId) });
    },
  });
}

export function useGitPush() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (sessionId: string) => api.gitPush(sessionId),
    onSuccess: (_, sessionId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.gitStatus(sessionId) });
    },
  });
}

export function useGitCommit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ sessionId, req }: { sessionId: string; req: CommitRequest }) =>
      api.gitCommit(sessionId, req),
    onSuccess: (_, { sessionId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.gitStatus(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.gitLog(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.gitDiff(sessionId) });
    },
  });
}

export function useGitReset() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ sessionId, req }: { sessionId: string; req: ResetRequest }) =>
      api.gitReset(sessionId, req),
    onSuccess: (_, { sessionId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.gitStatus(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.gitDiff(sessionId) });
    },
  });
}

export function useGitCheckout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ sessionId, req }: { sessionId: string; req: CheckoutRequest }) =>
      api.gitCheckout(sessionId, req),
    onSuccess: (_, { sessionId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.gitStatus(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.gitBranches(sessionId) });
    },
  });
}

// --- Config ---

export function useConfig() {
  return useQuery({
    queryKey: queryKeys.config,
    queryFn: api.getConfig,
  });
}

export function useUpdateConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: UpdateConfigRequest) => api.updateConfig(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.config });
    },
  });
}

export function useConfigValue(key: string) {
  return useQuery({
    queryKey: queryKeys.configValue(key),
    queryFn: () => api.getConfigValue(key),
  });
}

export function useSetConfigValue() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ key, req }: { key: string; req: SetConfigValueRequest }) =>
      api.setConfigValue(key, req),
    onSuccess: (_, { key }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.config });
      queryClient.invalidateQueries({ queryKey: queryKeys.configValue(key) });
    },
  });
}

export function useDeleteConfigValue() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (key: string) => api.deleteConfigValue(key),
    onSuccess: (_, key) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.config });
      queryClient.invalidateQueries({ queryKey: queryKeys.configValue(key) });
    },
  });
}

export function useBackends() {
  return useQuery({
    queryKey: queryKeys.backends,
    queryFn: api.listBackends,
  });
}

export function usePresets() {
  return useQuery({
    queryKey: queryKeys.presets,
    queryFn: api.listPresets,
  });
}

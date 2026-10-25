import { request } from "../api/client";
export const coverageCall = <T,>(body: Record<string, unknown>) => request<T>("POST", "/api/tests/coverage", body);

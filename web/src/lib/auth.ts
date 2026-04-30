const KEY = 'sui-trace.token';
const USER_KEY = 'sui-trace.user';

export interface AuthUser {
  id: string;
  address: string;
  role: string;
}

export function getToken(): string | null {
  return localStorage.getItem(KEY);
}

export function getUser(): AuthUser | null {
  const raw = localStorage.getItem(USER_KEY);
  return raw ? (JSON.parse(raw) as AuthUser) : null;
}

export function login(token: string, user: AuthUser) {
  localStorage.setItem(KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export function logout() {
  localStorage.removeItem(KEY);
  localStorage.removeItem(USER_KEY);
}

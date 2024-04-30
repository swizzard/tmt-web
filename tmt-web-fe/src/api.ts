const API_URL = new URL("http://0.0.0.0:8080/");

export interface Authorized<T> {
  authToken: string;
  data: T;
}
export interface LoginInput {
  email: string;
  password: string;
}
export async function authorize({
  email: client_id,
  password: client_secret,
}: LoginInput): Promise<{ access_token: string }> {
  const endpoint = new URL("authorize", API_URL);
  const body = JSON.stringify({ client_id, client_secret });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  return resp.json();
}

export async function logout({
  authToken,
}: Authorized<{}>): Promise<{ ok: boolean }> {
  const endpoint = new URL("logout", API_URL);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
  });
  return resp.json();
}

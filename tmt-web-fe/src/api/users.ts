import { AuthorizedPost, TMTError } from ".";
import { API_URL } from "./_lib";

export async function authorize({
  email: client_id,
  password: client_secret,
}: LoginInput): Promise<AuthResponse> {
  const endpoint = new URL("authorize", API_URL);
  const body = JSON.stringify({ client_id, client_secret });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const data = await resp.json();
  if (resp.ok) {
    return data;
  } else {
    throw new TMTError(data.error);
  }
}
export async function logout({
  authToken,
}: AuthorizedPost<{}>): Promise<{ ok: boolean }> {
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

export async function signup(input: SignupInput): Promise<SignupResponse> {
  const endpoint = new URL("users", API_URL);
  const body = JSON.stringify(input);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const respData = await resp.json();
  if (resp.ok) {
    return respData;
  } else {
    throw new TMTError(respData.error);
  }
}
export async function confirmUser(
  userId: string,
  input: ConfirmUserInput,
): Promise<User> {
  const endpoint = new URL(`users/${userId}`, API_URL);
  const body = JSON.stringify(input);
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const respData = await resp.json();
  if (resp.ok) {
    return respData;
  } else {
    throw new TMTError(respData.error);
  }
}
export async function renewToken({
  token,
}: RenewRequest): Promise<AuthResponse> {
  const endpoint = new URL("renew", API_URL);
  const body = JSON.stringify({ token });
  const resp = await fetch(endpoint.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  });
  const data = await resp.json();
  if (!resp.ok) {
    throw new TMTError(data.error);
  } else {
    return data;
  }
}

export interface LoginInput {
  email: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
}

interface RenewRequest {
  token: string;
}

export interface SignupInput {
  password: string;
  email: string;
}

export interface SignupResponse {
  email: string;
  invite_id: string;
  user_id: string;
}

export interface ConfirmUserInput {
  email: string;
  invite_id: string;
}

export interface User {
  id: string;
  email: string;
  confirmed: boolean;
}

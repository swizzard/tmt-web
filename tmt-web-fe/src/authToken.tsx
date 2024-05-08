import { useState, useEffect } from "react";

const LOCAL_STORAGE_KEY = "authToken";

export default function useAuthToken() {
  const key = "authToken";
  const [authToken, setAuthToken_] = useState<string | null>(null);
  function setAuthToken(token: string) {
    setToken(token);
    setAuthToken_(token);
  }
  function refreshAuthToken() {
    const localToken = localStorage.getItem(key);
    if (localToken) {
      setAuthToken_(localToken);
    }
    return localToken;
  }
  useEffect(() => {
    const token = localStorage.getItem(key);
    setAuthToken_(token);
  });
  return { authToken, refreshAuthToken, setAuthToken };
}

export function getToken() {
  return localStorage.getItem(LOCAL_STORAGE_KEY);
}

export function setToken(token: string | null) {
  if (token) {
    localStorage.setItem(LOCAL_STORAGE_KEY, token);
  } else {
    localStorage.removeItem(LOCAL_STORAGE_KEY);
  }
}

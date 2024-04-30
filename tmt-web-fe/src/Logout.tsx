import { useState } from "react";
import { logout } from "./api";

export interface LogoutProps {
  setAuthToken: (authToken: string | undefined) => void;
  authToken: string | undefined;
}
export default function Logout({ authToken, setAuthToken }: LogoutProps) {
  const [err, setErr] = useState<string | undefined>(undefined);
  const onClick = async () => {
    if (!authToken) {
      setErr("already logged out");
    } else {
      try {
        const resp = await logout({ authToken: authToken!, data: {} });
        if (resp.ok) {
          console.log("logged out");
          setAuthToken(undefined);
        } else {
          console.log("logout error, logging out anyway");
          setAuthToken(undefined);
        }
      } catch (e: any) {
        setErr(e.message ?? JSON.stringify(e));
      }
    }
  };

  return (
    <div className="Logout">
      {err && <div className="err">{err}</div>}
      <button type="button" onClick={onClick}>
        Logout
      </button>
    </div>
  );
}

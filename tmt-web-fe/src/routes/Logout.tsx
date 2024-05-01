import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { logout } from "../api";

export interface LogoutProps {
  setAuthToken: (authToken: string | undefined) => void;
  authToken: string | undefined;
}
export default function Logout({ authToken, setAuthToken }: LogoutProps) {
  const navigate = useNavigate();
  const onClick = async () => {
    if (!authToken) {
      console.log("already logged out");
      navigate("/", { state: { authToken: undefined } });
    } else {
      try {
        const resp = await logout({ authToken: authToken!, data: {} });
        if (resp.ok) {
          console.log("logged out");
        } else {
          console.log("logout error, logging out anyway");
        }
        setAuthToken(undefined);
        navigate("/", { state: { authToken: undefined } });
      } catch (e: any) {
        console.error(e);
      }
    }
  };

  return (
    <div className="Logout">
      <button type="button" onClick={onClick}>
        Logout
      </button>
    </div>
  );
}

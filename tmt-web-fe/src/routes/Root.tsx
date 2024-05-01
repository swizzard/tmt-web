import React, { useState } from "react";
import { useLocation, Link } from "react-router-dom";
export interface RootProps {
  authToken?: string | undefined;
}
export default function Root({ authToken }: RootProps) {
  const { state } = useLocation();
  return (
    <div className="App">
      {authToken ?? state?.authToken ? (
        <div>
          <h1>Welcome!</h1>
          <p>You are logged in.</p>
          <p>
            <Link to="/logout">Logout</Link>
          </p>
        </div>
      ) : (
        <div>
          <h1>Welcome!</h1>
          <p>You are not logged in.</p>
          <p>
            <Link to="/login">Login</Link>
          </p>
        </div>
      )}
    </div>
  );
}

import React, { useEffect, useState } from "react";
import "./App.css";
import Login from "./Login";
import Logout from "./Logout";

function App() {
  const [authToken, setAuthToken] = useState<string | undefined>();

  useEffect(() => {
    console.log(` App got authToken: ${authToken}`);
  }, [authToken]);

  return (
    <div className="App">
      {authToken ? (
        <Logout authToken={authToken} setAuthToken={setAuthToken} />
      ) : (
        <Login setAuthToken={setAuthToken} />
      )}
    </div>
  );
}

export default App;

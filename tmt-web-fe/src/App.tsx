import React, { useEffect, useState } from "react";
import { RouterProvider } from "react-router-dom";
import router from "./Router";
import "./App.css";

function App() {
  const [authToken, setAuthToken] = useState<string | undefined>(undefined);
  return <RouterProvider router={router({ authToken, setAuthToken })} />;
}

export default App;

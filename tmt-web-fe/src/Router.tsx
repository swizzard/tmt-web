import { createBrowserRouter } from "react-router-dom";
import { Error, Login, Logout, Root, UserTabs } from "./routes";

export interface RouterProps {
  setAuthToken: (authToken: string | undefined) => void;
  authToken: string | undefined;
}

export default function router({ authToken, setAuthToken }: RouterProps) {
  return createBrowserRouter(
    [
      {
        path: "/",
        element: <Root authToken={authToken} />,
        errorElement: <Error />,
      },
      {
        path: "/login",
        element: <Login setAuthToken={setAuthToken} />,
        errorElement: <Error />,
      },
      {
        path: "/logout",
        element: <Logout authToken={authToken} setAuthToken={setAuthToken} />,
        errorElement: <Error />,
      },
      {
        path: "/tabs",
        element: <UserTabs authToken={authToken} setAuthToken={setAuthToken} />,
        errorElement: <Error />,
      },
    ],
    { basename: "/app" },
  );
}

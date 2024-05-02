import { createBrowserRouter } from "react-router-dom";
import * as routes from "./routes";

export interface RouterProps {
  setAuthToken: (authToken: string | undefined) => void;
  authToken: string | undefined;
}

export default function router({ authToken, setAuthToken }: RouterProps) {
  return createBrowserRouter(
    [
      {
        path: "/",
        element: <routes.Root authToken={authToken} />,
        errorElement: <routes.Error />,
      },
      {
        path: "/login",
        element: <routes.Login setAuthToken={setAuthToken} />,
        errorElement: <routes.Error />,
      },
      {
        path: "/logout",
        element: (
          <routes.Logout authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
      },
      {
        path: "/tabs",
        element: (
          <routes.UserTabs authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
      },
      {
        path: "/tabs/:tabId/edit",
        element: (
          <routes.EditTab authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
      },
      {
        path: "/tabs/:tabId/delete",
        element: (
          <routes.DeleteTab authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
      },
    ],
    { basename: "/app" },
  );
}

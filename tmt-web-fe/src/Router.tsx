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
        element: <routes.Login />,
        errorElement: <routes.Error />,
        action: routes.mkLoginAction(setAuthToken),
      },

      {
        path: "/logout",
        element: (
          <routes.Logout authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
      },
      {
        path: "/signup",
        element: <routes.SignUp />,
        errorElement: <routes.Error />,
        action: routes.signUpAction,
      },
      {
        path: "/confirm/:userId",
        element: <routes.ConfirmUser />,
        errorElement: <routes.Error />,
        action: routes.confirmUserAction,
      },
      {
        path: "/tabs",
        element: (
          <routes.UserTabs authToken={authToken} setAuthToken={setAuthToken} />
        ),
        errorElement: <routes.Error />,
        loader: async ({ request }) => request.headers.get("Authorization"),
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

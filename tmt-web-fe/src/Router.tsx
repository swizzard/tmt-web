import { createBrowserRouter, Navigate } from "react-router-dom";
import * as routes from "./routes";

export default function router() {
  return createBrowserRouter(
    [
      {
        path: "/",
        element: <routes.Root />,
        children: [
          {
            path: "",
            element: <routes.Home />,
            errorElement: <routes.Error />,
          },
          {
            path: "login",
            element: <routes.Login />,
            errorElement: <routes.Error />,
            action: routes.loginAction,
          },
          {
            path: "logout",
            element: <routes.Logout />,
            errorElement: <routes.Error />,
            action: routes.logoutAction,
          },
          {
            path: "signup",
            element: <routes.SignUp />,
            errorElement: <routes.Error />,
            action: routes.signUpAction,
          },
          {
            path: "confirm/:userId",
            element: <routes.ConfirmUser />,
            errorElement: <routes.Error />,
            action: routes.confirmUserAction,
          },
          {
            path: "tabs",
            children: [
              {
                path: "",
                element: <Navigate to="/tabs/personal/1" />,
              },
              {
                path: "create",
                element: <routes.CreateTab />,
                errorElement: <routes.Error />,
                action: routes.createTabAction,
              },
              {
                path: "personal/:page",
                element: <routes.UserTabs />,
                errorElement: <routes.Error />,
                loader: routes.userTabsLoader,
              },
              {
                path: "edit/:tabId",
                element: <routes.EditTab />,
                errorElement: <routes.Error />,
                action: routes.editTabAction,
                loader: routes.getTabDetailsLoader,
              },
              {
                path: "delete/:tabId",
                element: <routes.DeleteTab />,
                errorElement: <routes.Error />,
                action: routes.deleteTabAction,
                loader: routes.getTabDetailsLoader,
              },
            ],
          },
        ],
      },
    ],
    { basename: "/app" },
  );
}

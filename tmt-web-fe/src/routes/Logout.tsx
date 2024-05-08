import { Form, redirect } from "react-router-dom";
import { logout } from "../api";
import { getToken, setToken } from "../authToken";

export async function action({ request }: { request: Request }) {
  const authToken = getToken();
  if (!authToken) {
    throw new Error("Unauthorized");
  }
  try {
    await logout({ authToken, data: {} });
    setToken(null);
    return redirect("/");
  } catch (e) {
    console.error(e);
    throw new Error("Internal Server Error");
  }
}

export default function Logout() {
  return (
    <div className="Logout">
      <Form method="post" action="/logout">
        <button type="submit">Logout</button>
      </Form>
    </div>
  );
}

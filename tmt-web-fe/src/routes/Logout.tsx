import { Form, Link, redirect } from "react-router-dom";
import { logout } from "../api";
import { getToken, setToken } from "../authToken";
import "./Logout.css";

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
      <h1>Logout?</h1>
      <Form method="post" action="/logout">
        <button type="submit">Logout</button>
      </Form>
      <Link to="/tabs/personal/1">
        <button type="button">Home</button>
      </Link>
    </div>
  );
}

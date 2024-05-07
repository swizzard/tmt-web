import { Form, redirect } from "react-router-dom";
import { authorize, LoginInput } from "../api";

export function mkAction(
  setAuthToken: (authToken: string | undefined) => void,
) {
  return async function action({ request }: { request: Request }) {
    const formData = await request.formData();
    const data: LoginInput = {
      email: formData.get("email") as string,
      password: formData.get("password") as string,
    };
    const { access_token } = await authorize(data);
    setAuthToken(access_token);
    return redirect("/tabs");
  };
}
export default function Login() {
  return (
    <div className="Login">
      <Form method="post" action="/login">
        <label htmlFor="email">Email</label>
        <input type="email" id="email" name="email" />
        <label htmlFor="password">Password</label>
        <input type="password" id="password" name="password" />
        <button type="submit">Login</button>
      </Form>
    </div>
  );
}

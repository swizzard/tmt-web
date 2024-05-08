import { Form, redirect } from "react-router-dom";
import { authorize, LoginInput } from "../api";
import { setToken } from "../authToken";

export async function action({ request }: { request: Request }) {
  const formData = await request.formData();
  const data: LoginInput = {
    email: formData.get("email") as string,
    password: formData.get("password") as string,
  };
  try {
    const { access_token } = await authorize(data);
    setToken(access_token);
    return redirect("/tabs/personal/1");
  } catch (e) {
    throw e;
  }
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

import { Form, redirect } from "react-router-dom";
import { SignupInput, signup } from "../api";

export async function action({ request }: { request: Request }) {
  const formData = await request.formData();
  const data: SignupInput = {
    email: formData.get("email") as string,
    password: formData.get("password") as string,
  };
  try {
    const { invite_id, user_id } = await signup(data);
    return redirect(`/confirm/${user_id}?invite_id=${invite_id}`);
  } catch (e: any) {
    throw e;
  }
}

export default function SignUp() {
  return (
    <div className="SignUp">
      <Form method="post" action="/signup">
        <label htmlFor="email">Email</label>
        <input type="email" id="email" name="email" />
        <label htmlFor="password">Password</label>
        <input type="password" id="password" name="password" />
        <button type="submit">Sign Up</button>
      </Form>
    </div>
  );
}

import { Form, Params, redirect, useParams } from "react-router-dom";
import { ConfirmUserInput, confirmUser } from "../api";

export async function action({
  params,
  request,
}: {
  params: Params;
  request: Request;
}) {
  const userId = params.userId;
  const formData = await request.formData();
  const data: ConfirmUserInput = {
    email: formData.get("email") as string,
    invite_id: formData.get("invite_id") as string,
  };
  try {
    await confirmUser(userId!, data);
    return redirect("/login");
  } catch (e: any) {
    throw e;
  }
}

export default function ConfirmUser() {
  const params = useParams();
  return (
    <div className="ConfirmUser">
      <Form method="post" action={`/confirm/${params.userId}`}>
        <label htmlFor="email">Email</label>
        <input type="email" id="email" name="email" />
        <label htmlFor="invite_id">Invite ID</label>
        <input type="text" id="invite_id" name="invite_id" />
        <button type="submit">Confirm Invite</button>
      </Form>
    </div>
  );
}

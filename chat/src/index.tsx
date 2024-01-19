/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Chat } from "./pages/chat";
import { UserContextWrapper } from "./components/contexts/UserContext";

const root = document.getElementById("root");

render(
  () => (
    <UserContextWrapper>
      <Chat />
    </UserContextWrapper>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);

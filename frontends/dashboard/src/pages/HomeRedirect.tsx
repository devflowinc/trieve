import { useNavigate } from "@solidjs/router";
import { onMount } from "solid-js";

export const HomeRedirect = () => {
  const navigate = useNavigate();

  onMount(() => {
    navigate("/org");
  });

  return <></>;
};

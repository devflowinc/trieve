import { createEffect, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";

export const Home = () => {
  const userContext = useContext(UserContext);

  createEffect(() => {
    window.location.href = `${
      window.origin
    }/dashboard/${userContext.selectedOrganizationId?.()}`;
  });
  return <div />;
};

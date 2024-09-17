import { useContext } from "solid-js";
import { ApiContext } from "..";

export const useTrieve = () => {
  const trieve = useContext(ApiContext);
  return trieve;
};

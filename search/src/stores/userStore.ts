import { atom } from "nanostores";
import { UserDTO, isUserDTO } from "../../utils/apiTypes";
import { persistentAtom } from "@nanostores/persistent";

const apiHost: string = import.meta.env.VITE_API_HOST;

export const isLoadingUser = atom<boolean>(true);
export const currentUser = persistentAtom("currentUser", null, {
  encode: JSON.stringify,
  decode: (encoded: string) => {
    try {
      if (isUserDTO(JSON.parse(encoded))) {
        return JSON.parse(encoded) as UserDTO;
      } else {
        return null;
      }
    } catch (e) {
      return null;
    }
  },
});

isLoadingUser.set(true);

fetch(`${apiHost}/auth/me`, {
  credentials: "include",
})
  .then((res) => {
    if (res.status === 401) {
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard`;
    }
    if (res.ok) {
      return res.json();
    }
  })
  .then((data) => {
    currentUser.set(data as unknown as UserDTO);
  })
  .catch((err) => {
    console.error(err);
  })
  .finally(() => {
    isLoadingUser.set(false);
  });

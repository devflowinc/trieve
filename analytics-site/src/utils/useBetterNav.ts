import { useNavigate, useSearchParams } from "@solidjs/router";

export const useBetterNav = () => {
  const [queryParams] = useSearchParams();
  const navigate = useNavigate();

  return (pathname: string) => {
    // Preserve the
    const searchParams = new URLSearchParams(
      queryParams as Record<string, string>,
    );
    const search = searchParams.toString();
    navigate(pathname + (search ? `?${search}` : ""));
  };
};

import { useSearchParams } from "solid-start";
import { detectReferralToken } from "~/types/actix-api";

const PaymentSuccess = () => {
  const [searchParams] = useSearchParams();
  detectReferralToken(searchParams.t);

  return (
    <div class="flex h-screen w-screen items-center justify-center bg-neutral-50 px-10 dark:bg-neutral-800">
      <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
        <a href="/" class="flex flex-col items-center">
          <img src="/Logo.png" alt="Arguflow Logo" class="mx-auto my-2" />
        </a>
        <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
          <div class="flex flex-col space-y-2 text-center font-bold">
            <span class="py-2 text-3xl">Thank you for your payment!</span>
            <span class="py-2 text-lg">
              Please check your email to finish registration
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PaymentSuccess;

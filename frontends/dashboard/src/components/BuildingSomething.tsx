import { BiRegularInfoCircle } from "solid-icons/bi";

export const BuildingSomething = () => {
  return (
    <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
      <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
      <p class="text-sm text-blue-700">
        Building something? Share in our{" "}
        <a class="underline" href="https://discord.gg/s4CX3vczyn">
          Discord
        </a>{" "}
        or{" "}
        <a
          class="underline"
          href="https://matrix.to/#/#trieve-general:trieve.ai"
        >
          Matrix
        </a>
        ; we would love to hear about it!
      </p>
    </div>
  );
};

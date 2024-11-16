import { createSignal } from "solid-js";

function App() {
  const [count, setCount] = createSignal(0);

  return (
    <>
      <div></div>
      <h1>Vite + Solid</h1>
      <div class="bg-red-200">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count()}
        </button>
      </div>
      <p class="read-the-docs">Hello world</p>
    </>
  );
}

export default App;

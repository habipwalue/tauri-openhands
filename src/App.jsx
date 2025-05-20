import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [num1, setNum1] = useState(0);
  const [num2, setNum2] = useState(0);
  const [sumResult, setSumResult] = useState(null);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function calculateSum() {
    // Call our new sum_numbers function
    const result = await invoke("sum_numbers", { a: parseInt(num1), b: parseInt(num2) });
    setSumResult(result);
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <div className="calculator">
        <h2>Number Calculator</h2>
        <div className="row">
          <input
            type="number"
            value={num1}
            onChange={(e) => setNum1(e.currentTarget.value)}
            placeholder="First number"
          />
          <span>+</span>
          <input
            type="number"
            value={num2}
            onChange={(e) => setNum2(e.currentTarget.value)}
            placeholder="Second number"
          />
          <button onClick={calculateSum}>Calculate Sum</button>
        </div>
        {sumResult !== null && (
          <p>
            Sum result: <strong>{sumResult}</strong>
          </p>
        )}
      </div>
    </main>
  );
}

export default App;

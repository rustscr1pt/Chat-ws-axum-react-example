import './App.css';
import {useEffect, useState} from "react";

function App() {

  const [messages, setMessages] = useState([]);
  const [socket, setSocket] = useState(undefined);

  useEffect(() => {
    const socket = new WebSocket("ws://localhost:8000/");
    socket.onmessage = (e) => setMessages((prev) => [...prev, e.data]);
    setSocket(socket);
    return () => socket.close();
  }, []);

  const submit = (e) => {
    e.preventDefault();
    if (!socket) return;
    socket.send(e.target.input.value);
    e.target.input.value = "";
  };

  return (
    <>
      <h1>WebSocket Chat App</h1>
      <ul>
        {
          messages.map((body, index) => (
              <li key={index}>
                {body}
              </li>
          ))
        }
      </ul>
      <form onSubmit={submit}>
        <input type="text" name="input"/>
        <button type="submit">Send</button>
      </form>
    </>
  );
}

export default App;

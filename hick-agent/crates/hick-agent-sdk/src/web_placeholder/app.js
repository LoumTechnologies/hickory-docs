(() => {
  const logEl = document.getElementById("log");
  const promptEl = document.getElementById("prompt");

  function line(text) {
    logEl.textContent += text + "\n";
    logEl.scrollTop = logEl.scrollHeight;
  }

  const wsProto = location.protocol === "https:" ? "wss:" : "ws:";
  const ws = new WebSocket(`${wsProto}//${location.host}/ws`);

  ws.onopen = () => line("[ws] connected");
  ws.onclose = () => line("[ws] closed");
  ws.onerror = () => line("[ws] error");
  ws.onmessage = (ev) => {
    try {
      const v = JSON.parse(ev.data);
      line(JSON.stringify(v));
    } catch {
      line(String(ev.data));
    }
  };

  function sendPayload(obj) {
    if (ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify(obj));
  }

  document.getElementById("send").onclick = () => {
    const text = promptEl.value.trim();
    if (!text) return;
    sendPayload({ type: "message", text });
    promptEl.value = "";
  };

  document.getElementById("cancel").onclick = () => sendPayload({ type: "cancel" });

  promptEl.addEventListener("keydown", (e) => {
    if (e.key === "Enter") document.getElementById("send").click();
  });
})();

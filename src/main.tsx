import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";

// macOS WebKit touch event workaround for transparent Tauri windows.
// Touch events interfere with click detection — blocking them restores normal behavior.
// See: https://github.com/tauri-apps/tauri/discussions/11957
if (navigator.userAgent.includes("Mac")) {
  const orig = EventTarget.prototype.addEventListener;
  EventTarget.prototype.addEventListener = function (type, listener, options) {
    if (type === "touchstart" || type === "touchend" || type === "touchmove") return;
    return orig.call(this, type, listener, options);
  };
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

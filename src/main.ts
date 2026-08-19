import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

/**
 * Red de seguridad: si la interfaz falla al montar, una ventana en blanco no
 * dice nada. Esto pinta el error para que se pueda leer y reportar.
 */
function pantallaDeError(detalle: string) {
  document.body.innerHTML = "";
  const box = document.createElement("div");
  box.setAttribute(
    "style",
    "font:14px/20px system-ui,sans-serif;color:#141413;background:#f4f4f2;" +
      "height:100%;padding:24px;overflow:auto;box-sizing:border-box"
  );
  const title = document.createElement("h1");
  title.textContent = "La interfaz no pudo cargar";
  title.setAttribute("style", "font-size:18px;font-weight:600;margin:0 0 8px");
  const pre = document.createElement("pre");
  pre.textContent = detalle;
  pre.setAttribute(
    "style",
    "white-space:pre-wrap;color:#5e5d58;font:12px/18px ui-monospace,monospace;margin:0"
  );
  box.append(title, pre);
  document.body.append(box);
}

window.addEventListener("error", (e) => {
  pantallaDeError(`${e.message}\n${e.filename}:${e.lineno}:${e.colno}`);
});

window.addEventListener("unhandledrejection", (e) => {
  pantallaDeError(String(e.reason?.stack ?? e.reason));
});

const target = document.getElementById("app");
if (!target) {
  pantallaDeError("No se encontró el contenedor #app en index.html");
} else {
  try {
    mount(App, { target });
  } catch (e) {
    pantallaDeError(e instanceof Error ? (e.stack ?? e.message) : String(e));
  }
}

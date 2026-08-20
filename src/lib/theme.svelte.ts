// Tema claro/oscuro (Fase 3). Los tokens de ambos ya viven en app.css; esto
// solo decide cuál se aplica y lo recuerda en la base.

import * as api from "./api";

export type Tema = "claro" | "oscuro" | "sistema";

/** Estado global del tema. Las gráficas lo leen para repintarse al cambiar. */
export const tema = $state({ elegido: "claro" as Tema, aplicado: "claro" as "claro" | "oscuro" });

function resolver(elegido: Tema): "claro" | "oscuro" {
  if (elegido !== "sistema") return elegido;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "oscuro" : "claro";
}

function pintar() {
  tema.aplicado = resolver(tema.elegido);
  document.documentElement.dataset.theme = tema.aplicado === "oscuro" ? "dark" : "light";
}

/** Se llama una vez al arrancar, con el valor guardado en ajustes. */
export function iniciarTema(guardado: string | undefined) {
  tema.elegido = (["claro", "oscuro", "sistema"] as const).includes(guardado as Tema)
    ? (guardado as Tema)
    : "claro";
  pintar();

  // Si sigue al sistema, hay que reaccionar cuando Windows cambie de modo.
  window
    .matchMedia?.("(prefers-color-scheme: dark)")
    .addEventListener("change", () => tema.elegido === "sistema" && pintar());
}

export async function cambiarTema(nuevo: Tema) {
  tema.elegido = nuevo;
  pintar();
  await api.setSettings({ theme: nuevo });
}

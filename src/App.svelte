<script lang="ts">
  import Shell from "./lib/components/Shell.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Today from "./routes/Today.svelte";
  import History from "./routes/History.svelte";
  import DayDetail from "./routes/DayDetail.svelte";
  import Stats from "./routes/Stats.svelte";
  import Settings from "./routes/Settings.svelte";
  import Onboarding from "./routes/Onboarding.svelte";
  import Recovery from "./routes/Recovery.svelte";
  import BrokenStreak from "./routes/BrokenStreak.svelte";
  import QuickEntry from "./routes/QuickEntry.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import * as api from "./lib/api";
  import type { Bootstrap, MissingDay, TodayView } from "./lib/types";

  // Las dos ventanas cargan el mismo index.html; lo que las distingue es su
  // etiqueta. Un fragmento en la URL no sobrevive al protocolo de assets.
  const esCapturaRapida = getCurrentWindow().label === "quick";

  let boot = $state<Bootstrap | null>(null);
  let view = $state<TodayView | null>(null);
  let missing = $state<MissingDay[]>([]);
  let showRecovery = $state(false);
  let route = $state<"hoy" | "historial" | "estadisticas" | "ajustes">("hoy");
  let openDate = $state<string | null>(null);
  let brokenDate = $state<string | null>(null);
  let fatal = $state("");

  async function load() {
    try {
      boot = await api.bootstrap();
      if (!boot.needsOnboarding) {
        view = await api.getToday();
        missing = await api.getMissingDays();
        showRecovery = missing.length > 0;
      }
    } catch (e) {
      fatal = e instanceof Error ? e.message : String(e);
    }
  }

  if (!esCapturaRapida) load();

  async function refreshToday() {
    view = await api.getToday();
  }

  function goto(to: string) {
    openDate = null;
    brokenDate = null;
    route = to as typeof route;
  }
</script>

{#if esCapturaRapida}
  <QuickEntry />
{:else if fatal}
  <TitleBar />
  <div class="fatal">
    <h1 class="section-title">No se pudo abrir la base de datos</h1>
    <p class="muted">{fatal}</p>
    <p class="hint">
      Tus datos viven en %APPDATA%\75hard\data.db. Si el archivo se movió o quedó bloqueado por
      otro proceso, ciérralo y vuelve a abrir la app.
    </p>
    <button class="primary" onclick={() => location.reload()}>Reintentar</button>
  </div>
{:else if !boot}
  <TitleBar />
  <div class="loading muted">Abriendo…</div>
{:else if boot.needsOnboarding}
  <TitleBar subtitle="Nuevo reto" />
  <Onboarding {boot} ondone={load} />
{:else if brokenDate}
  <TitleBar subtitle="Racha rota" />
  <BrokenStreak
    date={brokenDate}
    onhistory={() => {
      brokenDate = null;
      route = "historial";
    }}
    onrestart={async () => {
      brokenDate = null;
      await load();
    }}
    ondismiss={async () => {
      brokenDate = null;
      await refreshToday();
    }}
  />
{:else if view}
  <Shell
    {route}
    subtitle={view.dayNumber ? `Día ${view.dayNumber} de ${view.targetDays}` : view.weekdayLabel}
    streak={view.streak}
    onnavigate={goto}
  >
    {#if openDate}
      <DayDetail
        date={openDate}
        onback={async () => {
          openDate = null;
          await refreshToday();
        }}
      />
    {:else if route === "hoy"}
      <Today {view} onupdate={(v) => (view = v)} onbroken={(d) => (brokenDate = d)} />
    {:else if route === "historial"}
      <History onopenday={(d) => (openDate = d)} />
    {:else if route === "estadisticas"}
      <Stats />
    {:else}
      <Settings {boot} onreload={load} />
    {/if}
  </Shell>

  {#if showRecovery}
    <Recovery
      days={missing}
      onclose={() => (showRecovery = false)}
      ondone={async (aLlenar) => {
        showRecovery = false;
        await refreshToday();
        // "Llenar ahora" lleva directo al primero de esos días.
        if (aLlenar.length > 0) {
          openDate = aLlenar[0];
        } else {
          missing = [];
        }
      }}
    />
  {/if}
{/if}

<style>
  .loading,
  .fatal {
    height: calc(100% - var(--titlebar-h));
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 48px;
    text-align: center;
  }

  .fatal p {
    margin: 0;
    max-width: 520px;
  }
</style>

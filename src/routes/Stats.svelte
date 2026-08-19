<script lang="ts">
  // Fase 2. Esta pantalla existe para que la navegación sea honesta: dice qué
  // falta en vez de mostrar gráficas vacías.
  const previstas = [
    ["Sueño", "Barras diarias con línea de meta, promedio móvil de 7 días, histograma de hora de dormir."],
    ["Ejercicio", "Minutos por semana con split bajo techo / al aire libre y distribución por tipo."],
    ["Glucosa", "Dispersión temporal con bandas de referencia por contexto y exportación a PDF para el médico."],
    ["Peso y trabajo", "Peso con promedio móvil de 7 días (nunca el dato crudo) y horas por categoría."],
  ];
</script>

<div class="page">
  <h1 class="section-title">Estadísticas</h1>
  <div class="card note">
    <p>
      Las gráficas son la Fase 2. Los datos ya se están guardando desde hoy, así que cuando
      lleguen tendrán historia de verdad que mostrar.
    </p>
  </div>

  <ul class="rows">
    {#each previstas as [nombre, detalle] (nombre)}
      <li>
        <span class="name">{nombre}</span>
        <span class="muted">{detalle}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
  .page {
    max-width: 900px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .note {
    background: var(--surface-sunken);
  }

  .note p {
    margin: 0;
    color: var(--ink-secondary);
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    background: var(--surface-2);
  }

  .rows li {
    display: grid;
    grid-template-columns: 160px 1fr;
    gap: 16px;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
  }

  .rows li:last-child {
    border-bottom: none;
  }

  .name {
    font-weight: 500;
  }
</style>

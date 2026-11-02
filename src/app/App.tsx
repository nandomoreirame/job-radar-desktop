const SPIKE_COLUMNS = 6;
const SPIKE_CARDS = 4;

export function App() {
  return (
    <main className="noir-backdrop flex h-full flex-col gap-4 p-6">
      <h1 className="text-lg font-semibold">job-radar</h1>
      <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto">
        {Array.from({ length: SPIKE_COLUMNS }, (_, column) => (
          <section key={column} className="noir-panel flex flex-col gap-3 rounded-xl p-3">
            {Array.from({ length: SPIKE_CARDS }, (_, row) => (
              <div key={row} className="noir-card h-28 rounded-lg" />
            ))}
          </section>
        ))}
      </div>
    </main>
  );
}

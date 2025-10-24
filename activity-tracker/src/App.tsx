import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Settings from "./Settings";

interface ActiveWindowInfo {
  title: string;
  app_name: string;
}

interface AggregatedActivity {
  app_name: string;
  window_title: string;
  duration: number; // in seconds
}

function App() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindowInfo | null>(null);
  const [isIdle, setIsIdle] = useState(false);
  const [report, setReport] = useState<AggregatedActivity[]>([]);
  const [view, setView] = useState("main");

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const info: ActiveWindowInfo = await invoke("get_active_window_info");
        setActiveWindow(info);
        const idle: boolean = await invoke("is_idle");
        setIsIdle(idle);
      } catch (error) {
        console.error("Error fetching active window info:", error);
      }
    }, 1000); // Poll every second

    return () => clearInterval(interval);
  }, []);

  async function getReport() {
    try {
      const today = new Date().toISOString().slice(0, 10);
      const dailyReport: AggregatedActivity[] = await invoke("get_daily_report", { date: today });
      setReport(dailyReport);
    } catch (error) {
      console.error("Error fetching daily report:", error);
    }
  }

  if (view === "settings") {
    return <Settings setView={setView} />;
  }

  return (
    <main className="container">
      <div className="row">
        <h1>Activity Tracker</h1>
        <button onClick={() => setView("settings")}>Settings</button>
      </div>
      <div className="row">
        <p>
          <strong>Status:</strong> {isIdle ? "Idle" : "Active"}
        </p>
      </div>
      {activeWindow && (
        <div className="row">
          <p>
            <strong>App:</strong> {activeWindow.app_name}
          </p>
          <p>
            <strong>Title:</strong> {activeWindow.title}
          </p>
        </div>
      )}
      <div className="row">
        <button onClick={getReport}>Get Daily Report</button>
      </div>
      {report.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>App Name</th>
              <th>Window Title</th>
              <th>Duration (seconds)</th>
            </tr>
          </thead>
          <tbody>
            {report.map((activity, index) => (
              <tr key={index}>
                <td>{activity.app_name}</td>
                <td>{activity.window_title}</td>
                <td>{activity.duration}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}

export default App;

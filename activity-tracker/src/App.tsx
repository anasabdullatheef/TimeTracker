import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ActiveWindowInfo {
  title: string;
  app_name: string;
}

interface Activity {
  id: number;
  app_name: string;
  window_title: string;
  start_time: string;
  end_time: string;
}

function App() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindowInfo | null>(null);
  const [isIdle, setIsIdle] = useState(false);
  const [report, setReport] = useState<Activity[]>([]);

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
      const dailyReport: Activity[] = await invoke("get_daily_report", { date: today });
      setReport(dailyReport);
    } catch (error) {
      console.error("Error fetching daily report:", error);
    }
  }

  return (
    <main className="container">
      <h1>Activity Tracker</h1>
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
              <th>Start Time</th>
              <th>End Time</th>
            </tr>
          </thead>
          <tbody>
            {report.map((activity) => (
              <tr key={activity.id}>
                <td>{activity.app_name}</td>
                <td>{activity.window_title}</td>
                <td>{new Date(activity.start_time).toLocaleTimeString()}</td>
                <td>{new Date(activity.end_time).toLocaleTimeString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}

export default App;

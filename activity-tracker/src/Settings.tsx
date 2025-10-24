import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SettingsProps {
    setView: (view: string) => void;
}

const Settings = ({ setView }: SettingsProps) => {
    const [idleTime, setIdleTime] = useState(300);
    const [ignoredApps, setIgnoredApps] = useState<string[]>([]);
    const [newIgnoredApp, setNewIgnoredApp] = useState("");

    useEffect(() => {
        const fetchIdleTime = async () => {
            try {
                const time: number = await invoke("get_idle_time");
                setIdleTime(time);
            } catch (error) {
                console.error("Error fetching idle time:", error);
            }
        };
        const fetchIgnoredApps = async () => {
            try {
                const apps: string[] = await invoke("get_ignored_apps");
                setIgnoredApps(apps);
            } catch (error) {
                console.error("Error fetching ignored apps:", error);
            }
        };
        fetchIdleTime();
        fetchIgnoredApps();
    }, []);

    const handleSave = async () => {
        try {
            await invoke("set_idle_time", { idleTime });
            alert("Settings saved!");
        } catch (error) {
            console.error("Error saving idle time:", error);
        }
    };

    const handleAddIgnoredApp = async () => {
        try {
            await invoke("add_ignored_app", { appName: newIgnoredApp });
            setIgnoredApps([...ignoredApps, newIgnoredApp]);
            setNewIgnoredApp("");
        } catch (error) {
            console.error("Error adding ignored app:", error);
        }
    };

    const handleRemoveIgnoredApp = async (appName: string) => {
        try {
            await invoke("remove_ignored_app", { appName });
            setIgnoredApps(ignoredApps.filter((app) => app !== appName));
        } catch (error) {
            console.error("Error removing ignored app:", error);
        }
    };

  return (
    <div>
      <h1>Settings</h1>
      <div>
        <label htmlFor="idleTime">Idle Time (seconds):</label>
        <input
            type="number"
            id="idleTime"
            value={idleTime}
            onChange={(e) => setIdleTime(parseInt(e.target.value, 10))}
        />
      </div>
      <button onClick={handleSave}>Save</button>
      <div>
        <h2>Ignored Apps</h2>
        <ul>
            {ignoredApps.map((app) => (
                <li key={app}>
                    {app} <button onClick={() => handleRemoveIgnoredApp(app)}>Remove</button>
                </li>
            ))}
        </ul>
        <input
            type="text"
            value={newIgnoredApp}
            onChange={(e) => setNewIgnoredApp(e.target.value)}
        />
        <button onClick={handleAddIgnoredApp}>Add</button>
      </div>
      <button onClick={() => setView("main")}>Back</button>
    </div>
  );
};

export default Settings;

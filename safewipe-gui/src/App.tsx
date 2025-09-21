import React, { useState, useEffect, createContext, useContext } from "react";
import { ThemeProvider, createTheme, CssBaseline } from "@mui/material";
import Dashboard from "./screens/Dashboard";
import DeviceSelection from "./screens/DeviceSelection";
import WipeMethod from "./screens/WipeMethod";
import WipeProgress from "./screens/WipeProgress";
import Verification from "./screens/Verification";
import Report from "./screens/Report";
import Settings from "./screens/Settings";
import Navigation from "./components/Navigation";
import ConfirmationDialog from "./components/ConfirmationDialog";
import "./App.css";

type Screen =
  | "dashboard"
  | "device-selection"
  | "wipe-method"
  | "wipe-progress"
  | "verification"
  | "report"
  | "settings";

export const NavigationContext = createContext<{
  screen: Screen;
  setScreen: (s: Screen) => void;
} | undefined>(undefined);

const theme = createTheme({
  palette: {
    mode: "light", // TODO: Make this dynamic for dark mode
    primary: { main: "#1976d2" },
    secondary: { main: "#00bcd4" },
    error: { main: "#d32f2f" },
    success: { main: "#388e3c" },
    warning: { main: "#fbc02d" },
  },
  shape: { borderRadius: 12 },
  typography: { fontFamily: "Inter, Roboto, Arial, sans-serif" },
});

const App: React.FC = () => {
  const [screen, setScreen] = useState<Screen>("dashboard");
  const [showConfirm, setShowConfirm] = useState(false);
  const [selectedDevice, setSelectedDevice] = useState<string>("");

  useEffect(() => {
    const handler = (e: any) => {
      if (e.detail) setScreen(e.detail);
    };
    window.addEventListener("navigate", handler);
    return () => window.removeEventListener("navigate", handler);
  }, []);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <NavigationContext.Provider value={{ screen, setScreen }}>
        <div className="app-root">
          <Navigation />
          {screen === "dashboard" && <Dashboard />}
          {screen === "device-selection" && <DeviceSelection />}
          {screen === "wipe-method" && <WipeMethod />}
          {screen === "wipe-progress" && <WipeProgress />}
          {screen === "verification" && <Verification />}
          {screen === "report" && <Report />}
          {screen === "settings" && <Settings />}
          <ConfirmationDialog
            open={showConfirm}
            onCancel={() => setShowConfirm(false)}
            onConfirm={() => setShowConfirm(false)}
            deviceName={selectedDevice}
          />
        </div>
      </NavigationContext.Provider>
    </ThemeProvider>
  );
};

export default App;

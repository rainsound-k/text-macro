import { getCurrentWindow } from "@tauri-apps/api/window";
import MacroPicker from "./picker/MacroPicker";
import Settings from "./settings/Settings";

const label = getCurrentWindow().label;

export default function App() {
  if (label === "picker") return <MacroPicker />;
  if (label === "settings") return <Settings />;
  return null;
}

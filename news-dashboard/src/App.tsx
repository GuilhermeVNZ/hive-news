import { Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import PagesConfig from "./pages/PagesConfig";
import Sources from "./pages/Sources";
import Logs from "./pages/Logs";

function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/pages" element={<PagesConfig />} />
        <Route path="/sources" element={<Sources />} />
        <Route path="/logs" element={<Logs />} />
      </Routes>
    </Layout>
  );
}

export default App;


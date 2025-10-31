import { Routes, Route } from "react-router-dom";
import { AuthProvider } from "./context/AuthContext";
import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import Sites from "./pages/Sites";
import Sources from "./pages/Sources";
import Logs from "./pages/Logs";
import Writer from "./pages/Writer";
import Educational from "./pages/Educational";

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route
          path="/*"
          element={
            <ProtectedRoute>
              <Layout>
                <Routes>
                  <Route path="/" element={<Dashboard />} />
                  {/** Pages route removed */}
                  <Route path="/sites" element={<Sites />} />
                  <Route path="/writer" element={<Writer />} />
                  <Route path="/educational" element={<Educational />} />
                  <Route path="/sources" element={<Sources />} />
                  <Route path="/logs" element={<Logs />} />
                </Routes>
              </Layout>
            </ProtectedRoute>
          }
        />
      </Routes>
    </AuthProvider>
  );
}

export default App;


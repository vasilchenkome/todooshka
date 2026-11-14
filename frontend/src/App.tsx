import { Navigate, Route, Routes } from "react-router-dom";

import LoginPage from "./features/auth/LoginPage";
import { RequireAuth } from "./features/auth/RequireAuth";
import BoardPage from "./features/board/BoardPage";
import ProjectsPage from "./features/projects/ProjectsPage";

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/"
        element={
          <RequireAuth>
            <ProjectsPage />
          </RequireAuth>
        }
      />
      <Route
        path="/projects/:projectId"
        element={
          <RequireAuth>
            <BoardPage />
          </RequireAuth>
        }
      />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

# Minesweeper

A full-stack Minesweeper game built with skaffold allowing a flexible backend and frontend, designed to run on Kubernetes with automated development workflows.

## 🚀 Quick Start

1.  **Dependencies:** Ensure you have `docker`, `minikube`, `skaffold`, and `kubectl` installed. 
    *Note: You do **not** need the .NET SDK installed on your machine; all building and testing is handled via Docker.*
2.  **Auth Setup:** Create a Google OAuth Client in the [Google Cloud Console](https://console.cloud.google.com/).
3.  **Run:** Execute the quick-start script:
    ```bash
    ./quick-start.sh
    ```
    *This script will verify your environment, start Minikube, and launch `skaffold dev`.*

---

## 🧪 Testing

The backend uses **Testcontainers** for integration testing with a real database. To run these tests without needing the .NET SDK locally, use the provided script which mounts the Docker socket:

```bash
./src/backend/dotnet/run-tests.sh
```

---

## 🛠 Tech Stack

-   **Orchestration:** Kubernetes & Skaffold
-   **Backend:** ASP.NET Core 6.0 Web API
-   **Frontend:** Angular 13
-   **Frontend** .Net MVC
-   **Database:** MongoDB (or In-Memory for local testing)
-   **Auth:** Google OAuth 2.0

## 📂 Project Structure

-   `/src/backend/dotnet`: The core Web API logic and MongoDB integration.
-   `/src/frontend/angular`: The primary web interface.
-   `/src/frontend/dotnet`: An alternative .NET MVC frontend implementation.

## 🔐 Configuration
Secrets are managed via Kustomize. Add your credentials to:
`src/backend/dotnet/src/kubernetes-manifests/.auth.env`